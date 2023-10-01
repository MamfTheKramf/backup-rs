//! Contains function for restoring a backup.

use std::{fs::{self, File}, path::PathBuf, io};

use chrono::NaiveDateTime;
use config::profile_config::ProfileConfig;
use log::{error, info, warn, debug};
use zip::ZipArchive;

use crate::{
    cli_args::Args,
    common::is_target_dir_available,
    dialog::{retry_dialog, DialogResult, RETRY}, 
};

/// Restores the files from the latest backup of the provided [ProfileConfig] that is older than the given `timestamp`.
///
/// If there is no such backup, nothing happens.
pub fn restore(profile_config: &ProfileConfig, timestamp: NaiveDateTime, _args: &Args) {
    if !available_target_dir_dialog(profile_config) {
        info!("Target dir {:?} wasn't available and canceled.", profile_config.target_dir);
        return;
    }

    let best_backup = find_backup_archive(profile_config, timestamp);
    println!("Found best: {:?}", best_backup);
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
    let mut choice = DialogResult(RETRY);
    while !is_target_dir_available(&profile_config.target_dir, false)
        && choice == DialogResult(RETRY)
    {
        let msg = format!("Das Verzeichnis mit den Backups {:?} scheint nicht verfügpar zu sein.\nBitte schließe die externe Festplatte an und versuche es erneut.", profile_config.target_dir);
        let title = "Backupverzeichnis nicht verfügbar.";
        choice = retry_dialog(title, &msg);
    }

    choice == DialogResult(RETRY)
}

/// Finds the latest backup file in the target dir that is older than the provided timestamp.
/// 
/// Returns [None] if no such backup file was found. This function doesn't go through the target dir recursively.
fn find_backup_archive(profile_config: &ProfileConfig, timestamp: NaiveDateTime) -> Option<PathBuf> {
    let entries = match fs::read_dir(&profile_config.target_dir) {
        Ok(entries) => entries,
        Err(err) => {
            error!("Error reading dir: {:?}", err);
            return None;
        }
    };

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

        // Extract the creation date from the filename; one could use the creation date of the file, but this way we can be really sure
        let creation_date = path
            .file_name()
            .map(|name| name.to_str().unwrap_or("")) // convert OsStr into normal str
            .map(|name| name.strip_suffix(".zip").unwrap_or(""))
            .map(|name| {
                name.strip_prefix(&(profile_config.get_uuid().as_hyphenated().to_string() + "_"))
                    .unwrap_or("")
            })
            .unwrap_or("");
        // actually parse str into NaiveDateTime
        let creation_date = match NaiveDateTime::parse_from_str(creation_date, "%Y-%m-%d_%H-%M") {
            Ok(date_time) => date_time,
            Err(e) => {
                warn!("Couldn't parse date {:?} because {:?}", creation_date, e);
                continue;
            }
        };

        // update current best
        if creation_date <= timestamp
            && best_backup.as_ref().map_or(true, |&(curr_best_date, _)| curr_best_date < creation_date)
        {   
            debug!("Update best_backup to {:?}", creation_date);
            best_backup = Some((creation_date, path));
        }
    }

    best_backup.and_then(|(_, path)| Some(path))
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

    for i in 0..zip.len() {
        let mut file = match zip.by_index(i) {
            Ok(file) => file,
            Err(e) => {
                error!("Error extracting file: {:?}", e);
                return;
            }
        };
        let filepath = PathBuf::from(file.name());

        if let Some(p) = filepath.parent() {
            if !p.exists() {
                if let Err(e) = fs::create_dir_all(p) {
                    error!("Couldn't create dir {:?} because {:?}", filepath.parent(), e);
                    return;
                }
            }
        }
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