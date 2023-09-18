use crate::transport::transport_client::TransportClient;
use anyhow::anyhow;
use std::{
    cell::RefCell,
    convert::Infallible,
    net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket},
    time::Duration,
};

#[derive(Debug)]
pub struct UdpClient {
    socket: UdpSocket,
    dst_socket_addr: RefCell<SocketAddr>,
}

impl Clone for UdpClient {
    fn clone(&self) -> Self {
        Self {
            socket: self.socket.try_clone().unwrap(),
            dst_socket_addr: self.dst_socket_addr.clone(),
        }
    }
}

impl UdpClient {
    pub const SRC_UDP_PORT: u16 = 53212; // just arbitrary port number

    pub fn new(src_ip: Ipv4Addr, dst_ip: Ipv4Addr, dst_udpport: u16) -> anyhow::Result<Self> {
        let src_udpsocket = format!("{src_ip}:{}", Self::SRC_UDP_PORT);
        let socket = UdpSocket::bind(src_udpsocket)?;

        let _ = socket.set_read_timeout(Some(Duration::new(3, 0)));
        let _ = socket.set_write_timeout(Some(Duration::new(3, 0)));

        let dst_ipv4 = IpAddr::V4(dst_ip); // only supports ipv4 for now
        let dst_socket_addr = RefCell::new(SocketAddr::from((dst_ipv4, dst_udpport)));

        Ok(UdpClient {
            socket,
            dst_socket_addr,
        })
    }

    pub fn update_dest(&self, dst_ip: Ipv4Addr, dst_udpport: u16) -> anyhow::Result<()> {
        let dst_socket_addr = SocketAddr::from((dst_ip, dst_udpport));
        self.dst_socket_addr.replace(dst_socket_addr);

        Ok(())
    }
}

impl TransportClient for UdpClient {
    fn send(&self, data: Box<[u8]>) -> anyhow::Result<usize> {
        match self
            .socket
            .send_to(&data, *self.dst_socket_addr.borrow())
            .map_err(|err| Err(anyhow!("failed to send packet, error: {}", err)))
        {
            Ok(send_resp_size) => Ok(send_resp_size),
            Err(err) => err,
        }
    }

    fn receive(&self) -> anyhow::Result<Box<[u8]>> {
        let mut buf: Vec<u8> = vec![0; 300]; // pre-allocate 300 bytes
        match self
            .socket
            .recv_from(&mut buf)
            .map_err(|err| Err(anyhow!("failed to receive packet, error: {}", err)))
        {
            Ok(_) => Ok(buf.into_boxed_slice()),
            Err(err) => err,
        }
    }

    fn get_dst_conn_details(&self) -> anyhow::Result<(IpAddr, u16), Infallible> {
        Ok((
            self.dst_socket_addr.borrow().ip(),
            self.dst_socket_addr.borrow().port(),
        ))
    }

    fn debug(&self) -> String {
        let socket = format!(
            "{}:{}",
            self.socket.local_addr().unwrap().ip(),
            self.socket.local_addr().unwrap().port()
        );
        let dst_socket_addr = self.dst_socket_addr.borrow().to_string();
        format!("{} {}", socket, dst_socket_addr)
    }
}
