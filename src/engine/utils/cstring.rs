use std::ffi::CStr;

macro_rules! cstr {
    ($s:expr) => {
        concat!($s, "\0").as_ptr().cast::<std::os::raw::c_char>()
    };
}

#[inline(always)]
pub fn to_cstr<'a>(raw_str: *const std::os::raw::c_char) -> &'a CStr {
    unsafe { CStr::from_ptr(raw_str) }
}

pub(crate) use cstr;
