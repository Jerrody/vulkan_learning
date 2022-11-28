#[macro_export]
macro_rules! cstr {
    ($s:expr) => {
        unsafe { std::ffi::CStr::from_bytes_with_nul_unchecked(concat!($s, "\0").as_bytes()) }
    };
}

pub(crate) use cstr;
