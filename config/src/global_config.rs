//! Contains structs and functions for the general program configuration

use serde::{Deserialize, Serialize};
use std::{
    fs::{File, OpenOptions},
    io::{BufReader, BufWriter, Error},
    path::PathBuf,
};

/// Class containing general configuration.
/// Can read general configuration from a given file and store it in a given file.
/// 
/// `profile_configs` specify the path to the directory in which the `.json` files for the profiles can be found.
#[derive(Debug, Deserialize, Serialize)]
pub struct GeneralConfig {
    pub profile_configs: PathBuf,
}

impl GeneralConfig {
    /// Reads general configuration from the file at the provided path.
    /// 
    /// # Returns
    /// [Ok] containing a [GeneralConfig] instance or an [Error]
    pub fn read(global_config_file: PathBuf) -> Result<GeneralConfig, Error> {
        let file = File::open(global_config_file)?;
        let reader = BufReader::new(file);

        let config = serde_json::from_reader(reader)?;

        Ok(config)
    }

    /// Stores general configuration from the file at the provided path.
    /// Will create a file, if it doesn't exist yet.
    /// 
    /// # Returns
    /// [Ok] containing a [GeneralConfig] instance or an [Error]
    pub fn store(&self, global_config_file: PathBuf) -> Result<(), Error> {
        let file = OpenOptions::new()
            .write(true)
            .create(true)
            .open(global_config_file)?;
        let writer = BufWriter::new(file);

        match serde_json::to_writer_pretty(writer, self) {
            Ok(_) => Ok(()),
            Err(e) => Err(e.into()),
        }
    }
}

#[cfg(test)]
mod global_config_tests {
    use super::*;
    use std::fs;

    fn delete_file(path: PathBuf) {
        match fs::remove_file(path) {
            Err(e) => println!("{:?}", e),
            _ => (),
        }
    }

    #[test]
    fn store_test() -> Result<(), Error> {
        let tmp_file = PathBuf::from("test_tmp/store_test.json");
        let config = GeneralConfig {
            profile_configs: PathBuf::from("test"),
        };
        config.store(tmp_file.clone())?;
        delete_file(tmp_file);
        Ok(())
    }

    #[test]
    fn read_test() -> Result<(), Error> {
        let tmp_file = PathBuf::from("test_tmp/read_test.json");
        let config = GeneralConfig::read(tmp_file)?;
        assert_eq!(config.profile_configs, PathBuf::from("test"));
        Ok(())
    }

    #[test]
    fn read_non_existing() {
        let file = PathBuf::from("Non-existing.abc");
        let config = GeneralConfig::read(file);
        assert!(config.is_err());
    }
}
