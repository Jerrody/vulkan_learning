use std::ffi::CStr;

pub const unsafe fn cstr(str: &str) -> &CStr {
    CStr::from_bytes_with_nul_unchecked(str.as_bytes())
}
