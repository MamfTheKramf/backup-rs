use config::general_config::GeneralConfig;
use rocket::http::Status;
use rocket::State;

/// Returns the path to the dir that stores the [ProfileConfig]s.
/// 
/// If the [GeneralConfig] file can't be opened, a `404` Error is returned.
/// On other errors, a `500` Error is returned.
#[get("/profiles/config_dir")]
pub async fn get_profile_config_dir(general_config: &State<GeneralConfig>) -> (Status, String) {
    (Status::Ok, format!("{:?}", general_config.profile_configs))
}

// #[get("/profiles")]
// pub async fn get_profile_configs()
