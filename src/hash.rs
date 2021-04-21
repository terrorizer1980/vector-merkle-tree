use super::Bytes32;
use tiny_keccak::{Hasher, Keccak};

pub fn keccak(data: &[u8]) -> Bytes32 {
    let mut hash = [0; 32];
    let mut hasher = Keccak::v256();
    hasher.update(data);
    hasher.finalize(&mut hash);
    hash
}

pub fn combine(a: &Bytes32, b: &Bytes32) -> Bytes32 {
    let (first, second) = if a < b { (a, b) } else { (b, a) };

    let mut keccak256 = Keccak::v256();
    keccak256.update(first);
    keccak256.update(second);
    let mut into = [0; 32];
    keccak256.finalize(&mut into);
    into
}
