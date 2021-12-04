use std::time::{SystemTime, UNIX_EPOCH};

use argon2::{Argon2, PasswordHasher};
use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::SaltString;

pub fn get_password_hash(password: &str) -> String {
	let salt = SaltString::generate(&mut OsRng);
	let argon2 = Argon2::default();
	let password_hash = argon2.hash_password(password.as_bytes(), &salt).unwrap().to_string();
	log::debug!("password hash is {}", password_hash);

	password_hash
}

pub fn password_matches_requirements(password: &str) -> bool {
	password.len() >= 10
}

pub fn time_now() -> i64 {
	let now = SystemTime::now();
	let since_unix_epoch = now.duration_since(UNIX_EPOCH).expect("We time travelled");
	since_unix_epoch.as_millis() as i64
}