//! Contains structs and functions for profile configurations.

use std::{
    fs::{File, OpenOptions},
    io::{BufReader, BufWriter, Error},
    path::PathBuf,
};

use crate::interval::*;
use chrono::{offset, Days, NaiveDateTime};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

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
    /// Datetime specifying when the next backup should be made
    next_backup: NaiveDateTime,
}

impl ProfileConfig {
    /// Creates new [ProfileConfig] instance.
    /// The `next_backup` field gets set to the creation time of this instance.
    ///
    /// # Params
    /// - `name`: Name of the profile,
    /// - `target_dir`: Directory to place backup files in,
    /// - `files_to_include`: List of files to include in backup,
    /// - `dirs_to_include`: List of dirs to include in backup,
    /// - `files_to_exclude`: List of files to exclude from backup,
    /// - `dirs_to_exclude`: List of dirs to exclude from backup,
    /// - `interval`: Interval specifying when to make the next backup.
    pub fn new(
        name: String,
        target_dir: PathBuf,
        files_to_include: Vec<PathBuf>,
        dirs_to_include: Vec<PathBuf>,
        files_to_exclude: Vec<PathBuf>,
        dirs_to_exclude: Vec<PathBuf>,
        interval: Interval,
    ) -> ProfileConfig {
        let uuid = Uuid::new_v4();
        let now = offset::Local::now().naive_local();

        ProfileConfig {
            name,
            uuid,
            target_dir,
            files_to_include,
            dirs_to_include,
            files_to_exclude,
            dirs_to_exclude,
            interval,
            next_backup: now,
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
            _ => Ok(()),
        }
    }

    /// Returns immutable reference to `uuid`
    pub fn get_uuid(&self) -> &Uuid {
        &self.uuid
    }

    /// Returns immutable reference to `next_backup`.
    /// `next_backup` isn't guaranteed to be matched by `interval. It must always be checked first.
    pub fn next_backup(&self) -> &NaiveDateTime {
        &self.next_backup
    }

    /// Updates the `next_backup` field according to `interval`.
    /// The new values is always at least the current time.
    ///
    /// # Returns
    /// Immutable reference to the `next_backup` field.
    /// It isn't guaranteed, that `next_backup` is matched by `interval`. It must alwys be checked first.
    pub fn set_next_backup(&mut self) -> &NaiveDateTime {
        let now = offset::Local::now().naive_local();
        if now >= self.next_backup {
            self.next_backup = now;
        }

        match self.interval.next_datetime(self.next_backup) {
            Some(datetime) => self.next_backup = datetime,
            None => {
                self.next_backup = self
                    .next_backup
                    .checked_add_days(Days::new(365))
                    .unwrap_or(self.next_backup)
            }
        }

        self.next_backup()
    }

    /// Converts a [PathBuf] describing a directory and a [Uuid] into a filename.
    fn dir_uuid_to_file(dir: &PathBuf, uuid: Uuid) -> PathBuf {
        PathBuf::from(format!(
            "{}/{}.json",
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
        let target_file_dir = PathBuf::from("ho");
        let interval = IntervalBuilder::default().build().unwrap();
        let config = ProfileConfig::new(
            name.clone(),
            target_file_dir.clone(),
            vec![],
            vec![],
            vec![],
            vec![],
            interval,
        );
        assert_eq!(config.name, name);
    }

    #[test]
    fn store_test() {
        let name = "Hutzi".to_string();
        let config_file_dir = PathBuf::from("test_tmp");
        let target_file_dir = PathBuf::from("ho");
        let interval = IntervalBuilder::default().build().unwrap();
        let config = ProfileConfig::new(
            name.clone(),
            target_file_dir.clone(),
            vec![],
            vec![],
            vec![],
            vec![],
            interval,
        );
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

    mod set_next_backup_tests {
        use chrono::{NaiveDate, Datelike};

        use super::*;

        #[test]
        fn found_next_backup() {
            let interval = IntervalBuilder::default()
                .minutes(SpecifierKind::First)
                .hours(SpecifierKind::First)
                .monthdays(SpecifierKind::First)
                .build()
                .unwrap();
            let mut p = ProfileConfig::new(
                String::from("Hutzi"),
                PathBuf::from("ho"),
                vec![],
                vec![],
                vec![],
                vec![],
                interval,
            );
            let now = offset::Local::now().naive_local();
            let next_month = now.month() % 12 + 1;
            let next_year = now.year() + now.month() as i32 / 12;
            let next_match = NaiveDate::from_ymd_opt(next_year, next_month, 1).unwrap()
                .and_hms_opt(0, 0, 0).unwrap();
            assert_eq!(p.set_next_backup(), &next_match);

            p.next_backup = NaiveDate::from_ymd_opt(2000, 12, 16).unwrap()
                .and_hms_opt(12, 30, 6).unwrap();
            let now = offset::Local::now().naive_local();
            let next_month = now.month() % 12 + 1;
            let next_year = now.year() + now.month() as i32 / 12;
            let next_match = NaiveDate::from_ymd_opt(next_year, next_month, 1).unwrap()
                .and_hms_opt(0, 0, 0).unwrap();
            assert_eq!(p.set_next_backup(), &next_match);
        }

        #[test]
        fn not_found_next_backup() {
            let interval = IntervalBuilder::default()
                .minutes(SpecifierKind::None)
                .build()
                .unwrap();
            let mut p = ProfileConfig::new(
                String::from("Hutzi"),
                PathBuf::from("ho"),
                vec![],
                vec![],
                vec![],
                vec![],
                interval,
            );

            let now = offset::Local::now().naive_local();
            assert!(&now <= p.set_next_backup());
        }
    }
}
