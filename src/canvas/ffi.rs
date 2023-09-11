use fontconfig_sys::{ffi_dispatch, *};
use std::ffi::CString;

pub fn font_config_init() -> bool {
    // Ensure Fontconfig is initialized
    let result = unsafe { ffi_dispatch!(LIB, FcInit,) };
    result == 1
}

pub fn font_config_add_file<P: AsRef<std::path::Path>>(path: P) -> bool {
    let path = CString::new(path.as_ref().to_str().unwrap()).unwrap();

    let current = unsafe { ffi_dispatch!(LIB, FcConfigGetCurrent,) };
    let result = unsafe { ffi_dispatch!(LIB, FcConfigAppFontAddFile, current, path.as_ptr() as _) };
    result == 1
}
