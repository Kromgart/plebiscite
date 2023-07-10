use actix_web::{cookie, get, post, web, App, HttpRequest, HttpResponse, HttpServer, Responder};

mod db_driver;
mod session;

use db_driver::{DbDriver, User, DbError};

use plebiscite_types::{LoginInfo, UsergroupData};

//----------------------------------------------------------------

impl actix_web::error::ResponseError for DbError {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::ServiceUnavailable().body("Database error")
    }
}


#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let drv = db_driver::DbDriver::new().await;
    let app_data = web::Data::new(drv.clone());

    HttpServer::new(move || {
        App::new()
            .app_data(app_data.clone())
            .service(static_file)
            .service(page_spa_main)
            .service(page_login)
            .service(api_login)
            .service(api_register_login)
            .service(
                web::scope("/api")
                    .wrap(session::SessionMiddlewareFactory::new(drv.clone()))
                    .service(current_user)
                    .service(user_groups)
                    .service(user_group_create)
            )
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}

#[get("/{filepath:.+\\.*(js|wasm)}")]
async fn static_file(filepath: web::Path<String>) -> actix_web::Result<actix_files::NamedFile> {
    let newpath = format!("server_root/{}", filepath);
    println!("Requesting static file: '{}'", newpath);
    Ok(actix_files::NamedFile::open(newpath)?)
}

#[get("/login")]
async fn page_login() -> actix_web::Result<actix_files::NamedFile> {
    Ok(actix_files::NamedFile::open("server_root/login.html")?)
}

//---------- session required ---------------

#[get("/")]
async fn page_spa_main(
    req: HttpRequest,
    drv: web::Data<DbDriver>,
) -> actix_web::Result<HttpResponse> {
    let drv = drv.get_ref();

    match session::get_logged_in_user(&req, drv).await? {
        Some(_) => {
            let file = actix_files::NamedFile::open("server_root/index.html")?;
            let mut resp = file.into_response(&req);
            resp.headers_mut().insert(
                actix_web::http::header::CROSS_ORIGIN_OPENER_POLICY,
                actix_web::http::header::HeaderValue::from_static("same-origin"),
            );
            resp.headers_mut().insert(
                actix_web::http::header::CROSS_ORIGIN_EMBEDDER_POLICY,
                actix_web::http::header::HeaderValue::from_static("require-corp"),
            );
            Ok(resp)
        },
        None => {
            let resp = HttpResponse::SeeOther()
            .insert_header((
                "Location",
                req.url_for_static("page_login").unwrap().as_str(),
            ))
            .finish();
            Ok(resp)
        }
    }
}

//---------- api: public -------------------

fn login_with_cookie(req: HttpRequest, session_id: Option<uuid::Uuid>) -> HttpResponse
{
    if let Some(session_id) = session_id {
        let cookie = cookie::Cookie::build(session::SESSION_ID, session_id.to_string())
            //.domain("localhost")
            .path("/")
            //.secure(true)
            .http_only(true)
            .finish();
        HttpResponse::Ok()
            .cookie(cookie)
            .body(req.url_for_static("page_spa_main").unwrap().to_string())
    } else {
        HttpResponse::Unauthorized().finish()
    }
}


#[post("/api/login")]
async fn api_login(
    req: HttpRequest,
    drv: web::Data<DbDriver>,
    form: web::Json<LoginInfo>,
) -> impl Responder {
    println!("trying to login as {}, {}", form.username, form.password);
    drv.get_ref()
        .try_login(&form.username, &form.password)
        .await
        .map(|session_id| login_with_cookie(req, session_id))
}

#[post("/api/register")]
async fn api_register_login(
    req: HttpRequest,
    drv: web::Data<DbDriver>,
    form: actix_web::web::Json<LoginInfo>,
) -> impl Responder {
    println!("trying to register as {}, {}", form.username, form.password);
    drv.get_ref()
        .try_register_login(&form.username, &form.password)
        .await
        .map(|session_id| login_with_cookie(req, session_id))
}
//---------- api: login protected -----------

macro_rules! respond_ok_json {
    ($drv:ident, $fn:ident ($($args:expr),+)) => {
        $drv.get_ref()
            .$fn($($args),+)
            .await
            .map(|result| HttpResponse::Ok().json(result))
    };
}

macro_rules! respond_ok_text {
    ($drv:ident, $fn:ident ($($args:expr),+) $(-> $($cont:tt)+)?) => {
        $drv.get_ref()
            .$fn($($args),+)
            .await
            .map(|result| HttpResponse::Ok().body(result $(.$($cont)+)?))
    };
}

#[get("/current_user")]
async fn current_user(user: User) -> impl Responder {
    HttpResponse::Ok().body(user.data.user_name)
}


#[get("/user/groups")]
async fn user_groups(user: User, drv: web::Data<DbDriver>) -> Result<HttpResponse, DbError> {
    std::thread::sleep(std::time::Duration::from_millis(1000));
    respond_ok_json!(drv, get_assigned_usergroups(user.user_id))
}

#[post("/user/groups/create")]
async fn user_group_create(user: User, drv: web::Data<DbDriver>, web::Json(group): web::Json<UsergroupData>) -> Result<HttpResponse, DbError> {
    println!("creating group {:?}", group);
    respond_ok_json!(drv, create_usergroup(user.user_id, group))
    //respond_ok_text!(drv, create_usergroup(user.user_id, group) -> value.to_string())
}
