use serde::{Serialize, Deserialize};
use sqlx::{Executor, FromRow, Pool, Postgres, Row};
use sqlx::postgres::PgRow;
use crate::util::{get_password_hash, time_now};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewUser {
	pub username: String,
	pub password: String,
	pub email_address: Option<String>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct User {
	pub user_id: i64,
	pub username: String,
	pub email_address: Option<String>,
	pub creation_date: i64,
	pub last_login_date: i64,
}

impl User {
	pub async fn new(new_user: &NewUser, pool: &Pool<Postgres>) -> Result<Self, Box<dyn std::error::Error>> {
		let password_hash = get_password_hash(new_user.password.as_str());
		let now = time_now();

		let created_user = sqlx::query!(
			r#"
			INSERT INTO users(username, password, email_address, creation_time, last_login_time)
			VALUES ($1, $2, $3, $4, $5)
			RETURNING user_id AS new_id
			"#,
			new_user.username.to_lowercase(),
			password_hash,
			new_user.email_address,
			now,
			now,
		)
			.fetch_one(pool)
			.await?;


		Ok(Self {
			user_id: created_user.new_id,
			username: new_user.username.to_lowercase(),
			email_address: new_user.email_address.clone(),
			creation_date: now,
			last_login_date: now,
		})
	}

	/// Takes a user ID and retrieves the corresponding User from the Database
	pub async fn user_from_id(user_id: i64, pool: &Pool<Postgres>) -> Result<User, Box<dyn std::error::Error>> {
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

impl FromRow<'_, PgRow> for User {
	fn from_row(row: &PgRow) -> Result<Self, sqlx::Error> {
		let user_id = row.try_get("user_id")?;
		let username = row.try_get("username")?;
		let email_address = row.try_get("email_address")?;
		let creation_date = row.try_get("create_date")?;
		let last_login_date = row.try_get("last_login_date")?;


		Ok(Self {
			user_id,
			username,
			email_address,
			last_login_date,
			creation_date,
		})
	}
}