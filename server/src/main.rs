use std::path::PathBuf;

use api::get_profile_config_dir;
use cli_args::parse_args;
use log::info;

#[macro_use] extern crate rocket;

mod cli_args;
mod api;

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

    rocket::build()
        .manage(PathBuf::from(args.general_config))
        .mount("/", routes![index])
        .mount("/api", routes![get_profile_config_dir])
}
