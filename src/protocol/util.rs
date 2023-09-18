pub fn generate_xor_checksum(bytes: &[u8]) -> anyhow::Result<u8> {
    let mut acc: u8 = 0;
    for byte in bytes.iter() {
        acc ^= *byte;
    }

    Ok(acc)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    pub fn generate_xor_checksum_should_ok() {
        let bytes = vec![0x02, 0x80, 0x00, 0x00];
        let result = generate_xor_checksum(&bytes);

        assert!(result.is_ok_and(|r| r == 0x82));
    }
}
