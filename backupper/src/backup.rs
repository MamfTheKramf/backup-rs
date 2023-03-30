//! Contains functions for actually creating a backup file.

use std::{
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    path::PathBuf,
};

use chrono::offset;
use config::{
    general_config::GeneralConfig, interval::DateTimeMatch, profile_config::ProfileConfig,
};
use zip::{write::FileOptions, ZipWriter};

use crate::{
    cli_args::Args,
    dialog::{retry_dialog, DialogResult, RETRY},
    scheduler::schedule_backup, common::is_target_dir_available,
};

/// Handles the provided [ProfileConfig].
/// Checks when the next update is due and either schedules another call to this executable or performs the update.
/// Might also open an alert window, if necessary.
///
/// Also stores the updated version of profile config.
pub fn handle_profile(
    profile_config: &mut ProfileConfig,
    general_config: &GeneralConfig,
    args: &Args,
) {
    let (update_next_backup, do_perform_backup) = is_scheduled(profile_config, args.force);

    // actually perform backup
    if do_perform_backup {
        if let Err(msg) = perform_backup(profile_config, args) {
            println!("{}", msg);
        }
    }

    // update next_backup if needed
    if update_next_backup {
        let next_scheduled =
            profile_config.get_next_scheduled(Some(offset::Local::now().naive_local()));
        profile_config.next_backup = next_scheduled;
    }

    if let Err(err) = profile_config.store(&general_config.profile_configs) {
        println!(
            "Couldn't store profile_config {:?}.\nGot error: {:?}",
            profile_config, err
        );
    }

    if let Err(msg) = schedule_backup(
        profile_config.get_uuid().clone(),
        profile_config.next_backup,
    ) {
        println!("Couldn't set up next backup.\nGot error: {:?}", msg);
    }
}

