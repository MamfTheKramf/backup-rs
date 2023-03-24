//! Contains functions for actually creating a backup file.

use std::{fs::{self, OpenOptions, File}, path::PathBuf, io::{Read, Write}};

use chrono::offset;
use config::{profile_config::ProfileConfig, interval::DateTimeMatch, general_config::GeneralConfig};
use windows::Win32::UI::WindowsAndMessaging::{MB_ICONWARNING, MESSAGEBOX_RESULT};
use zip::{ZipWriter, write::FileOptions};


use crate::{cli_args::Args, dialog::retry_dialog, scheduler::schedule_backup};

/// Handles the provided [ProfileConfig].
/// Checks when the next update is due and either schedules another call to this executable or performs the update.
/// Might also open an alert window, if necessary.
/// 
/// Also stores the updated version of profile config.
pub fn handle_profile(profile_config: &mut ProfileConfig, general_config: &GeneralConfig, args: &Args) {
    let now = offset::Local::now().naive_local();
    let next_backup = *profile_config.next_backup();

    if !args.force && next_backup < now {
        println!("Next backup isn't planned yet. It's scheduled for {:?}", next_backup);
        return;
    }

    let scheduled = *profile_config.set_next_backup(None);

    if args.verbose {
        println!("Next backup was set to {}", next_backup.format("%Y-%m-%d %H:%M").to_string());
        println!("Next backup is scheduled for {}", scheduled.format("%Y-%m-%d %H:%M").to_string());
    }

    // check if we had a match
    let next_backup_matches = profile_config.interval.matches_datetime(next_backup) == DateTimeMatch::Ok;
    let skipped_next_backup = next_backup <= now;
    let scheduled_matches = profile_config.interval.matches_datetime(scheduled) == DateTimeMatch::Ok;
    let skipped_scheduled = scheduled <= now;
    let skipped_match = next_backup_matches && skipped_next_backup ||
        scheduled_matches && skipped_scheduled;

    if args.verbose {
        println!("next_backup_matches:\t{}", next_backup_matches);
        println!("skipped_next_backup:\t{}", skipped_next_backup);
        println!("scheduled_matches:\t{}", scheduled_matches);
        println!("skipped_scheduled:\t{}", skipped_scheduled);
        println!("skipped_match:\t{}", skipped_match);
    }

    // actually perform backup
    if skipped_match {
        if let Err(msg) = perform_backup(profile_config, args) {
            println!("{}", msg);
            return;
        }
    }

    // reset next_backup if needed
    if skipped_scheduled {
        profile_config.set_next_backup(Some(offset::Local::now().naive_local()));
    }

    if let Err(err) = profile_config.store(&general_config.profile_configs) {
        println!("Couldn't store profile_config {:?}.\nGot error: {:?}", profile_config, err);
        return;
    }

    if let Err(msg) = schedule_backup(profile_config.get_uuid().clone(), *profile_config.next_backup()) {
        println!("Couldn't set up next backup.\nGot error: {:?}", msg);
    }
}

/// Performs actual backup.
/// 
/// 1. If the target directory for the zip archive is accesible and opens retry dialog boxes until it is accesibly, or the backup is cancelled.
/// 2. Creates a file for the zip archive.
/// 3. Recursively goes through directories to include and adds each file, not matched by the excluded files to the archive
/// 4. Goes through the files to include and adds each file, not matched by the included dirs to the archive
/// 5. Stores zip an exits
fn perform_backup(profile_config: &ProfileConfig, args: &Args) -> std::result::Result<(), String> {
    // if target dir isn't available, open dialog
    let mut choice = MESSAGEBOX_RESULT(4);
    while !is_target_dir_available(profile_config) && choice == MESSAGEBOX_RESULT(4) {
        let msg = format!("Das Verzeichnis {:?} scheint nicht verfügpar zu sein.\nBitte schließe die externe Festplatte an und versuche es erneut.", profile_config.target_dir);
        let title = "Zielfverzeichnis nicht verfügbar.";
        choice = retry_dialog(title, &msg, MB_ICONWARNING);
    }
    if choice != MESSAGEBOX_RESULT(4) {
        return Err(format!("Directory {:?} isn't available and retry was cancled", profile_config.target_dir))
    }

    // now the target dir should be available
    
    // set up zip archive
    let filename = profile_config.get_uuid().as_hyphenated().to_string() + &chrono::offset::Local::now().naive_local().format("%Y-%m-%d_%H-%M").to_string() + ".zip";
    let path = profile_config.target_dir.as_path().join(filename);
    let file = match OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path.clone()) {
            Ok(file) => file,
            Err(err) => return Err(format!("Error creating file {:?}: {:?}", path, err)),
        };
    let mut zip = ZipWriter::new(file);
    
    // add all directories
    for dir in &profile_config.dirs_to_include {
        if let Err(msg) = add_directory(&mut zip, dir, profile_config, args) {
            if args.verbose {
                println!("Couldn't add dir {:?} because {:?}", dir, msg);
            }
        }
    }

    // add all files
    for file in &profile_config.files_to_include {
        if let Err(msg) = add_file(&mut zip, file, profile_config, args) {
            if args.verbose {
                println!("Couldn't add file {:?} because {:?}", file, msg);
            }
        }
    }

    if let Err(err) = zip.finish() {
        remove_archive(zip, path);
        return Err(format!("Couldn't finish archive because of {:?}", err));
    }

    if args.verbose {
        println!("Finished archive in {:?}", path);
    }
    Ok(())
}

