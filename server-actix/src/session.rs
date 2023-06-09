use std::future::{ Ready, ready };
use std::rc::Rc;

use actix_web::{ FromRequest, HttpRequest, HttpMessage };
use actix_web::dev::{ self, ServiceRequest, Service };

use futures_util::future::LocalBoxFuture;

use crate::db_driver::{ DbDriver, User, DbResult };


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

impl FromRequest for User {
    type Error = ExtractUserError;
    type Future = Ready<Result<User, ExtractUserError>>;
    
    fn from_request(req: &HttpRequest, _payload: &mut dev::Payload) -> Self::Future {
        let data = match req.extensions().get::<User>() {
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

pub async fn get_logged_in_user(req: &HttpRequest, drv: &DbDriver) -> DbResult<Option<User>> {
    match req.cookie(SESSION_ID).and_then(|sid| uuid::Uuid::parse_str(sid.value()).ok()) {
        None => Ok(None),
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

            match get_logged_in_user(req.request(), &drv).await {
                Ok(Some(lgu)) => {
                    req.extensions_mut().insert(lgu);
                    srv.call(req).await
                },
                Ok(None) => {
                    println!("SessionMiddleware: no session");
                    Err(actix_web::error::ErrorUnauthorized("Session is missing"))
                },
                Err(e) => {
                    println!("SessionMiddleware: {:?}", e);
                    Err(actix_web::error::ErrorServiceUnavailable("Database error"))
                },
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
