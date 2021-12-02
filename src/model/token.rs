use sqlx::{Pool, Postgres};

use crate::model::user::User;

pub struct Token {
	pub user_id: i64,
}

impl Token {
	pub fn new(user_id: i64) -> Self {
		Token {
			user_id
		}
	}

	pub async fn into_user(self, pool: &Pool<Postgres>) -> Result<User, Box<dyn std::error::Error>> {
		User::from_id(self.user_id, pool).await
	}
}

impl From<User> for Token {
	fn from(user: User) -> Self {
		Token {
			user_id: user.user_id
		}
	}
}