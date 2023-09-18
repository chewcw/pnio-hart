use super::module_twin::{Desired, ModuleTwin, Reported};
use crate::{client::sender::Sender, config::Config};
use std::{
    default::Default,
    ffi::{c_char, c_int, c_uchar, c_void, CStr, CString},
    ptr, str,
    sync::RwLock,
    thread,
    time::Duration,
};

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

#[derive(Debug)]
pub struct IotEdge {
    module_client_handle: IOTHUB_CLIENT_CORE_HANDLE,
    pub config: RwLock<Vec<Config>>,
}

impl IotEdge {
    pub fn new(conn_str: Option<String>) -> anyhow::Result<Self> {
        // TODO: to support more transport protocols
        // let mqtt_protocol: IOTHUB_CLIENT_TRANSPORT_PROVIDER = Some(MQTT_Protocol);
        let mqtt_protocol: IOTHUB_CLIENT_TRANSPORT_PROVIDER = Some(AMQP_Protocol);
        let config = RwLock::new(vec![Default::default()]);
        match conn_str {
            Some(c) => unsafe {
                let conn_str = CString::new(c.as_str())?.into_raw();

                let module_client =
                    IoTHubModuleClient_CreateFromConnectionString(conn_str, mqtt_protocol);

                drop(CString::from_raw(conn_str));

                Ok(Self {
                    module_client_handle: module_client,
                    config,
                })
            },
            None => unsafe {
                let module_client = IoTHubModuleClient_CreateFromEnvironment(mqtt_protocol);

                Ok(Self {
                    module_client_handle: module_client,
                    config,
                })
            },
        }
    }

    pub fn read_module_twin(&mut self) {
        unsafe {
            // pass self into the callback
            let user_ctx_cb = self as *mut Self as *mut c_void;
            IoTHubModuleClient_GetTwinAsync(
                self.module_client_handle,
                Some(Self::iothub_client_device_twin_callback),
                user_ctx_cb,
            );
        }
    }

    pub fn set_module_twin_callback(&mut self) {
        // pass self into the callback
        let user_ctx_cb = self as *mut Self as *mut c_void;
        unsafe {
            IoTHubModuleClient_SetModuleTwinCallback(
                self.module_client_handle,
                Some(Self::iothub_client_device_twin_callback),
                user_ctx_cb,
            );
        }
    }

    extern "C" fn iothub_client_device_twin_callback(
        update_state: DEVICE_TWIN_UPDATE_STATE,
        payload: *const c_uchar,
        size: usize,
        userContextCallback: *mut c_void,
    ) {
        log::info!("module twin updated");
        unsafe {
            // get the module twin
            // the signness of c_char on x86_64 is i8, while on armv7 is u8
            let c_str = CStr::from_ptr(payload as *const c_char);
            let bytes = &c_str.to_bytes()[..size];
            let twin_str = str::from_utf8(bytes).unwrap();
            log::debug!("got module twin: {}", twin_str);

            // sometime got the full module twin (with the desired and reported prop),
            // while sometime got the desired module twin only, so deserialize the
            // module twin accordingly.
            // TODO: this may not be sophisticated enough
            let mut module_twin: ModuleTwin = Default::default();
            if twin_str.contains(r#""desired":"#) && twin_str.contains(r#""reported":"#) {
                match ModuleTwin::deserialize(twin_str) {
                    Ok(twin) => module_twin = twin,
                    Err(err) => {
                        log::error!("failed to deserialize module twin: {err}");
                        return;
                    }
                };
            } else {
                match Desired::deserialize(twin_str) {
                    Ok(twin) => module_twin.desired = twin,
                    Err(err) => {
                        log::error!("failed to deserialize module twin: {err}");
                        return;
                    }
                }
            };

            let iotedge = userContextCallback as *mut IotEdge;

            // update the config object based on the module twin
            let mut config = (*iotedge).config.write().unwrap();
            *config = module_twin.desired.config;
            log::debug!("updated iotedge object: {:?}", (*iotedge));
            drop(config);

            // update the reported properties
            if let Err(err) = (*iotedge).send_reported_prop() {
                log::error!("{}", err);
            };
        }
    }

    fn send_reported_prop(&self) -> anyhow::Result<()> {
        // report null at first to clear the reported properties
        // the reason is to eliminate wrong information registered
        // during development time
        // TODO: maybe make this a flag?
        // let null: String = String::from(r#"{config: null}"#);
        // let size = null.len();
        // let null: *const c_uchar = null.as_ptr() as *const c_uchar;
        // let user_ctx_cb = ptr::null_mut();
        // unsafe {
        //     IoTHubModuleClient_SendReportedState(
        //         self.module_client_handle,
        //         null,
        //         size,
        //         Some(Self::send_reported_state_callback),
        //         user_ctx_cb,
        //     );
        // }

        // report real config
        let config = self.config.read().unwrap();
        let reported = Reported {
            version: 0, // doesn't matter
            config: Some(config.clone()),
        };

        let reported = reported.serialize()?;
        let size = reported.len();
        log::debug!("reported properties to be sent: {}", reported);

        let reported: *const c_uchar = reported.as_ptr() as *const c_uchar;
        let user_ctx_cb = ptr::null_mut();
        unsafe {
            IoTHubModuleClient_SendReportedState(
                self.module_client_handle,
                reported,
                size,
                Some(Self::send_reported_state_callback),
                user_ctx_cb,
            );
        }

        Ok(())
    }

