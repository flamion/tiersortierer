use actix_web::{get, HttpResponse, post, Responder, web};
use sqlx::{Pool, Postgres};

use crate::model::token::Token;
use crate::model::user::{NewUser, User};
use crate::util::password_matches_requirements;

#[post("")]
pub async fn create_user(new_user: web::Json<NewUser>, db_pool: web::Data<Pool<Postgres>>) -> Result<impl Responder, Box<dyn std::error::Error>> {
	if User::username_exists(new_user.username.as_str(), &db_pool).await? {
		return Ok(HttpResponse::Conflict().finish());
	}

	if !password_matches_requirements(new_user.password.as_str()) {
		return Ok(HttpResponse::BadRequest().body("Password does not meet the requirements"));
	}

	let new_user = User::new(&*new_user, &db_pool).await?;

	Ok(HttpResponse::Created().json(new_user.user_id))
}

#[get("")]
pub async fn get_user(db_pool: web::Data<Pool<Postgres>>, token: Token) -> Result<impl Responder, Box<dyn std::error::Error>> {
	Ok(web::Json(token.into_user(&db_pool).await?))
}