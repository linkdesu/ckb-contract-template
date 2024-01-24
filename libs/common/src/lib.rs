#![cfg_attr(target_arch = "riscv64", no_std)]
#[cfg(feature = "no_std")]
extern crate alloc;

#[macro_use]
pub mod macros;

pub mod util;
