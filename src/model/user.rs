use std::error::Error;
use std::fmt::{Display, Formatter};

use actix_web::{HttpResponse, HttpResponseBuilder, ResponseError};
use actix_web::http::StatusCode;
use argon2::{PasswordHash, PasswordVerifier};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Pool, Postgres, Row};
use sqlx::postgres::PgRow;

use crate::model::token::Token;
use crate::util::{get_password_hash, time_now};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
// TODO sanitize username to remove spaces, make it lowercase etc.
pub struct NewUser {
	pub username: String,
	pub password: String,
	pub email_address: Option<String>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoginUser {
	pub username: String,
	pub password: String,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct User {
	pub user_id: i64,
	pub username: String,
	pub email_address: Option<String>,
	pub creation_time: i64,
	pub last_login_time: i64,
	pub is_admin: bool,
}

#[derive(Debug, Clone, Copy)]
pub enum LoginError {
	WrongCredentials = 401,
	InternalServerError = 500,
}

impl LoginUser {
	/// Verifies the given credentials and creates a new `Token`, returning the struct of the newly
	/// created `Token`.
	pub async fn login(&self, db_pool: &Pool<Postgres>) -> Result<Token, LoginError> {
		let query = sqlx::query!(
			"SELECT user_id, password FROM users WHERE LOWER(username) = LOWER($1)",
			self.username,
		)
			.fetch_optional(db_pool)
			.await?;

		if let Some(query) = query {
			let user_id = query.user_id;
			let user = User::from_id(user_id, db_pool).await?;

			let db_password_hash = query.password;
			let parsed_hash = PasswordHash::new(db_password_hash.as_str())?;
			if !argon2::Argon2::default().verify_password(self.password.as_bytes(), &parsed_hash).is_ok() {
				return Err(LoginError::WrongCredentials);
			}


			Ok(Token::new(&user, db_pool).await?)
		} else {
			Err(LoginError::WrongCredentials)
		}
	}
}

impl User {
	/// Creates a new user and puts the details in the database. Also returns the struct of the
	/// newly created user.
	pub async fn new(new_user: &NewUser, pool: &Pool<Postgres>) -> Result<Self, Box<dyn std::error::Error>> {
		let password_hash = get_password_hash(new_user.password.as_str());
		let now = time_now();

		let created_user = sqlx::query!(
			r#"
			INSERT INTO users(username, password, email_address, creation_time, last_login_time, is_admin)
			VALUES ($1, $2, $3, $4, $5, $6)
			RETURNING user_id AS new_id
			"#,
			new_user.username.to_lowercase(),
			password_hash,
			new_user.email_address,
			now,
			now,
			false
		)
			.fetch_one(pool)
			.await?;


		Ok(Self {
			user_id: created_user.new_id,
			username: new_user.username.to_lowercase(),
			email_address: new_user.email_address.clone(),
			creation_time: now,
			last_login_time: now,
			is_admin: false,
		})
	}

	/// Takes a user ID and retrieves the corresponding User from the Database
	pub async fn from_id(user_id: i64, pool: &Pool<Postgres>) -> Result<User, Box<dyn std::error::Error>> {
		let user = sqlx::query_as(r#"SELECT * FROM users WHERE user_id = $1"#)
			.bind(user_id)
			.fetch_one(pool)
			.await?;


		Ok(user)
	}

	/// Checks the database whether a user with the specified username already exists
	//  If the answer from the db is empty user does not exist.
	pub async fn id_exists(id: i64, pool: &Pool<Postgres>) -> Result<bool, Box<dyn std::error::Error>> {
		let user_row = sqlx::query!(r#"SELECT user_id FROM users WHERE user_id = $1"#, id)
			.fetch_all(pool)
			.await?;

		log::trace!("User rows fetched from db is empty: {}", user_row.is_empty());

		Ok(!user_row.is_empty())
	}

	/// Checks the database whether a user with the specified username already exists
	//  If the answer from the db is empty user does not exist.
	// TODO use query! macro if that works
	pub async fn username_exists(username: &str, pool: &Pool<Postgres>) -> Result<bool, Box<dyn std::error::Error>> {
		let user_row = sqlx::query(r#"SELECT user_id FROM users WHERE LOWER(username) = LOWER($1)"#)
			.bind(username)
			.fetch_all(pool)
			.await?;

		log::trace!("User rows fetched from db is empty: {}", user_row.is_empty());

		Ok(!user_row.is_empty())
	}

	/// Deletes a user from the database.
	/// Returns a boolean of whether it was successful or whether there was a problem.
	pub async fn delete(user_id: i64, pool: &Pool<Postgres>) -> bool {
		let query = sqlx::query!("DELETE FROM users WHERE user_id = $1", user_id)
			.execute(pool)
			.await;

		query.is_ok()
	}
}

// impl for query_as.
// A custom impl is needed as the password, which is saved in the db, is not present in the user struct.
impl FromRow<'_, PgRow> for User {
	fn from_row(row: &PgRow) -> Result<Self, sqlx::Error> {
		let user_id = row.try_get("user_id")?;
		let username = row.try_get("username")?;
		let email_address = row.try_get("email_address")?;
		let creation_time = row.try_get("creation_time")?;
		let last_login_time = row.try_get("last_login_time")?;
		let is_admin = row.try_get("is_admin")?;


		Ok(Self {
			user_id,
			username,
			email_address,
			last_login_time,
			creation_time,
			is_admin,
		})
	}
}

impl Display for LoginError {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", match self {
			LoginError::WrongCredentials => "Incorrect Credentials Provided",
			LoginError::InternalServerError => "Internal Server Error",
		})
	}
}

impl Error for LoginError {}


// This can probably be done better...
impl From<sqlx::Error> for LoginError {
	fn from(error: sqlx::Error) -> Self {
		log::error!("{:?}", error);
		LoginError::InternalServerError
	}
}

impl From<Box<dyn std::error::Error>> for LoginError {
	fn from(error: Box<dyn Error>) -> Self {
		log::error!("{}", error);
		LoginError::InternalServerError
	}
}

impl From<argon2::password_hash::Error> for LoginError {
	fn from(error: argon2::password_hash::Error) -> Self {
		log::error!("{}", error);
		LoginError::InternalServerError
	}
}

impl ResponseError for LoginError {
	fn status_code(&self) -> StatusCode {
		match self {
			LoginError::WrongCredentials => StatusCode::UNAUTHORIZED,
			LoginError::InternalServerError => StatusCode::INTERNAL_SERVER_ERROR,
		}
	}

	fn error_response(&self) -> HttpResponse {
		HttpResponseBuilder::new(self.status_code())
			.body(self.to_string())
	}
}