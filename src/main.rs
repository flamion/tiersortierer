use std::sync::Arc;
use std::time::Duration;
use sqlx::postgres::PgPoolOptions;
use crate::config::Config;

mod config;

type Error = Box<dyn std::error::Error + Send + Sync>;

#[actix_web::main]
async fn main() -> Result<(), Error> {
	let config = Arc::new(Config::new()?);

	let pool = PgPoolOptions::new()
		.max_lifetime(Duration::from_secs(6000))
		.min_connections(3)
		.max_connections(10)
		.connect(config.database.database_url.as_str())
		.await?;

	// Without giving it arguments it will default to $PROJECT_ROOT/migrations
	sqlx::migrate!()
		.run(&pool)
		.await?;

	Ok(())
}