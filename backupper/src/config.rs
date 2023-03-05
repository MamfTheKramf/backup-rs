//! Contains functions and methods for loading and managing configs.

use std::path::PathBuf;

use config::general_config::GeneralConfig;

const GENERAL_CONFIG_PATH: &'static str = "./test_dir/general_config.json";


/// Loads the general Config file.
/// 
/// # Returns
/// [Result::Ok] containing the config if it could be loaded.
/// [Result::Err] containing a [String] describing the issue if some occured.
pub fn load_general_config() -> Result<GeneralConfig, String> {
    let path = PathBuf::from(GENERAL_CONFIG_PATH);

    match path.try_exists() {
        Ok(val) => if !val { return Err(String::from("General config file doesn't exist!")); },
        Err(e) => return Err(e.to_string()),
    }

    if !path.is_file() {
        return Err(String::from("Path to general config file doesn't point to a file!"));
    }

    match GeneralConfig::read(path) {
        Err(e) => Err(e.to_string()),
        Ok(config) => Ok(config),
    }
}