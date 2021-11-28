use crate::Error;
use std::fs::File;
use std::io::Read;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Config {
	pub database: Database,
}

#[derive(Deserialize)]
pub struct Database {
	pub database_url: String,
}

impl Config {
	pub fn new() -> Result<Self, Error> {
		let mut config_file = File::open("./config.toml")?;
		let mut config = String::new();
		config_file.read_to_string(&mut config)?;

		Ok(toml::from_str(config.as_str())?)
	}
}