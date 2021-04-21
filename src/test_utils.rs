use super::*;

pub(crate) fn hex_encode(data: Bytes32) -> String {
    let mut s = "0".repeat(64);
    faster_hex::hex_encode(&data, unsafe { &mut s.as_bytes_mut() }).unwrap();
    s
}
