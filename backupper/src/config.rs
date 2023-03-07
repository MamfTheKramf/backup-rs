//! Contains functions and methods for loading and managing configs.

use std::{ffi::OsStr, fs, io::Error, path::PathBuf};

use config::{general_config::GeneralConfig, profile_config::ProfileConfig};

use crate::cli_args::Args;

const GENERAL_CONFIG_PATH: &'static str = "./test_dir/general_config.json";

/// Loads the general Config file.
///
/// # Returns
/// [Result::Ok] containing the config if it could be loaded.
/// [Result::Err] containing a [String] describing the issue if some occured.
pub fn load_general_config() -> Result<GeneralConfig, String> {
    let path = PathBuf::from(GENERAL_CONFIG_PATH);

    match path.try_exists() {
        Ok(val) => {
            if !val {
                return Err(String::from("General config file doesn't exist!"));
            }
        }
        Err(e) => return Err(e.to_string()),
    }

    if !path.is_file() {
        return Err(String::from(
            "Path to general config file doesn't point to a file!",
        ));
    }

    match GeneralConfig::read(path) {
        Err(e) => Err(e.to_string()),
        Ok(config) => Ok(config),
    }
}

/// Checks if the `path` points to a valid directory.
///
/// # Retuns
/// [Ok] if `path` points to a valid directory.
/// [Err] containing a [String] describing the issue if not
fn valid_dir_path(path: &PathBuf) -> Result<(), String> {
    match path.try_exists() {
        Ok(val) => {
            if !val {
                return Err(format!("Directory {} doesn't exist!", path.display()));
            }
        }
        Err(e) => return Err(e.to_string()),
    }

    if !path.is_dir() {
        return Err(format!(
            "Path {} doesn't point to directory!",
            path.display()
        ));
    }

    Ok(())
}

/// Checks if the given [ProfileConfig] either has the `name` or the `uuid` specified in `args`.
///
/// # Returns
/// Given [ProfileConfig] if it does.
/// [None] else.
fn matches_args(profile_conf: Option<ProfileConfig>, args: &Args) -> Option<ProfileConfig> {
    let profile_conf = profile_conf?;
    let name_match;
    match &args.name {
        Some(name) => name_match = &profile_conf.name == name,
        None => name_match = true,
    }
    let uuid_match;
    match &args.uuid {
        Some(uuid) => {
            if let Ok(uuid) = uuid::Uuid::parse_str(uuid) {
                uuid_match = profile_conf.get_uuid() == &uuid;
            } else {
                uuid_match = true;
            }
        }
        None => uuid_match = true,
    };

    if name_match && uuid_match {
        Some(profile_conf)
    } else {
        None
    }
}

/// Loads profile configs from the specification in the provided [GeneralConfig].
/// 
/// Only returns those [ProfileConfig]s that match the `name` or the `uuid` given in `cli_args`. If both are [None], all found [ProfileConfig]s are returned.
/// 
/// Is a soft fail in such a way that it will skip those config files it can't read / parse.
/// If you want a hard fail, that will return an [Err] as soon as one [ProfileConfig] fails, use [hard_load_profile_configs].
pub fn soft_load_profile_configs(
    general_config: &GeneralConfig,
    cli_args: &Args,
) -> Result<Vec<ProfileConfig>, String> {
    let path = &general_config.profile_configs;

    valid_dir_path(path)?;

    let read_dir = fs::read_dir(path);
    if read_dir.is_err() {
        return Err(read_dir
            .err()
            .expect("is_err was true, but unwrapping err still failed!")
            .to_string());
    }

    Ok(read_dir
        .ok()
        .expect("This must be an ok!")
        .filter_map(|entry| {
            // filter out bad entries or the ones that are no JSONs
            if entry.is_err() {
                return None;
            }
            let dir_entry = entry.as_ref().ok()?;
            if dir_entry.path().extension().unwrap_or_default() != OsStr::new("json") {
                return None;
            }

            // load the profile
            matches_args(ProfileConfig::load(&entry.ok()?.path()).ok(), cli_args)
        })
        .collect())
}

/// Loads profile configs from the specification in the provided [GeneralConfig].
/// 
/// Only returns those [ProfileConfig]s that match the `name` or the `uuid` given in `cli_args`. If both are [None], all found [ProfileConfig]s are returned.
/// 
/// Is a hard fail in such a way that it will return an [Err] as soon as one [ProfileConfig] fails.
/// If you want a soft fail that simply skips the broken files and continues to try the others, use [soft_load_profile_configs].
pub fn hard_load_profile_configs(
    general_config: &GeneralConfig,
    cli_args: &Args
) -> Result<Vec<ProfileConfig>, String> {
    let path = &general_config.profile_configs;

    valid_dir_path(path)?;

    let read_dir = fs::read_dir(path);
    if read_dir.is_err() {
        return Err(read_dir
            .err()
            .expect("is_err was true, but unwrapping err still failed!")
            .to_string());
    }

    let collect_res: Result<Vec<ProfileConfig>, Error> = read_dir
        .ok()
        .expect("This must be an ok!")
        .filter_map(|entry| {
            // filter out bad entries or the ones that are no JSONs
            if entry.is_err() {
                return None;
            }
            let dir_entry = entry.as_ref().ok()?;
            if dir_entry.path().extension().unwrap_or_default() != OsStr::new("json") {
                return None;
            }

            entry.ok()
        })
        .map(|entry| ProfileConfig::load(&entry.path()))
        .collect();

    match collect_res {
        Ok(vec) => Ok(vec
            .into_iter()
            .filter_map(|conf| matches_args(Some(conf), cli_args))
            .collect()),
        Err(e) => Err(e.to_string()),
    }
}

