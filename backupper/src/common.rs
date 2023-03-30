//! Contains utility functions that are used in different modules.

use std::{path::PathBuf, fs};

/// Checks if the target directory specified in [ProfileConfig] is writable or not.
/// 
/// # Parameters
/// - `dir_path`: Path to directory to check for availability,
/// - `is_writable`: Whether is should be checked if the directory is writable as well; if set to `false` only existence is checked
pub fn is_target_dir_available(dir_path: &PathBuf, is_writeable: bool) -> bool {
    match fs::metadata(dir_path) {
        Err(_) => false,
        Ok(metadata) => metadata.is_dir() && (!is_writeable && !metadata.permissions().readonly()),
    }
}