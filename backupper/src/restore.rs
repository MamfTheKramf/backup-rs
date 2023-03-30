//! Contains function for restoring a backup.

use std::{fs::{self, File}, path::PathBuf, io};

use chrono::NaiveDateTime;
use config::profile_config::ProfileConfig;
use zip::ZipArchive;

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

    let best_backup = best_backup.unwrap();

    let file = match File::open(&best_backup.1) {
        Ok(file) => file,
        Err(e) => {
            println!("Error opening file {:?}: {:?}", best_backup.1, e);
            return;
        }
    };

    let mut zip = match ZipArchive::new(file) {
        Ok(file) => file,
        Err(e) => {
            println!("Couldn't create archive because {:?}", e);
            return;
        }
    };

    for i in 0..zip.len() {
        let mut file = match zip.by_index(i) {
            Ok(file) => file,
            Err(e) => {
                println!("Error extracting file: {:?}", e);
                return;
            }
        };
        let filepath = PathBuf::from(file.name());

        if let Some(p) = filepath.parent() {
            if !p.exists() {
                if let Err(e) = fs::create_dir_all(p) {
                    println!("Couldn't create dir {:?} because {:?}", filepath.parent(), e);
                    return;
                }
            }
        }
        let mut outfile = match fs::File::create(&filepath) {
            Ok(outfile) => outfile,
            Err(e) => {
                println!("Couldn't create outfile {:?} because {:?}", filepath, e);
                return;
            }
        };
        if let Err(e) = io::copy(&mut file, &mut outfile) {
            println!("Couldn't copy to outfile because {:?}", e);
            return;
        }
    }
}
