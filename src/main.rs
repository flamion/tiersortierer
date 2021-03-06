use std::time::Duration;

use actix_web::{App, HttpServer, web};
use log::LevelFilter;
use simple_logger::SimpleLogger;
use sqlx::postgres::PgPoolOptions;

use crate::config::Config;
use crate::endpoints::admin::admin_endpoint_test;
use crate::endpoints::user::{create_user, get_user, login};
use crate::model::admin_auth_middleware::AdminAuthenticator;

mod config;
mod model;
mod endpoints;
mod util;

type Error = Box<dyn std::error::Error + Send + Sync>;

#[actix_web::main]
async fn main() -> Result<(), Error> {
	SimpleLogger::new()
		.with_level(LevelFilter::Error)
		.with_module_level("tiersortierer", LevelFilter::Debug)
		.init()
		.unwrap();

	log::info!("Reading config...");
	let config = Config::new().expect("Config file not found.");
	log::info!("Done!");

	log::info!("Connecting to the database...");
	let pool = PgPoolOptions::new()
		.max_lifetime(Duration::from_secs(6000))
		.min_connections(3)
		.max_connections(10)
		.connect(config.database.database_url.as_str())
		.await?;
	log::info!("Done!");

	// Without giving it arguments it will default to $PROJECT_ROOT/migrations
	log::info!("Migrating Database...");
	sqlx::migrate!()
		.run(&pool)
		.await?;
	log::info!("Done!");

	let pool_data = web::Data::new(pool);
	let config_data = web::Data::new(config);

	log::info!("Starting actix server.");
	HttpServer::new(move || {
		// let json_config = web::JsonConfig::default()
		// 	.limit(4096);

		App::new()
			.app_data(pool_data.clone())
			.app_data(config_data.clone())
			.service(web::scope("/user")
				.service(create_user)
				.service(get_user)
				.service(login)
			)
			.service(web::scope("/admin")
				.wrap(AdminAuthenticator)
				.service(admin_endpoint_test)
			)
	})
		.bind("127.0.0.1:8080")?
		.run()
		.await?;

	Ok(())
}