use super::*;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

pub(crate) fn hex_encode(data: Bytes32) -> String {
    let mut s = "0".repeat(64);
    faster_hex::hex_encode(&data, unsafe { &mut s.as_bytes_mut() }).unwrap();
    s
}

pub(crate) fn rand<T>(seed: T) -> u64
where
    T: Hash,
{
    let mut hasher = DefaultHasher::new();
    seed.hash(&mut hasher);
    hasher.finish()
}
