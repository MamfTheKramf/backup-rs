//! Contains functions and methods for loading and managing configs.

use std::{ffi::OsStr, fs, io::Error, path::PathBuf};

use config::{general_config::GeneralConfig, profile_config::ProfileConfig};

const GENERAL_CONFIG_PATH: &'static str = "./general_config.json";

/// Loads the general config file.
/// Either from the provided `path` or from [GENERAL_CONFIG_PATH] if `path` is [None]
///
/// # Returns
/// [Result::Ok] containing the config if it could be loaded.
/// [Result::Err] containing a [String] describing the issue if some occured.
pub fn load_general_config(path: Option<&str>) -> Result<GeneralConfig, String> {
    let path = PathBuf::from(path.unwrap_or(GENERAL_CONFIG_PATH));

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

    match GeneralConfig::read(&path) {
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

/// Trait for matching [ProfileConfig]s.
/// For a [ProfileConfig] to match, either its `name` must match what is returned by [ProfileSpecifier::name],
/// or its `uuid` has to match what is returned by [ProfileSpecifier::uuid].
pub trait ProfileSpecifier {
    fn name(&self) -> Option<&str>;

    fn uuid(&self) -> Option<&str>;
}

/// Checks if the given [ProfileConfig] either has the `name` or the `uuid` specified in `args`.
///
/// # Returns
/// Given [ProfileConfig] if it does.
/// [None] else.
fn matches_specifier<T: ProfileSpecifier>(
    profile_conf: Option<ProfileConfig>,
    specifier: &T,
) -> Option<ProfileConfig> {
    let profile_conf = profile_conf?;
    let name_match;
    match specifier.name() {
        Some(name) => name_match = &profile_conf.name == name,
        None => name_match = true,
    }
    let uuid_match;
    match specifier.uuid() {
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
pub fn soft_load_profile_configs<T: ProfileSpecifier>(
    general_config: &GeneralConfig,
    specifier: &T,
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
            matches_specifier(ProfileConfig::load(&entry.ok()?.path()).ok(), specifier)
        })
        .collect())
}

/// Loads profile configs from the specification in the provided [GeneralConfig].
///
/// Only returns those [ProfileConfig]s that match the `name` or the `uuid` given in `cli_args`. If both are [None], all found [ProfileConfig]s are returned.
///
/// Is a hard fail in such a way that it will return an [Err] as soon as one [ProfileConfig] fails.
/// If you want a soft fail that simply skips the broken files and continues to try the others, use [soft_load_profile_configs].
#[allow(dead_code)]
pub fn hard_load_profile_configs<T: ProfileSpecifier>(
    general_config: &GeneralConfig,
    specifier: &T,
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
            .filter_map(|conf| matches_specifier(Some(conf), specifier))
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

    struct MockProfileSpecifier {
        name: Option<String>,
        uuid: Option<String>,
    }

    impl MockProfileSpecifier {
        pub fn new(name: Option<String>, uuid: Option<String>) -> MockProfileSpecifier {
            MockProfileSpecifier {
                name,
                uuid,
            }
        }

        pub fn with_name(name: Option<String>) -> MockProfileSpecifier {
            Self::new(name, None)
        }

        pub fn with_uuid(uuid: Option<String>) -> MockProfileSpecifier {
            Self::new(None, uuid)
        }

        pub fn with_none() -> MockProfileSpecifier {
            Self::new(None, None)
        }
    }

    impl ProfileSpecifier for MockProfileSpecifier {
        fn name(&self) -> Option<&str> {
            self.name.as_ref().map(|name| name.as_str())
        }

        fn uuid(&self) -> Option<&str> {
            self.uuid.as_ref().map(|uuid| uuid.as_str())
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
            let specifier = MockProfileSpecifier::with_none();
            assert!(soft_load_profile_configs(&config, &specifier).is_err());

            let path = PathBuf::from("./Cargo.toml");
            let config = GeneralConfig {
                profile_configs: path,
            };
            assert!(soft_load_profile_configs(&config, &specifier).is_err());
        }

        #[test]
        fn valid_configs() {
            let path = PathBuf::from("./test_dir/valid_profile_configs");
            let config = GeneralConfig {
                profile_configs: path,
            };
            let specifier = MockProfileSpecifier::with_none();
            let configs = soft_load_profile_configs(&config, &specifier).unwrap();

            assert_eq!(configs.len(), 5);
        }

        #[test]
        fn valid_configs_name() {
            let path = PathBuf::from("./test_dir/valid_profile_configs");
            let config = GeneralConfig {
                profile_configs: path,
            };
            let specifier = MockProfileSpecifier::with_name(Some(String::from("Hutzi")));
            let configs = soft_load_profile_configs(&config, &specifier).unwrap();

            assert_eq!(configs.len(), 3);
        }

        #[test]
        fn valid_configs_uuid() {
            let path = PathBuf::from("./test_dir/valid_profile_configs");
            let config = GeneralConfig {
                profile_configs: path,
            };
            let specifier = MockProfileSpecifier::with_uuid(Some(String::from("6f41ec8a-da22-4e77-9a9c-50d18556375f")));
            let configs = soft_load_profile_configs(&config, &specifier).unwrap();

            assert_eq!(configs.len(), 1);
        }

        #[test]
        fn invalid_configs() {
            let path = PathBuf::from("./test_dir/invalid_profile_configs");
            let config = GeneralConfig {
                profile_configs: path,
            };
            let specifier = MockProfileSpecifier::with_none();
            let configs = soft_load_profile_configs(&config, &specifier).unwrap();

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
            let specifier = MockProfileSpecifier::with_none();
            assert!(hard_load_profile_configs(&config, &specifier).is_err());

            let path = PathBuf::from("./Cargo.toml");
            let config = GeneralConfig {
                profile_configs: path,
            };
            assert!(hard_load_profile_configs(&config, &specifier).is_err());
        }

        #[test]
        fn valid_configs() {
            let path = PathBuf::from("./test_dir/valid_profile_configs");
            let config = GeneralConfig {
                profile_configs: path,
            };
            let specifer = MockProfileSpecifier::with_none();
            let configs = hard_load_profile_configs(&config, &specifer).unwrap();

            assert_eq!(configs.len(), 5);
        }

        #[test]
        fn valid_configs_name() {
            let path = PathBuf::from("./test_dir/valid_profile_configs");
            let config = GeneralConfig {
                profile_configs: path,
            };
            let specifier = MockProfileSpecifier::with_name(Some(String::from("Hutzi")));
            let configs = hard_load_profile_configs(&config, &specifier).unwrap();

            assert_eq!(configs.len(), 3);
        }

        #[test]
        fn valid_configs_uuid() {
            let path = PathBuf::from("./test_dir/valid_profile_configs");
            let config = GeneralConfig {
                profile_configs: path,
            };
            let specifier = MockProfileSpecifier::with_uuid(Some(String::from("6f41ec8a-da22-4e77-9a9c-50d18556375f")));
            let configs = hard_load_profile_configs(&config, &specifier).unwrap();

            assert_eq!(configs.len(), 1);
        }

        #[test]
        fn invalid_configs() {
            let path = PathBuf::from("./test_dir/invalid_profile_configs");
            let config = GeneralConfig {
                profile_configs: path,
            };
            let specifier = MockProfileSpecifier::with_none();
            let configs = hard_load_profile_configs(&config, &specifier);

            assert!(configs.is_err());
        }
    }
}
