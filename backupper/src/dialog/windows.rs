//! Contains the windows implementation of the dialog functions

use windows::{core::*, Win32::UI::WindowsAndMessaging::*};

use super::DialogResult;

fn as_u16_vec(text: &str) -> Vec<u16> {
    text.encode_utf16().collect()
}

fn generic_message_box(title: &str, msg: &str, style: MESSAGEBOX_STYLE) -> MESSAGEBOX_RESULT {
    let title = as_u16_vec(&format!("{}\0", title));
    let msg: Vec<u16> = as_u16_vec(&format!("{}\0", msg));
    unsafe {
        MessageBoxW(
            None,
            PCWSTR::from_raw(msg.as_ptr()),
            PCWSTR::from_raw(title.as_ptr()),
            style,
        )
    }
}

/// Converts a windows MessageBoxW Result to a [DialogResult].
///
/// Mapping table can be found here: https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-messagebox#return-value
fn message_box_res_to_dialog_res(message_box_result: i32) -> DialogResult {
    match message_box_result {
        1 => DialogResult::OK,
        2 => DialogResult::CANCEL,
        4 => DialogResult::RETRY,
        _ => DialogResult::UNKNOWN,
    }
}

/// Opens a Retry-Message box with the given parameters and returns the users answer.
pub fn retry_dialog(title: &str, msg: &str) -> DialogResult {
    let res = generic_message_box(title, msg, MB_RETRYCANCEL | MB_ICONWARNING);

    message_box_res_to_dialog_res(res.0)
}

/// Opens an Info-Message box wiht the given parameters and returns the users answer.
pub fn info_dialog(title: &str, msg: &str) -> DialogResult {
    let res = generic_message_box(title, msg, MB_OK | MB_ICONINFORMATION);

    message_box_res_to_dialog_res(res.0)
}
