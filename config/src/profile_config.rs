//! Contains structs and functions for profile configurations.

use std::{path::PathBuf, fs::{OpenOptions, File}, io::{Error, BufWriter, BufReader}};

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::interval::*;

/// Struct representing a profile configuration.
#[derive(Debug, Deserialize, Serialize)]
pub struct ProfileConfig {
    /// Descriptive name of the profile; doesn't need to be unique
    pub name: String,
    /// Unique identifier for the profile; will be generated automatically, when the profile is created
    uuid: Uuid,
    /// Path to the config file where this profile will be read from / stored at
    config_file: PathBuf,
    /// Path to directory where the backup files will be stored
    pub target_dir: PathBuf,
    /// Paths to files to include in backup
    pub files_to_include: Vec<PathBuf>,
    /// Interval specifying when to make the next backup
    pub interval: Interval, // TODO
}

impl ProfileConfig {
    /// Creates new [ProfileConfig] instance.
    /// 
    /// # Params
    /// - `name`: Name of the profile,
    /// - `config_file_dir`: Directory in which to place this config when stored. Do NOT provide a filename. This will be done automatically,
    /// - `target_dir`: Directory to place backup files in,
    /// - `files_to_include`: List of files to include in backup,
    /// - `interval`: Interval specifying when to make the next backup.
    pub fn new(name: String, config_file_dir: PathBuf, target_dir: PathBuf, files_to_include: Vec<PathBuf>, interval: Interval) -> ProfileConfig {
        let uuid = Uuid::new_v4();
        let config_file = Self::dir_uuid_to_file(config_file_dir, uuid);
        ProfileConfig {
            name,
            uuid,
            config_file,
            target_dir,
            files_to_include,
            interval
        }
    }

    /// Attempts to load [ProfileConfig] for the given [Uuid] from the specified directory.
    /// 
    /// # Returns
    /// [Ok] containing [ProfileConfig] if the file exists and is the correct format. [Error] else.
    pub fn load(config_file_dir: PathBuf, uuid: Uuid) -> Result<ProfileConfig, Error> {
        let file_path = Self::dir_uuid_to_file(config_file_dir, uuid);
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);

        let config = serde_json::from_reader(reader)?;
        Ok(config)
    }

    /// Returns imutable reference to `uuid`
    pub fn get_uuid(&self) -> &Uuid {
        &self.uuid
    }

    /// Returns immutable reference to `config_file`
    pub fn get_config_file(&self) -> &PathBuf {
        &self.config_file
    }

    /// Stores configuration to the files specified in `config_file`
    pub fn store(&self) -> Result<(), Error> {
        let file = OpenOptions::new()
            .write(true)
            .create(true)
            .open(&self.config_file)?;
        let writer = BufWriter::new(file);

        match serde_json::to_writer_pretty(writer, self) {
            Err(e) => Err(e.into()),
            _ => Ok(())
        }
    }

    /// Converts a [PathBuf] describing a directory and a [Uuid] into a filename.
    fn dir_uuid_to_file(dir: PathBuf, uuid: Uuid) -> PathBuf {
        PathBuf::from(format!("{}/{}.json",
            dir.to_str().unwrap_or(""),
            uuid.as_hyphenated()
        ))
    }
}

#[cfg(test)]
mod profile_config_tests {
    use super::*;
    use std::fs;

    fn delete_file(path: PathBuf) {
        match fs::remove_file(path) {
            Err(e) => println!("{:?}", e),
            _ => (),
        }
    }

    #[test]
    fn new_test() {
        let name = "Hutzi".to_string();
        let config_file_dir = PathBuf::from("hi");
        let target_file_dir = PathBuf::from("ho");
        let interval = IntervalBuilder::default().build().unwrap();
        let config = ProfileConfig::new(name.clone(), config_file_dir.clone(), target_file_dir.clone(), vec![], interval);
        assert_eq!(config.name, name);
        assert_eq!(config.get_config_file().file_name().unwrap().to_str().unwrap(), format!("{}.json", config.get_uuid().as_hyphenated()));
    }

    #[test]
    fn store_test() {
        let name = "Hutzi".to_string();
        let config_file_dir = PathBuf::from("test_tmp");
        let target_file_dir = PathBuf::from("ho");
        let interval = IntervalBuilder::default().build().unwrap();
        let config = ProfileConfig::new(name.clone(), config_file_dir.clone(), target_file_dir.clone(), vec![], interval);
        assert!(config.store().is_ok());
        delete_file(config.get_config_file().clone());
    }

    #[test]
    fn load_test() {
        let uuid = Uuid::parse_str("001a828a-30ca-4b12-9756-6ce9696ac868").unwrap();
        let config_file_dir = PathBuf::from("test_tmp");
        let config = ProfileConfig::load(config_file_dir, uuid.clone());
        assert!(config.is_ok());
        let config = config.unwrap();
        assert_eq!(config.name, "Hutzi");
    }

    #[test]
    fn load_non_existing() {
        let uuid = Uuid::new_v4();
        let dir_path = PathBuf::from("hutzi");
        let config = ProfileConfig::load(dir_path, uuid);
        assert!(config.is_err());
    }
}