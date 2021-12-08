use std::error::Error;
use std::fmt::{Display, Formatter};
use std::future::Future;
use std::pin::Pin;

use actix_web::{FromRequest, HttpRequest, HttpResponse, HttpResponseBuilder, ResponseError, web};
use actix_web::dev::Payload;
use actix_web::http::StatusCode;
use serde::Serialize;
use sqlx::{FromRow, Pool, Postgres, Row};
use sqlx::postgres::PgRow;

use crate::Config;
use crate::model::user::User;
use crate::util::{generate_token, get_token_from_header, time_now};

// Maybe completely replace TokenError with a custom crate error
#[derive(Debug, Clone, Copy)]
pub enum TokenError {
	BadTokenFormat = 400,
	TokenUnauthorized = 401,
	InternalServerError = 500,
}

#[derive(Serialize)]
pub struct Token {
	pub user_id: i64,
	pub token_string: String,
	#[serde(skip_serializing)]
	pub is_admin: bool,
	#[serde(skip_serializing)]
	pub creation_time: i64,
}

impl Token {
	/// Creates a new token, saves it in the database and returns a struct with information about the token.
	pub async fn new(user: &User, pool: &Pool<Postgres>) -> Result<Self, sqlx::Error> {
		let token = generate_token();
		let creation_time = time_now();

		let _ = sqlx::query!(
			r#"INSERT INTO tokens(user_id, token, creation_time) VALUES ($1, $2, $3)"#,
			user.user_id,
			token,
			creation_time
		)
			.execute(pool)
			.await?;


		Ok(Token {
			user_id: user.user_id,
			token_string: token,
			is_admin: user.is_admin,
			creation_time,
		})
	}

	/// Turns the token into a user struct.
	pub async fn into_user(self, pool: &Pool<Postgres>) -> Result<User, Box<dyn std::error::Error>> {
		User::from_id(self.user_id, pool).await
	}

	/// Retrieves information about a token from the database.
	pub async fn from_str(token: &str, db_pool: &Pool<Postgres>, config: &Config) -> Result<Self, TokenError> {
		let query: Option<Token> = sqlx::query_as(
			"SELECT t.creation_time AS creation_time, \
			t.user_id AS user_id, t.token AS token, u.is_admin AS is_admin \
			FROM tokens t \
			INNER JOIN users u ON t.user_id = u.user_id AND t.token = $1;"
		)
			.bind(token)
			.fetch_optional(db_pool)
			.await?;

		return if let Some(token) = query {
			if token.is_invalid(&config) {
				return Err(TokenError::TokenUnauthorized);
			}

			Ok(token)
		} else {
			Err(TokenError::TokenUnauthorized)
		};
	}

	/// Checks whether a token is still valid.
	#[inline(always)]
	pub fn is_invalid(&self, config: &Config) -> bool {
		let time_now = time_now();

		(time_now - self.creation_time) > config.general.token_valid_duration
	}
}

impl FromRow<'_, PgRow> for Token {
	fn from_row(row: &'_ PgRow) -> Result<Self, sqlx::Error> {
		let user_id = row.try_get("user_id")?;
		let token_string = row.try_get("token")?;
		let is_admin = row.try_get("is_admin")?;
		let creation_time = row.try_get("creation_time")?;


		Ok(Self {
			user_id,
			token_string,
			is_admin,
			creation_time,
		})
	}
}

impl FromRequest for Token {
	type Error = actix_web::error::Error;
	type Future = Pin<Box<dyn Future<Output=Result<Self, Self::Error>>>>;

	fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
		let req = req.clone();

		Box::pin(async move {
			let db_pool = req.app_data::<web::Data<Pool<Postgres>>>().unwrap(); //We can unwrap as it **has** to be there
			let config = req.app_data::<web::Data<Config>>().unwrap(); //Same here, config has to exist otherwise we wouldn't be at this point

			let token = get_token_from_header(req.headers().get("Token"))?;
			let token = Token::from_str(token.as_str(), db_pool, config).await?;

			Ok(token)
		})
	}
}


impl Display for TokenError {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", match self {
			TokenError::BadTokenFormat => "Bad Token",
			TokenError::TokenUnauthorized => "Unauthorized",
			TokenError::InternalServerError => "Internal Server Error",
		})
	}
}

impl Error for TokenError {}

impl ResponseError for TokenError {
	fn status_code(&self) -> StatusCode {
		match self {
			TokenError::BadTokenFormat => StatusCode::BAD_REQUEST,
			TokenError::TokenUnauthorized => StatusCode::UNAUTHORIZED,
			TokenError::InternalServerError => StatusCode::INTERNAL_SERVER_ERROR,
		}
	}

	fn error_response(&self) -> HttpResponse {
		HttpResponseBuilder::new(self.status_code())
			.body(self.to_string())
	}
}

impl From<sqlx::Error> for TokenError {
	fn from(err: sqlx::Error) -> Self {
		log::error!("{:?}", err);
		TokenError::InternalServerError
	}
}