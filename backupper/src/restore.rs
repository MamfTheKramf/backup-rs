//! Contains function for restoring a backup.

use std::fs;

use chrono::NaiveDateTime;
use config::profile_config::ProfileConfig;

use crate::{
    cli_args::Args,
    common::is_target_dir_available,
    dialog::{retry_dialog, DialogResult, RETRY},
};

/// Restores the files from the latest backup of the provided [ProfileConfig] that is older than the given `timestamp`.
///
/// If there is no such backup, nothing happens.
pub fn restore(profile_config: &ProfileConfig, timestamp: NaiveDateTime, args: &Args) {
    // make sure, directory is available
    let mut choice = DialogResult(RETRY);
    while !is_target_dir_available(&profile_config.target_dir, false)
        && choice == DialogResult(RETRY)
    {
        let msg = format!("Das Verzeichnis mit den Backups {:?} scheint nicht verfügpar zu sein.\nBitte schließe die externe Festplatte an und versuche es erneut.", profile_config.target_dir);
        let title = "Backupverzeichnis nicht verfügbar.";
        choice = retry_dialog(title, &msg);
    }
    if choice != DialogResult(RETRY) {
        if args.verbose {
            println!(
                "Directory {:?} isn't available and retry was cancled",
                profile_config.target_dir
            )
        }
        return;
    }

    // find matching backup
    let entries = match fs::read_dir(&profile_config.target_dir) {
        Ok(entries) => entries,
        Err(err) => {
            println!("Error reading dir: {:?}", err);
            return;
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

        let creation_date = path
            .file_name()
            .map(|name| name.to_str().unwrap_or(""))
            .map(|name| name.strip_suffix(".zip").unwrap_or(""))
            .map(|name| {
                name.strip_prefix(&(profile_config.get_uuid().as_hyphenated().to_string() + "_"))
                    .unwrap_or("")
            })
            .unwrap_or("");
        let creation_date = match NaiveDateTime::parse_from_str(creation_date, "%Y-%m-%d_%H-%M") {
            Ok(date_time) => date_time,
            Err(e) => {
                println!("Couldn't parse date {:?} because {:?}", creation_date, e);
                continue;
            }
        };

        // update current best
        if creation_date <= timestamp
            && best_backup.as_ref().map_or(true, |&(curr_best_date, _)| curr_best_date < creation_date)
        {   
            println!("Update best_backup to {:?}", creation_date);
            best_backup = Some((creation_date, path));
        }
    }
    println!("Found best: {:?}", best_backup);
    if best_backup.is_none() {
        return;
    }
}