#[cfg(test)]
mod config_tests {
    use super::*;

    mod valid_dir_path_tests {
        use super::*;

        #[test]
        fn valid_dir_relative() {
            let path = PathBuf::from("./test_dir/");
            assert!(valid_dir_path(&path).is_ok());

            let path = PathBuf::from("./test_dir");
            assert!(valid_dir_path(&path).is_ok());
        }

        #[test]
        fn valid_dir_absolute() {
            let path = PathBuf::from("C:/");
            assert!(valid_dir_path(&path).is_ok());

            let path = PathBuf::from("c:/");
            assert!(valid_dir_path(&path).is_ok());
        }

        #[test]
        fn doesnt_exist() {
            let path = PathBuf::from("./hfgjdish/fhdjis");
            assert!(valid_dir_path(&path).is_err());
        }

        #[test]
        fn is_file() {
            let path = PathBuf::from("./Cargo.toml");
            assert!(valid_dir_path(&path).is_err());
        }
    }

    mod soft_load_profile_configs_tests {
        use super::*;

        #[test]
        fn invalid_path() {
            let path = PathBuf::from("./hfgjdish/fhdjis");
            let config = GeneralConfig {
                profile_configs: path,
            };
            let args = Args{ name: None, uuid: None };
            assert!(soft_load_profile_configs(&config, &args).is_err());

            let path = PathBuf::from("./Cargo.toml");
            let config = GeneralConfig {
                profile_configs: path,
            };
            assert!(soft_load_profile_configs(&config, &args).is_err());
        }

        #[test]
        fn valid_configs() {
            let path = PathBuf::from("./test_dir/valid_profile_configs");
            let config = GeneralConfig {
                profile_configs: path,
            };
            let args = Args{ name: None, uuid: None };
            let configs = soft_load_profile_configs(&config, &args).unwrap();

            assert_eq!(configs.len(), 5);
        }

        #[test]
        fn valid_configs_name() {
            let path = PathBuf::from("./test_dir/valid_profile_configs");
            let config = GeneralConfig {
                profile_configs: path,
            };
            let args = Args{ name: Some(String::from("Hutzi")), uuid: None };
            let configs = soft_load_profile_configs(&config, &args).unwrap();

            assert_eq!(configs.len(), 3);
        }

        #[test]
        fn valid_configs_uuid() {
            let path = PathBuf::from("./test_dir/valid_profile_configs");
            let config = GeneralConfig {
                profile_configs: path,
            };
            let args = Args{ name: None, uuid: Some(String::from("6f41ec8a-da22-4e77-9a9c-50d18556375f")) };
            let configs = soft_load_profile_configs(&config, &args).unwrap();

            assert_eq!(configs.len(), 1);
        }

        #[test]
        fn invalid_configs() {
            let path = PathBuf::from("./test_dir/invalid_profile_configs");
            let config = GeneralConfig {
                profile_configs: path,
            };
            let args = Args{ name: None, uuid: None };
            let configs = soft_load_profile_configs(&config, &args).unwrap();

            assert_eq!(configs.len(), 2);
        }
    }

    mod hard_load_profile_configs_tests {
        use super::*;

        #[test]
        fn invalid_path() {
            let path = PathBuf::from("./hfgjdish/fhdjis");
            let config = GeneralConfig {
                profile_configs: path,
            };
            let args = Args{ name: None, uuid: None };
            assert!(hard_load_profile_configs(&config, &args).is_err());

            let path = PathBuf::from("./Cargo.toml");
            let config = GeneralConfig {
                profile_configs: path,
            };
            assert!(hard_load_profile_configs(&config, &args).is_err());
        }

        #[test]
        fn valid_configs() {
            let path = PathBuf::from("./test_dir/valid_profile_configs");
            let config = GeneralConfig {
                profile_configs: path,
            };
            let args = Args{ name: None, uuid: None };
            let configs = hard_load_profile_configs(&config, &args).unwrap();

            assert_eq!(configs.len(), 5);
        }

        #[test]
        fn valid_configs_name() {
            let path = PathBuf::from("./test_dir/valid_profile_configs");
            let config = GeneralConfig {
                profile_configs: path,
            };
            let args = Args{ name: Some(String::from("Hutzi")), uuid: None };
            let configs = hard_load_profile_configs(&config, &args).unwrap();

            assert_eq!(configs.len(), 3);
        }

        #[test]
        fn valid_configs_uuid() {
            let path = PathBuf::from("./test_dir/valid_profile_configs");
            let config = GeneralConfig {
                profile_configs: path,
            };
            let args = Args{ name: None, uuid: Some(String::from("6f41ec8a-da22-4e77-9a9c-50d18556375f")) };
            let configs = hard_load_profile_configs(&config, &args).unwrap();

            assert_eq!(configs.len(), 1);
        }

        #[test]
        fn invalid_configs() {
            let path = PathBuf::from("./test_dir/invalid_profile_configs");
            let config = GeneralConfig {
                profile_configs: path,
            };
            let args = Args{ name: None, uuid: None };
            let configs = hard_load_profile_configs(&config, &args);

            assert!(configs.is_err());
        }
    }
}
