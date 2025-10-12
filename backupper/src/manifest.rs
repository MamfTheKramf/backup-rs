//! Contains the manifest struct which contains meta data about the backup archive

use ::log::error;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Manifest {
    /// Semver version of the backupper that created this archive
    pub backupper_version: String,
    /// Unix timestamp (with respect to Utc timezone) indicating the time this backup was created
    pub created: i64,
}

impl Manifest {
    /// Creates a fresh manifest with the `created` timestamp pointing to now and the version set to the current backuppers version
    pub fn new() -> Manifest {
        Manifest {
            backupper_version: String::from(env!("CARGO_PKG_VERSION")),
            created: chrono::offset::Utc::now().timestamp(),
        }
    }

    /// Checks if this given manifest works with this version of the backupper
    ///
    /// # Returns
    /// `true` if it matches; `false` else
    pub fn matches(&self) -> bool {
        let binary_version = env!("CARGO_PKG_VERSION");

        let binary_parts = parse_version(binary_version);
        if binary_parts.len() != 3 {
            error!(
                "Cannot parse binary version {} -> assume that manifest doesn't match",
                binary_version
            );
            return false;
        }

        let manifest_parts = parse_version(&self.backupper_version);
        if manifest_parts.len() != 3 {
            error!("Cannot parse backupper version of manifest {} -> assume that manifest doesn't match", self.backupper_version);
            return false;
        }

        manifest_parts[0] == binary_parts[0] && manifest_parts[1] == binary_parts[1]
    }
}

/// Simple parser for semver version strings
/// 
/// # Resturns
/// [Vec] of [u32]s where every part between dots that couldn't be parsed is omitted
fn parse_version(version_str: &str) -> Vec<u32> {
    version_str
        .split(".")
        .filter_map(|part| part.parse::<u32>().ok())
        .collect()
}

#[cfg(test)]
mod manifest_tests {
    use super::*;

    #[test]
    fn test_matches_same_version() {
        let manifest = Manifest::new();
        assert!(manifest.matches());
    }

    #[test]
    fn test_matches_different_patch() {
        let mut manifest = Manifest::new();
        let mut version_parts = manifest
            .backupper_version
            .split(".")
            .map(|part| part.parse::<u32>().expect("Version part was no number"))
            .collect::<Vec<u32>>();
        version_parts[2] += 1;
        manifest.backupper_version = version_parts
            .iter()
            .map(|part| part.to_string())
            .collect::<Vec<String>>()
            .join(".");
        assert!(manifest.matches());
    }

    #[test]
    fn test_not_matches_different_minor() {
        let mut manifest = Manifest::new();
        let mut version_parts = manifest
            .backupper_version
            .split(".")
            .map(|part| part.parse::<u32>().expect("Version part was no number"))
            .collect::<Vec<u32>>();
        version_parts[1] += 1;
        manifest.backupper_version = version_parts
            .iter()
            .map(|part| part.to_string())
            .collect::<Vec<String>>()
            .join(".");
        assert!(!manifest.matches());
    }

    #[test]
    fn test_not_matches_different_major() {
        let mut manifest = Manifest::new();
        let mut version_parts = manifest
            .backupper_version
            .split(".")
            .map(|part| part.parse::<u32>().expect("Version part was no number"))
            .collect::<Vec<u32>>();
        version_parts[0] += 1;
        manifest.backupper_version = version_parts
            .iter()
            .map(|part| part.to_string())
            .collect::<Vec<String>>()
            .join(".");
        assert!(!manifest.matches());
    }

    #[test]
    fn test_not_matches_bad_format() {
        let mut manifest = Manifest::new();
        manifest.backupper_version = String::from("hutzi-version");
        assert!(!manifest.matches());
    }
}
