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
    /// Path to directory where the backup files will be stored
    pub target_dir: PathBuf,
    /// Paths to files to include in backup
    pub files_to_include: Vec<PathBuf>,
    /// Paths to dirs to include in backup
    pub dirs_to_include: Vec<PathBuf>,
    /// Paths to files to exclude from backup. Only add files here if they would otherwise be included because of `dirs_to_include`.
    pub files_to_exclude: Vec<PathBuf>,
    /// Paths to dirs to exclude from backup. Only add files here if they would otherwise be included because of `dirs_to_include`.
    pub dirs_to_exclude: Vec<PathBuf>,
    /// Interval specifying when to make the next backup
    pub interval: Interval,
}

impl ProfileConfig {
    /// Creates new [ProfileConfig] instance.
    /// 
    /// # Params
    /// - `name`: Name of the profile,
    /// - `target_dir`: Directory to place backup files in,
    /// - `files_to_include`: List of files to include in backup,
    /// - `dirs_to_include`: List of dirs to include in backup,
    /// - `files_to_exclude`: List of files to exclude from backup,
    /// - `dirs_to_exclude`: List of dirs to exclude from backup,
    /// - `interval`: Interval specifying when to make the next backup.
    pub fn new(name: String, target_dir: PathBuf, files_to_include: Vec<PathBuf>, dirs_to_include: Vec<PathBuf>, files_to_exclude: Vec<PathBuf>, dirs_to_exclude: Vec<PathBuf>, interval: Interval) -> ProfileConfig {
        let uuid = Uuid::new_v4();
        ProfileConfig {
            name,
            uuid,
            target_dir,
            files_to_include,
            dirs_to_include,
            files_to_exclude,
            dirs_to_exclude,
            interval
        }
    }

    /// Attempts to load [ProfileConfig] from the given file.
    /// 
    /// # Returns
    /// [Ok] containing [ProfileConfig] if the file exists and is the correct format. [Error] else.
    pub fn load(file_path: &PathBuf) -> Result<ProfileConfig, Error> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);

        let config = serde_json::from_reader(reader)?;
        Ok(config)
    }

    /// Returns imutable reference to `uuid`
    pub fn get_uuid(&self) -> &Uuid {
        &self.uuid
    }

    /// Stores configuration to afile named after the own [Uuid] and places it into the directory pointed to by the given [PathBuf].
    pub fn store(&self, dir_path: &PathBuf) -> Result<(), Error> {
        let file_path = Self::dir_uuid_to_file(dir_path, self.uuid);
        let file = OpenOptions::new()
            .write(true)
            .create(true)
            .open(&file_path)?;
        let writer = BufWriter::new(file);

        match serde_json::to_writer_pretty(writer, self) {
            Err(e) => Err(e.into()),
            _ => Ok(())
        }
    }

    /// Converts a [PathBuf] describing a directory and a [Uuid] into a filename.
    fn dir_uuid_to_file(dir: &PathBuf, uuid: Uuid) -> PathBuf {
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
        let config = ProfileConfig::new(name.clone(), target_file_dir.clone(), vec![], vec![], vec![], vec![], interval);
        assert_eq!(config.name, name);
    }

    #[test]
    fn store_test() {
        let name = "Hutzi".to_string();
        let config_file_dir = PathBuf::from("test_tmp");
        let target_file_dir = PathBuf::from("ho");
        let interval = IntervalBuilder::default().build().unwrap();
        let config = ProfileConfig::new(name.clone(), target_file_dir.clone(), vec![], vec![], vec![], vec![], interval);
        assert!(config.store(&config_file_dir).is_ok());
        let file = ProfileConfig::dir_uuid_to_file(&config_file_dir, config.uuid);
        delete_file(file);
    }

    #[test]
    fn load_test() {
        let uuid = Uuid::parse_str("001a828a-30ca-4b12-9756-6ce9696ac868").unwrap();
        let config_file_dir = PathBuf::from("test_tmp");
        let file_path = ProfileConfig::dir_uuid_to_file(&config_file_dir, uuid);
        let config = ProfileConfig::load(&file_path);
        assert!(config.is_ok());
        let config = config.unwrap();
        assert_eq!(config.name, "Hutzi");
    }

    #[test]
    fn load_non_existing() {
        let uuid = Uuid::new_v4();
        let dir_path = PathBuf::from("hutzi");
        let file_path = ProfileConfig::dir_uuid_to_file(&dir_path, uuid);
        let config = ProfileConfig::load(&file_path);
        assert!(config.is_err());
    }
}