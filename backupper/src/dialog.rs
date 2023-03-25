//! Functions for opening dialog messages
#![allow(dead_code)]

#[cfg(windows)]
mod windows;

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct DialogResult(pub i32);

pub const OK: i32 = 1;
pub const CANCEL: i32 = 2;
pub const RETRY: i32 = 4;

/// Displays a retry dialog with the given `title` and `msg`.
/// 
/// # Parameters
/// - `title`: Title of the dialog window
/// - `msg`: Message to be displayed
/// The parameters don't have to end with a null-character `'\0'`. If needed, they will be added by the function.
/// 
/// # Returns
/// [DialogResult] depending on what the user clicked on.
pub fn retry_dialog(title: &str, msg: &str) -> DialogResult {
    #[cfg(target_family = "windows")]
    windows::retry_dialog(title, msg)
}

/// Display an info dialog with the given `title` and `msg`.
/// 
/// # Parameters
/// - `title`: Title of the dialog window
/// - `msg`: Message to be displayed
/// The parameters don't have to end with a null-character `'\0'`. If needed, they will be added by the function.
/// 
/// # Returns
/// [DialogResult] depending on what the user clicked on.
pub fn info_dialog(title: &str, msg: &str) -> DialogResult {
    #[cfg(target_family = "windows")]
    windows::info_dialog(title, msg)
}