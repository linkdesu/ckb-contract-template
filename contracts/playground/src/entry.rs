use core::result::Result;

use ckb_std::ckb_constants::Source;
use ckb_std::high_level;
use ckb_std::syscalls::SysError;
use common::debug;

use super::error::ScriptError;

pub fn main() -> Result<(), ScriptError> {
    debug!("====== Running playground ======");

    let mut i = 0;
    loop {
        let ret = high_level::load_cell(i, Source::Input).map(Some);
        match ret {
            Ok(Some(_cell)) => {
                debug!("Found inputs[{}]", i);
            }
            Ok(None) => {}
            Err(SysError::IndexOutOfBound) => {
                break;
            }
            Err(err) => {
                return Err(err.into());
            }
        }

        i += 1;
    }

    Ok(())
}
