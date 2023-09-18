use core::fmt::Debug;
use std::{convert::Infallible, net::IpAddr};

pub trait TransportClient {
    fn send(&self, data: Box<[u8]>) -> anyhow::Result<usize>;
    fn receive(&self) -> anyhow::Result<Box<[u8]>>;
    fn get_dst_conn_details(&self) -> anyhow::Result<(IpAddr, u16), Infallible>;
    fn debug(&self) -> String;
}

impl Debug for dyn TransportClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.debug())
    }
}
