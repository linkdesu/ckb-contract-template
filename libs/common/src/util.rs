use core::env;

#[cfg(feature = "no_std")]
pub use blake2b_ref::{Blake2b, Blake2bBuilder};
#[cfg(feature = "std")]
pub use blake2b_rs::{Blake2b, Blake2bBuilder};

pub fn get_current_env() -> &'static str {
    env!("NETWORK")
}

const CKB_HASH_LENGTH: usize = 32;
const CKB_HASH_PERSONALIZATION: &[u8] = b"ckb-default-hash";
const CKB_HASH_EMPTY: [u8; 32] = [0u8; 32];

pub fn new_blake2b() -> Blake2b {
    Blake2bBuilder::new(CKB_HASH_LENGTH)
        .personal(CKB_HASH_PERSONALIZATION)
        .build()
}

pub fn blake2b_256<T: AsRef<[u8]>>(s: T) -> [u8; 32] {
    if s.as_ref().is_empty() {
        return CKB_HASH_EMPTY;
    }

    let mut result = [0u8; 32];
    let mut blake2b = new_blake2b();
    blake2b.update(s.as_ref());
    blake2b.finalize(&mut result);
    result
}

pub fn add(a: u64, b: u64) -> u64 {
    a + b
}

pub fn minus(a: u64, b: u64) -> u64 {
    a - b
}