/// Checks if a backup actually has to be performed or if only the `next_backup` field of the profived [ProfileConfig] has to be updated, or none of both.
///
/// # Returns
/// 2 [bool]eans: First one specifies whether the `next_backup` field has to be updated, second one specifies whether a backup shall be performed.
fn is_scheduled(profile_config: &ProfileConfig, forced: bool) -> (bool, bool) {
    if forced {
        return (true, true);
    }

    let now = offset::Local::now().naive_local();
    let next_backup = profile_config.next_backup;

    // next backup isn't even scheduled for now
    if now < next_backup {
        return (false, false);
    }

    let scheduled = profile_config.get_next_scheduled(None);

    // check if we had a match
    let next_backup_matches =
        profile_config.interval.matches_datetime(next_backup) == DateTimeMatch::Ok;
    let scheduled_matches =
        profile_config.interval.matches_datetime(scheduled) == DateTimeMatch::Ok;
    let skipped_scheduled = scheduled <= now;

    let skipped_match = next_backup_matches || skipped_scheduled && scheduled_matches;

    (true, skipped_match)
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
    let mut choice = DialogResult(RETRY);
    while !is_target_dir_available(&profile_config.target_dir, true) && choice == DialogResult(RETRY) {
        let msg = format!("Das Verzeichnis {:?} scheint nicht verfügpar zu sein.\nBitte schließe die externe Festplatte an und versuche es erneut.", profile_config.target_dir);
        let title = "Zielfverzeichnis nicht verfügbar.";
        choice = retry_dialog(title, &msg);
    }
    if choice != DialogResult(RETRY) {
        return Err(format!(
            "Directory {:?} isn't available and retry was cancled",
            profile_config.target_dir
        ));
    }

    // now the target dir should be available

    // set up zip archive
    let filename = profile_config.get_uuid().as_hyphenated().to_string()
        + "_"
        + &chrono::offset::Local::now()
            .naive_local()
            .format("%Y-%m-%d_%H-%M")
            .to_string()
        + ".zip";
    let path = profile_config.target_dir.as_path().join(filename);
    let file = match OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path.clone())
    {
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

/// Walks through the given `dir` and adds all files not excluded to the zip-archive.
fn add_directory(
    zip: &mut ZipWriter<File>,
    dir: &PathBuf,
    profile_config: &ProfileConfig,
    args: &Args,
) -> Result<(), String> {
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
                if args.verbose {
                    println!("{}", msg);
                }
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
fn add_file(
    zip: &mut ZipWriter<File>,
    file: &PathBuf,
    profile_config: &ProfileConfig,
    args: &Args,
) -> Result<(), String> {
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
        return Ok(());
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
        Err(err) => {
            return Err(format!(
                "Couldn't open file {:?} because of {:?}",
                path, err
            ))
        }
    };

    if args.verbose {
        println!("Store {:?}", path);
    }

    let name = String::from(path.to_str().unwrap_or(""));
    if let Err(err) = zip.start_file(name, FileOptions::default()) {
        return Err(format!(
            "Couldn't start file {:?} because of {:?}",
            path, err
        ));
    }

    const N: usize = 0x2000;
    let mut buf = [0u8; N];
    loop {
        let read_bytes = match Read::by_ref(&mut file).take(N as u64).read(&mut buf) {
            Ok(n) => n,
            Err(err) => {
                return Err(format!(
                    "Couldn't read from {:?} because of {:?}",
                    path, err
                ))
            }
        };

        if read_bytes == 0 {
            break;
        }

        if let Err(err) = zip.write_all(&mut buf[..read_bytes]) {
            return Err(format!(
                "Couldn't write {:?} to archive because of {:?}",
                path, err
            ));
        }
    }

    if args.verbose {
        println!("Successfully added {:?} to archive.", path);
    }
    Ok(())
}

#[cfg(test)]
mod backup_tests {
    use config::interval::Interval;

    use super::*;

    fn dummy_profile_config(interval: Interval) -> ProfileConfig {
        ProfileConfig::new(
            String::from(""),
            PathBuf::new(),
            vec![],
            vec![],
            vec![],
            vec![],
            interval,
        )
    }

    mod is_scheduled_tests {
        use chrono::{Duration, NaiveDateTime, Timelike};
        use config::interval::{IntervalBuilder, Month};

        use super::*;

        #[test]
        fn forced_not_scheduled_yet() {
            let mut profile_config =
                dummy_profile_config(IntervalBuilder::default().build().unwrap());
            profile_config.next_backup =
                NaiveDateTime::parse_from_str("3000-12-31 23:59", "%Y-%m-%d %H:%M").unwrap();

            assert_eq!(is_scheduled(&profile_config, true), (true, true));
        }

        #[test]
        fn forced_missed_next_backup() {
            let mut profile_config =
                dummy_profile_config(IntervalBuilder::default().build().unwrap());
            profile_config.next_backup =
                NaiveDateTime::parse_from_str("2000-12-31 23:59", "%Y-%m-%d %H:%M").unwrap();

            assert_eq!(is_scheduled(&profile_config, true), (true, true));
        }

        #[test]
        fn not_scheduled_yet() {
            let mut profile_config =
                dummy_profile_config(IntervalBuilder::default().build().unwrap());
            profile_config.next_backup =
                NaiveDateTime::parse_from_str("3000-12-31 23:59", "%Y-%m-%d %H:%M").unwrap();

            assert_eq!(is_scheduled(&profile_config, false), (false, false));
        }

        #[test]
        fn missed_only_next_backup_no_match() {
            let mut profile_config = dummy_profile_config(
                IntervalBuilder::default()
                    .minutes(config::interval::SpecifierKind::None)
                    .build()
                    .unwrap(),
            );
            profile_config.next_backup = chrono::Local::now()
                .naive_local()
                .checked_sub_signed(Duration::hours(1))
                .unwrap();

            assert_eq!(is_scheduled(&profile_config, false), (true, false));
        }

        #[test]
        fn missed_only_next_backup_match() {
            let mut profile_config = dummy_profile_config(
                IntervalBuilder::default()
                    .minutes(config::interval::SpecifierKind::First)
                    .hours(config::interval::SpecifierKind::First)
                    .build()
                    .unwrap(),
            );

            let now = chrono::Local::now().naive_local();
            let morning = now
                .with_nanosecond(0)
                .unwrap()
                .with_second(0)
                .unwrap()
                .with_minute(0)
                .unwrap()
                .with_hour(0)
                .unwrap();
            profile_config.next_backup = morning;
            assert_eq!(is_scheduled(&profile_config, false), (true, true));
        }

        #[test]
        fn missed_both_both_match() {
            // matches every whole hour
            let mut profile_config = dummy_profile_config(
                IntervalBuilder::default()
                    .minutes(config::interval::SpecifierKind::First)
                    .build()
                    .unwrap(),
            );

            let five_hours_ago = chrono::Local::now()
            .naive_local()
            .checked_sub_signed(Duration::hours(5))
            .unwrap()
            .with_nanosecond(0).unwrap()
            .with_second(0).unwrap()
            .with_minute(0).unwrap();
            profile_config.next_backup = five_hours_ago;

            assert_eq!(is_scheduled(&profile_config, false), (true, true));
        }

        #[test]
        fn missed_both_only_scheduled_matches() {
            // matches every whole hour
            let mut profile_config = dummy_profile_config(
                IntervalBuilder::default()
                    .minutes(config::interval::SpecifierKind::First)
                    .build()
                    .unwrap(),
            );


            profile_config.next_backup = chrono::Local::now()
                .naive_local()
                .checked_sub_signed(Duration::hours(5))
                .unwrap();

            assert_eq!(is_scheduled(&profile_config, false), (true, true));
        }

        #[test]
        fn missed_both_only_next_backup_matches() {
            // matches every feb 29th
            let mut profile_config = dummy_profile_config(
                IntervalBuilder::default()
                    .minutes(config::interval::SpecifierKind::First)
                    .hours(config::interval::SpecifierKind::First)
                    .monthdays(config::interval::SpecifierKind::Nth(28)) // 28 because we start counting with 0
                    .months(config::interval::SpecifierKind::Nth(Month::February().into()))
                    .build()
                    .unwrap(),
            );

            let feb_29th_2004 = NaiveDateTime::parse_from_str("2004-02-29 00:00", "%Y-%m-%d %H:%M").unwrap();
            profile_config.next_backup = feb_29th_2004;

            assert_eq!(is_scheduled(&profile_config, false), (true, true));
        }

        #[test]
        fn missed_both_only_none_matches() {
            // matches every feb 29th
            let mut profile_config = dummy_profile_config(
                IntervalBuilder::default()
                    .minutes(config::interval::SpecifierKind::First)
                    .hours(config::interval::SpecifierKind::First)
                    .monthdays(config::interval::SpecifierKind::Nth(28))
                    .months(config::interval::SpecifierKind::Nth(Month::February().into()))
                    .build()
                    .unwrap(),
            );

            let apr_1st_2004 = NaiveDateTime::parse_from_str("2004-03-1 00:00", "%Y-%m-%d %H:%M").unwrap();
            profile_config.next_backup = apr_1st_2004;

            assert_eq!(is_scheduled(&profile_config, false), (true, false));
        }
    }
}
