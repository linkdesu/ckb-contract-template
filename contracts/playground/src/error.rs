use ckb_std::error::SysError;

/// Error
#[repr(i8)]
pub enum ScriptError {
    IndexOutOfBound = 1,
    ItemMissing,
    LengthNotEnough,
    Encoding,
}

impl From<SysError> for ScriptError {
    fn from(err: SysError) -> Self {
        use SysError::*;
        match err {
            IndexOutOfBound => Self::IndexOutOfBound,
            ItemMissing => Self::ItemMissing,
            LengthNotEnough(_) => Self::LengthNotEnough,
            Encoding => Self::Encoding,
            Unknown(err_code) => panic!("unexpected sys error {}", err_code),
        }
    }
}

impl Into<i8> for ScriptError {
    fn into(self) -> i8 {
        self as i8
    }
}
