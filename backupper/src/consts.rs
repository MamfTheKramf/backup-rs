//! Contains constant magic values

/// Filename of profile config within the backup archive
pub const PROFILE_CONF_NAME: &str = ".PROFILE_CONFIG";

/// Filename of manifest within the backup archive
pub const MANIFEST_NAME: &str = ".MANIFEST";

/// Filename of the file record within the backup archive
pub const FILE_RECORD_NAME: &str = ".FILE_RECORD";

/// List of reserved filenames to make it easier to filter out
pub const RESERVED_FILENAMES: [&str; 3] = [PROFILE_CONF_NAME, MANIFEST_NAME, FILE_RECORD_NAME];