    extern "C" fn send_reported_state_callback(
        status_code: c_int,
        userContextCallback: *mut c_void,
    ) {
        log::debug!("status of sending reported properties: {}", status_code);
    }

    extern "C" fn event_confirmation_callback(result: u32, userContextCallback: *mut c_void) {
        log::info!("result of send_event_async: {result}");
    }
}

impl Sender for IotEdge {
    fn send(&self, data: String) -> anyhow::Result<()> {
        let message = CString::new(data.as_str())?.into_raw();
        let user_ctx_cb: *mut c_void = ptr::null_mut();

        log::info!("sending message to IoT Hub");
        log::debug!("↑ message: {:?}", data);

        unsafe {
            let message_handle = IoTHubMessage_CreateFromString(message);
            // set message property
            let msg_id = CString::new("MSG_ID")?.into_raw();
            IoTHubMessage_SetMessageId(message_handle, msg_id);
            let core_id = CString::new("CORE_ID")?.into_raw();
            IoTHubMessage_SetCorrelationId(message_handle, core_id);
            let content_type = CString::new("application%2fjson")?.into_raw();
            IoTHubMessage_SetContentTypeSystemProperty(message_handle, content_type);
            let encoding = CString::new("utf-8")?.into_raw();
            IoTHubMessage_SetContentEncodingSystemProperty(message_handle, encoding);

            // send the message
            IoTHubModuleClient_SendEventAsync(
                self.module_client_handle,
                message_handle,
                Some(Self::event_confirmation_callback),
                user_ctx_cb,
            );

            thread::sleep(Duration::from_millis(500));

            log::info!("successfully sent message to IoT Hub");

            // destroy the message handle
            IoTHubMessage_Destroy(message_handle);

            drop(CString::from_raw(msg_id));
            drop(CString::from_raw(core_id));
            drop(CString::from_raw(content_type));
            drop(CString::from_raw(encoding));
            drop(CString::from_raw(message));
        }

        Ok(())
    }

    fn setup(&self) -> anyhow::Result<()> {
        // setting the auto URL Encoder (recommended for MQTT)
        // TODO: this is only for MQTT, for other transport type need to set other
        // options, for example, see https://github.com/Azure/azure-iot-sdk-c/blob/main/iothub_client/samples/iothub_convenience_sample/iothub_convenience_sample.c#L321
        // let url_encode_on = &true as *const bool as *const c_void;
        // let auto_url_encode_decode_opt = CString::new(*OPTION_AUTO_URL_ENCODE_DECODE)?.into_raw();

        // unsafe {
        //     IoTHubModuleClient_SetOption(
        //         self.module_client,
        //         auto_url_encode_decode_opt,
        //         url_encode_on,
        //     );
        //
        //     drop(CString::from_raw(auto_url_encode_decode_opt));
        // }

        Ok(())
    }

    fn get_config(&self) -> &RwLock<Vec<Config>> {
        &self.config
    }
}

#[cfg(test)]
mod test {
    use crate::dto::iotedge_message::IotedgeMessageDto;

    use super::*;
    use anyhow::anyhow;
    use std::{collections::HashMap, thread, time};

    #[test]
    fn get_device_twin() {
        let conn_str = String::from("HostName=ih-sea-sca-demo.azure-devices.net;DeviceId=test;ModuleId=test;SharedAccessKey=PcSCcAsx6kBhEek+SPR5nfyn5j8ykMS9V0D0Jyw02Eo=");
        let mut iotedge = IotEdge::new(Some(conn_str)).unwrap();
        iotedge.set_module_twin_callback();

        // sleeping for the read twin callback
        thread::sleep(time::Duration::from_secs(5));

        println!("iotedge's config: {:?}", &iotedge.config);

        println!("done...");
    }

    #[test]
    fn send_event() {
        let conn_str = String::from("HostName=ih-sea-sca-demo.azure-devices.net;DeviceId=test;ModuleId=test;SharedAccessKey=PcSCcAsx6kBhEek+SPR5nfyn5j8ykMS9V0D0Jyw02Eo=");
        let mut iotedge = IotEdge::new(Some(conn_str)).unwrap();

        let now = format!("{:?}", chrono::Utc::now());
        let message = IotedgeMessageDto {
            timestamp: now.as_str(),
            device_unique_name: "device_unique_name",
            hart_device_name: "hart_device_name",
            hart_command: 48,
            length: 10,
            bytes: &[0x01, 0x02, 0x03],
        };

        let message = serde_json::to_string(&message).unwrap();

        iotedge.send(message);

        thread::sleep(time::Duration::from_millis(700));
    }
}
