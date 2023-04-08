use std::path::PathBuf;

use cli_args::parse_args;
use config::general_config::GeneralConfig;
use log::info;

#[macro_use]
extern crate rocket;

mod api;
mod cli_args;
mod errors;

#[get("/")]
async fn index() -> &'static str {
    "Hello"
}

fn init_logger(path: &PathBuf) {
    match log4rs::init_file(path, Default::default()) {
        Ok(_) => info!("Initialized logger"),
        Err(e) => {
            println!("Couldn't initialize logger: {:?}", e);
            std::process::exit(1);
        }
    }
}

#[launch]
fn rocket() -> _ {
    let args = parse_args();
    init_logger(&PathBuf::from(&args.logger_config));
    info!("CLI-Args: {:#?}", args);

    std::env::set_var("ROCKET_CLI_COLORS", format!("{}", args.rocket_colors));

    let general_config = match GeneralConfig::read(&PathBuf::from(&args.general_config)) {
        Ok(config) => config,
        Err(e) => {
            log::error!(
                "Couldn't read general config from {}. Got: {:#?}",
                args.general_config,
                e
            );
            std::process::exit(1);
        }
    };

    rocket::build()
        .manage(general_config)
        .mount("/", routes![index])
        .mount(
            "/api",
            routes![
                api::get_profile_config_dir,
                api::get_profile_configs,
                api::get_profile_config_by_name,
                api::get_profile_config_by_uuid,
                api::create_blank_profile_config,
                api::delete_profile_config_by_name,
                api::delete_profile_config_by_uuid,
                api::update_profile_config
            ],
        )
}
