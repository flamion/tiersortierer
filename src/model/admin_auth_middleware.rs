use std::future::Future;
use std::pin::Pin;
use std::rc::Rc;
use std::task::{Context, Poll};

use actix_web::{Error, FromRequest};
use actix_web::dev::{Service, ServiceRequest, ServiceResponse, Transform};
use futures::future::{Ready, ready};

use crate::model::token::Token;
use crate::model::token::TokenError::TokenUnauthorized;

pub struct AdminAuthenticator;

impl<S: 'static, Req> Transform<S, ServiceRequest> for AdminAuthenticator
	where
		S: Service<ServiceRequest, Response=ServiceResponse<Req>, Error=Error>,
		S::Future: 'static,
{
	type Response = ServiceResponse<Req>;
	type Error = Error;
	type Transform = AdminAuthenticatorMiddleware<S>;
	type InitError = ();
	type Future = Ready<Result<Self::Transform, Self::InitError>>;

	fn new_transform(&self, service: S) -> Self::Future {
		ready(Ok(AdminAuthenticatorMiddleware {
			service: Rc::new(service),
		}))
	}
}

pub struct AdminAuthenticatorMiddleware<S> {
	service: Rc<S>,
}

impl<S: 'static, Req> Service<ServiceRequest> for AdminAuthenticatorMiddleware<S>
	where
		S: Service<ServiceRequest, Response=ServiceResponse<Req>, Error=Error>,
		S::Future: 'static,
{
	type Response = ServiceResponse<Req>;
	type Error = actix_web::Error;
	type Future = Pin<Box<dyn Future<Output=Result<Self::Response, Self::Error>>>>;


	fn poll_ready(&self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
		self.service.poll_ready(cx)
	}

	fn call(&self, mut req: ServiceRequest) -> Self::Future {
		let service = self.service.clone();

		Box::pin(async move {
			let mut parts = req.parts_mut();
			let token = Token::from_request(&parts.0, &mut parts.1).await?;

			if !token.is_admin {
				return Err(TokenUnauthorized.into());
			}

			service.call(req).await
		})
	}
}