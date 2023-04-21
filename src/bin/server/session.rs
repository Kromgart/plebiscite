use std::future::{ Ready, ready };
use std::rc::Rc;

use actix_web::{ FromRequest, HttpRequest, HttpMessage };
use actix_web::dev::{ self, ServiceRequest, Service };

use futures_util::future::LocalBoxFuture;

use crate::db_driver::{ DbDriver, LoggedInUser };


//-------------------------------------------------------------

pub const SESSION_ID: &'static str = "session_id";


//-------------------------------------------------------------

#[derive(Clone)]
pub struct ExtractUserError;

impl Into<actix_web::Error> for ExtractUserError {
    fn into(self) -> actix_web::Error {
        actix_web::error::ErrorUnauthorized("User not logged in")
    } 
}


//-------------------------------------------------------------


impl FromRequest for LoggedInUser {
    type Error = ExtractUserError;
    type Future = Ready<Result<LoggedInUser, ExtractUserError>>;
    
    fn from_request(req: &HttpRequest, _payload: &mut dev::Payload) -> Self::Future {
        let data = match req.extensions().get::<LoggedInUser>() {
            Some(lgu) => Ok(lgu.clone()),
            None => Err(ExtractUserError)
        };

        ready(data)
    }
}


//----------------------------------------------------------------


pub struct SessionMiddleware<S> {
    service: Rc<S>,
    drv: DbDriver,
}


pub async fn get_logged_in_user(req: &HttpRequest, drv: &DbDriver) -> Option<LoggedInUser> {
    match req.cookie(SESSION_ID).and_then(|sid| sqlx::types::Uuid::parse_str(sid.value()).ok()) {
        None => None,
        Some(sid) => drv.get_session_user(sid).await
    }
}


impl<S> Service<ServiceRequest> for SessionMiddleware<S>
where
    S: Service<ServiceRequest, Error = actix_web::Error> + 'static,
    S::Future: 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = LocalBoxFuture<'static, Result<S::Response, Self::Error>>;

    actix_web::dev::forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {

        let srv = self.service.clone();
        let drv = self.drv.clone();

        Box::pin(async move {

            if let Some(lgu) = get_logged_in_user(req.request(), &drv).await {
                req.extensions_mut().insert(lgu);
                srv.call(req).await
            } else {
                Err(actix_web::error::ErrorUnauthorized("Session is missing"))
            }
        })

    }

}

pub struct SessionMiddlewareFactory {
    drv: DbDriver
}

impl SessionMiddlewareFactory {
    pub fn new(drv: DbDriver) -> Self {
        SessionMiddlewareFactory { drv }
    }
}

impl<S> dev::Transform<S, ServiceRequest> for SessionMiddlewareFactory 
where
    S: Service<ServiceRequest, Error = actix_web::Error> + 'static,
    S::Future: 'static
{
    type Response = S::Response;
    type Error = actix_web::Error;
    type InitError = ();
    type Transform = SessionMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(SessionMiddleware {
            service: Rc::new(service),
            drv: self.drv.clone()
        }))
    }
}