/// Attempts to remove started zip-archive from filesystem.
/// You call this after an unrecoverable error occured, to clean up
#[allow(unused_must_use)]
fn remove_archive(mut zip: ZipWriter<File>, path: PathBuf) {
    zip.finish();
    fs::remove_file(path);
}

/// Checks if the target directory specified in [ProfileConfig] is writable or not.
fn is_target_dir_available(profile_config: &ProfileConfig) -> bool {
    match fs::metadata(profile_config.target_dir.clone()) {
        Err(_) => false,
        Ok(metadata) => metadata.is_dir() && !metadata.permissions().readonly()
    }
}

/// Walks through the given `dir` and adds all files not excluded to the zip-archive.
fn add_directory(zip: &mut ZipWriter<File>, dir: &PathBuf, profile_config: &ProfileConfig, args: &Args) -> Result<(), String> {
    if !dir.is_dir() {
        return Err(format!("{:?} is not a directory!", dir));
    }

    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(err) => return Err(format!("Error reading dir: {:?}", err)),
    };
    for entry in entries {
        if entry.is_err() {
            continue;
        }
        let entry = entry.unwrap();
        let path = entry.path();
        // skip excluded paths
        if profile_config.is_excluded(&path) {
            continue;
        }

        // go recursively into directories
        if path.is_dir() {
            if let Err(msg) = add_directory(zip, &path, profile_config, args) {
                if args.verbose { println!("{}", msg); }
            }
        }

        // actually store file
        if path.is_file() {
            match write_to_zip(&path, zip, args) {
                Ok(_) => (),
                Err(msg) => {
                    if args.verbose {
                        println!("{}", msg);
                    }
                }
            }
        }
    }
    Ok(())
}

/// Attempts to add file at the given path to the archive.
fn add_file(zip: &mut ZipWriter<File>, file: &PathBuf, profile_config: &ProfileConfig, args: &Args) -> Result<(), String> {
    if !file.is_file() {
        return Err(format!("{:?} is not a file!", file));
    }

    // included files shall overwrite the excluded files,
    // but it should not be added again, if it was already coverd by an included dir
    // --> if it is in an included dir, it might have not been added since is is also in an excluded dir
    //      --> add that file
    if profile_config.in_included_dirs(file) && !profile_config.is_excluded(file) {
        if args.verbose {
            println!("File {:?} is already covered by included dirs.", file);
        }
        return Ok(())
    }

    write_to_zip(file, zip, args)
}

/// Attempts to write the file at the specified `path` to the `zip`.
/// 
/// # Errors
/// Returns an [Err] describing the issue if something goes wrong
fn write_to_zip(path: &PathBuf, zip: &mut ZipWriter<File>, args: &Args) -> Result<(), String> {
    let mut file = match File::open(path) {
        Ok(file) => file,
        Err(err) => return Err(format!("Couldn't open file {:?} because of {:?}", path, err)),
    };

    if args.verbose {
        println!("Store {:?}", path);
    }
    
    let name = String::from(path.to_str().unwrap_or(""));
    if let Err(err) = zip.start_file(name, FileOptions::default()) {
        return Err(format!("Couldn't start file {:?} because of {:?}", path, err));
    }

    const N: usize = 0x2000;
    let mut buf = [0u8; N];
    loop {
        let read_bytes = match Read::by_ref(&mut file).take(N as u64).read(&mut buf) {
            Ok(n) => n,
            Err(err) => return Err(format!("Couldn't read from {:?} because of {:?}", path, err)),
        };

        if read_bytes == 0 { break; }

        if let Err(err) = zip.write_all(&mut buf[..read_bytes]) {
            return Err(format!("Couldn't write {:?} to archive because of {:?}", path, err));
        }
    }

    if args.verbose {
        println!("Successfully added {:?} to archive.", path);
    }
    Ok(())
}