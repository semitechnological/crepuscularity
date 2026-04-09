/// Lowercase hex (no `0x` prefix) for SHA-256 digests.
pub(crate) fn bytes_to_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}
