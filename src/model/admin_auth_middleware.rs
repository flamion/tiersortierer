use std::future::Future;
use std::pin::Pin;
use std::rc::Rc;
use std::task::{Context, Poll};
use actix_web::dev::{Service, ServiceRequest, ServiceResponse, Transform};
use actix_web::{Error, FromRequest};
use futures::future::{Either, Ready, ready};
use crate::model::token::Token;

pub struct AdminAuthenticator;

impl<S: 'static, Req> Transform<S, ServiceRequest> for AdminAuthenticator
	where
		S: Service<ServiceRequest, Response = ServiceResponse<Req>, Error = Error>,
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
	service: Rc<S>
}

impl<S: 'static, Req> Service<ServiceRequest> for AdminAuthenticatorMiddleware<S>
	where
		S: Service<ServiceRequest, Response = ServiceResponse<Req>, Error = Error>,
		S::Future: 'static,
{
	type Response = ServiceResponse<Req>;
	type Error = Error;
	// type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;
	type Future = Either<S::Future, Ready<Result<Self::Response, Self::Error>>>;

	fn poll_ready(&self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
		self.service.poll_ready(cx)
	}

	fn call(&self, mut req: ServiceRequest) -> Self::Future {
		let service = self.service.clone();



		Either::Left(service.call(req))
	}
}