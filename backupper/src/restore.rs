//! Contains function for restoring a backup.

use std::{
    collections::HashMap,
    fs::{self, File},
    io,
    path::PathBuf,
};

use chrono::NaiveDateTime;
use config::profile_config::ProfileConfig;
use log::{debug, error, info};
use uuid::Uuid;
use zip::{read::ZipFile, ZipArchive};

use crate::{
    cli_args::Args,
    common::is_target_dir_available,
    consts::{FILE_RECORD_NAME, MANIFEST_NAME, PROFILE_CONF_NAME, RESERVED_FILENAMES},
    dialog::{retry_dialog, DialogResult},
    manifest::Manifest,
};

/// Restores the files from the latest backup of the provided [ProfileConfig] that is older than the given `timestamp`.
///
/// If there is no such backup, nothing happens.
pub fn restore(profile_config: &ProfileConfig, timestamp: NaiveDateTime, _args: &Args) {
    if !available_target_dir_dialog(profile_config) {
        info!(
            "Target dir {:?} wasn't available and canceled.",
            profile_config.target_dir
        );
        return;
    }

    let best_backup = find_backup_archive(profile_config, timestamp);
    debug!("Found best: {:?}", best_backup);
    if best_backup.is_none() {
        return;
    }

    let best_backup = best_backup.unwrap();
    restore_from_backup(best_backup);
}

/// Opens retry dialog to attach external drive if the `profile_config`s target directory is not available.
///
/// # Returns
/// `true` if the restoring shall proceed.
/// `false` if cancel was selected.
fn available_target_dir_dialog(profile_config: &ProfileConfig) -> bool {
    // make sure, directory is available
    let mut choice = DialogResult::Retry;
    while !is_target_dir_available(&profile_config.target_dir, false)
        && choice == DialogResult::Retry
    {
        let msg = format!("Das Verzeichnis mit den Backups {:?} scheint nicht verfügpar zu sein.\nBitte schließe die externe Festplatte an und versuche es erneut.", profile_config.target_dir);
        let title = "Backupverzeichnis nicht verfügbar.";
        choice = retry_dialog(title, &msg);
    }

    choice == DialogResult::Retry
}

/// Finds the latest backup file in the target dir that is older than the provided timestamp.
///
/// Returns [None] if no such backup file was found. This function doesn't go through the target dir recursively.
fn find_backup_archive(
    profile_config: &ProfileConfig,
    timestamp: NaiveDateTime,
) -> Option<PathBuf> {
    let entries = match fs::read_dir(&profile_config.target_dir) {
        Ok(entries) => entries,
        Err(err) => {
            error!("Error reading dir: {:?}", err);
            return None;
        }
    };

    let timestamp = timestamp
        .and_local_timezone(chrono::Local)
        .earliest()
        .map(|timestamp| timestamp.timestamp())
        .unwrap_or(chrono::Local::now().timestamp());

    let mut best_backup = None;

    for entry in entries {
        if entry.is_err() {
            continue;
        }

        let entry = entry.unwrap();
        let path = entry.path();

        if path.is_dir() {
            continue;
        }

        let file = match File::open(&path) {
            Ok(file) => file,
            _ => continue,
        };
        let mut zip = match ZipArchive::new(file) {
            Ok(archive) => archive,
            _ => continue,
        };

        let backup_uuid = match zip
            .by_name(PROFILE_CONF_NAME)
            .or(Err(String::from("ProfileConfig entry not found")))
            .and_then(|zip_file| {
                serde_json::from_reader::<ZipFile<'_>, ProfileConfig>(zip_file)
                    .map_err(|e| format!("Couldn't parse ProfileConfig in backup: {:?}", e))
            }) {
            Ok(conf) => *conf.get_uuid(),
            Err(_) => continue,
        };
        if &backup_uuid != profile_config.get_uuid() {
            continue;
        }

        let manifest = match zip.by_name(MANIFEST_NAME).or(Err(())).and_then(|zip_file| {
            serde_json::from_reader::<ZipFile<'_>, Manifest>(zip_file).or(Err(()))
        }) {
            Ok(manifest) => manifest,
            _ => continue,
        };
        if !manifest.matches() {
            continue;
        }

        if manifest.created <= timestamp
            && manifest.created
                > best_backup
                    .as_ref()
                    .map(|&(current_best_ts, _)| current_best_ts)
                    .unwrap_or(i64::MIN)
        {
            best_backup = Some((manifest.created, path));
        }
    }

    best_backup.map(|(_, path)| path)
}

/// Restores each file in the given backup.
/// If a file already exists, it is everwritten. If it doesn't exist, it is created.
fn restore_from_backup(backup_file: PathBuf) {
    let file = match File::open(&backup_file) {
        Ok(file) => file,
        Err(e) => {
            error!("Error opening file {:?}: {:?}", backup_file, e);
            return;
        }
    };

    let mut zip = match ZipArchive::new(file) {
        Ok(file) => file,
        Err(e) => {
            error!("Couldn't create archive because {:?}", e);
            return;
        }
    };
    let file_record = match zip.by_name(FILE_RECORD_NAME) {
        Ok(file) => file,
        Err(e) => {
            error!("Couldn't find FileRecord bacaus {:?}", e);
            return;
        }
    };
    let file_record: HashMap<Uuid, String> = match serde_json::from_reader(file_record) {
        Ok(res) => res,
        Err(e) => {
            error!("Couldn't parse FileRecord: {:?}", e);
            return;
        }
    };

    for i in 0..zip.len() {
        let mut file = match zip.by_index(i) {
            Ok(file) => file,
            Err(e) => {
                error!("Error extracting file: {:?}", e);
                return;
            }
        };
        if RESERVED_FILENAMES.contains(&file.name()) {
            continue;
        }

        let id = match Uuid::parse_str(file.name()) {
            Ok(uuid) => uuid,
            Err(e) => {
                error!("Couldn't parse file id {} as uuid: {:?}", file.name(), e);
                return;
            }
        };
        let filepath = match file_record.get(&id) {
            Some(path) => PathBuf::from(path),
            None => {
                error!(
                    "Id {} not found in FileRecord. Couldn't map to file path",
                    id
                );
                return;
            }
        };

        if let Some(p) = filepath.parent() {
            if !p.exists() {
                if let Err(e) = fs::create_dir_all(p) {
                    error!(
                        "Couldn't create dir {:?} because {:?}",
                        filepath.parent(),
                        e
                    );
                    return;
                }
            }
        }
        info!("Restore file {:?}", filepath);
        let mut outfile = match fs::File::create(&filepath) {
            Ok(outfile) => outfile,
            Err(e) => {
                error!("Couldn't create outfile {:?} because {:?}", filepath, e);
                return;
            }
        };
        if let Err(e) = io::copy(&mut file, &mut outfile) {
            error!("Couldn't copy to outfile because {:?}", e);
            return;
        }
    }
}
