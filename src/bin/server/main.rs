use actix_web::{ web, get, post, cookie,
                 App, HttpServer, HttpRequest, HttpResponse, Responder
               };

mod db_driver;
mod session;

use db_driver::{ DbDriver, LoggedInUser };

//----------------------------------------------------------------

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
            .service(web::scope("/api")
                        .wrap(session::SessionMiddlewareFactory::new(drv.clone()))
                        .service(current_user)
                        .service(user_groups)
            )
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}


#[get("/{filepath:.+\\.*(js|wasm)}")]
async fn static_file(filepath: web::Path<String>) -> actix_web::Result<actix_files::NamedFile> {
    let newpath = format!("server_root/{}", filepath);
    println!("{}", newpath);
    Ok(actix_files::NamedFile::open(newpath)?)
}



#[get("/login")]
async fn page_login() -> actix_web::Result<actix_files::NamedFile> {
    Ok(actix_files::NamedFile::open("server_root/login.html")?)
}




//---------- session required ---------------

#[get("/")]
async fn page_spa_main(req: HttpRequest, drv: web::Data<DbDriver>) -> actix_web::Result<HttpResponse> {
    let drv = drv.get_ref();

    if let Some(_) = session::get_logged_in_user(&req, drv).await {
        let file = actix_files::NamedFile::open("server_root/index.html")?;
        Ok(file.into_response(&req))
    } else {
        let resp = HttpResponse::SeeOther()
                                .insert_header(("Location", req.url_for_static("page_login").unwrap().as_str()))
                                .finish();
        Ok(resp)
    }
}


//---------- api: public -------------------

#[derive(serde::Deserialize)]
struct LoginInfo {
    username: String,
    password: String
}

#[post("/api/login")]
async fn api_login(req: HttpRequest, drv: web::Data<DbDriver>, form: actix_web::web::Json<LoginInfo>) -> impl Responder {
    let result = format!("trying to login as {}, {}", form.username, form.password);
    println!("{}", result);

    let drv = drv.get_ref();
    if let Some(session_id) = drv.try_login(&form.username, &form.password).await { 
        let cookie = cookie::Cookie::build(session::SESSION_ID, session_id.to_string())
                                    //.domain("localhost")
                                    .path("/")
                                    //.secure(true)
                                    .http_only(true)
                                    .finish();
        HttpResponse::Ok()
            .cookie(cookie)
            .body(req.url_for_static("page_spa_main")
                     .unwrap()
                     .to_string())
    } else {
        HttpResponse::Unauthorized().finish()
    }
}


//---------- api: login protected -----------

#[get("/current_user")]
async fn current_user(user: LoggedInUser) -> impl Responder {
    HttpResponse::Ok().body(user.user_name)
}


#[get("/user/groups")]
async fn user_groups(user: LoggedInUser, drv: web::Data<DbDriver>) -> impl Responder {
    let drv = drv.get_ref();

    let groups = drv.get_assigned_usergroups(user.user_id).await;

    HttpResponse::Ok().json(groups)
}


