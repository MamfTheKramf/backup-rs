//! Functions for opening dialog messages

use windows::{core::*, Win32::UI::WindowsAndMessaging::*};


fn as_u16_vec(text: &str) -> Vec<u16> {
    text.encode_utf16()
        .collect()
}

/// Opens a Retry-Message box with the given parameters and returns the users answer.
pub fn retry_dialog(title: &str, msg: &str, styles: MESSAGEBOX_STYLE) -> MESSAGEBOX_RESULT {
    let title = as_u16_vec(&format!("{}\0", title));
    let msg: Vec<u16> = as_u16_vec(&format!("{}\0", msg));
    unsafe {
        MessageBoxW(None, PCWSTR::from_raw(msg.as_ptr()), PCWSTR::from_raw(title.as_ptr()), MB_RETRYCANCEL | styles)
    }
}

/// Opens an Info-Message box wiht the given parameters and returns the users answer.
pub fn info_dialog(title: &str, msg: &str, styles: MESSAGEBOX_STYLE) -> MESSAGEBOX_RESULT {
    let title = as_u16_vec(&format!("{}\0", title));
    let msg: Vec<u16> = as_u16_vec(&format!("{}\0", msg));
    unsafe {
        MessageBoxW(None, PCWSTR::from_raw(msg.as_ptr()), PCWSTR::from_raw(title.as_ptr()), MB_OK | MB_ICONINFORMATION | styles)
    }
}