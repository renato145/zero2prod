use crate::{
    session_state::TypedSession,
    utils::{e500, see_other},
};
use actix_session::SessionExt;
use actix_web::{
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpMessage,
};
use futures::future::{ok, LocalBoxFuture, Ready};
use std::{
    rc::Rc,
    task::{Context, Poll},
};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct UserId(pub Uuid);

pub struct CheckLoginMiddleware<S> {
    service: Rc<S>,
}

impl<S> Service<ServiceRequest> for CheckLoginMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse, Error = Error> + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&self, cx: &mut Context) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let service = self.service.clone();
        Box::pin(async move {
            let session = TypedSession::new(req.get_session());
            match session.get_user_id().map_err(e500)? {
                Some(id) => {
                    req.extensions_mut().insert(UserId(id));
                    service.call(req).await
                }
                None => Ok(req.into_response(see_other("/login"))),
            }
        })
    }
}

pub struct CheckLogin;

impl<S> Transform<S, ServiceRequest> for CheckLogin
where
    S: Service<ServiceRequest, Response = ServiceResponse, Error = Error> + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type InitError = ();
    type Transform = CheckLoginMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(CheckLoginMiddleware {
            service: Rc::new(service),
        })
    }
}
