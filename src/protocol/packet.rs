pub trait Packet {
    fn concat(&self) -> anyhow::Result<Vec<u8>>;
    fn size(&self) -> usize;
}
