use std::fs::File;
use std::io::Read;

use serde::Deserialize;

use crate::Error;

#[derive(Deserialize)]
pub struct Config {
	pub general: General,
	pub database: Database,
}

#[derive(Deserialize)]
pub struct General {
	#[serde(default = "token_valid_duration_default")]
	pub token_valid_duration: i64,
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

/// Default duration a token is valid for, in milliseconds
const fn token_valid_duration_default() -> i64 {
	//millis (one second) * 60 for 1 minute * 60 for 1 hour * 24 for 1 day * 7 for one week * 2 for 2 weeks
	1000 * 60 * 60 * 24 * 7 * 2
}