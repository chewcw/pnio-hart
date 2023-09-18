use crate::protocol::util;

pub struct HartCommand {}
impl HartCommand {
    const TRANSPARENT_MESSAGE_FORMAT: u8 = 0x00;

    pub fn construct_write_request(
        device_id: [u8; 5],
        command: u8,
        write_payload: Option<&[u8]>,
    ) -> anyhow::Result<Box<[u8]>> {
        let mut data: Vec<u8>;

        // command 0 with short device address is the special one
        // its purpose is to get device_id from the response
        // for other HART commands
        if command == 0 && device_id == [0x00; 5] {
            data = vec![
                Self::TRANSPARENT_MESSAGE_FORMAT, // transparent message format
                0x14,                             // number of preamble bytes
                0x02,                             // short frame with command 0
                0x00,                             // with command 0, this is 0x00
                0x00,                             // command 0
                0x00,                             // length in bytes
            ];
        } else {
            // the rest of HART commands can use long device address (device_id)
            // obtain through command 0
            data = vec![
                Self::TRANSPARENT_MESSAGE_FORMAT, // transparent message format
                0x05,                             // number of preamble bytes
                0x82,                             // long frame with command other than command 0
                device_id[0],                     // device_id
                device_id[1],
                device_id[2],
                device_id[3],
                device_id[4],
                command, // HART command, for example 48
            ];

            // TODO: uncertain
            // length in bytes
            // number of bytes to follow in the status and data bytes
            // I think set it to 0x01 should be fine
            data.push(0x01);

            // payload (if any)
            if let Some(w) = write_payload {
                data.extend(w.as_ref());
            }
        }

        // checksum
        let checksum = util::generate_xor_checksum(&data[2..])?;
        data.push(checksum);

        Ok(data.into_boxed_slice())
    }
}
