use std::{sync::Mutex, env};

use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder, Result, middleware::Logger, Error, error::ErrorBadRequest};
use serde_json::json;
use serde;

use utoipa::{
    openapi::{security::{ApiKey, ApiKeyValue, SecurityScheme}, self},
    Modify, OpenApi,
};
use utoipa::{ToSchema, IntoParams};
//use utoipa_swagger_ui::SwaggerUi;

struct AppState {
    hits: Mutex<u64>
}

#[derive(Debug, serde::Serialize, ToSchema)]
struct Person {
    name: String,
    picked_number: u64
}

#[utoipa::path(
    responses(
        (status = 200, description = "If everything is OK"),
        (status = 500, description = "If something if broken"),
    )
)]
#[get("/health")]
async fn health(state: web::Data<AppState>) -> Result<impl Responder> {
    let mut counter = state.hits.lock().unwrap();
    *counter += 1;

    Ok(web::Json(json!({
        "status": "UP",
        "hits": *counter
    })))
}

#[utoipa::path(
    responses(
        (status = 200, description = "Return a Person", body = [Person])
    )
)]
#[get("/hello/{name}/{number}")]
async fn hello(path: web::Path<(String, u64)>, state: web::Data<AppState>) -> Result<web::Json<Person> > {
    let (name, number) = path.into_inner();

    if number > 100 {
        return Err(ErrorBadRequest("Number is too high"));
    }

    let person = Person {
        name,
        picked_number: number
    };
    
    Ok(
        web::Json(person)
    )
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    //env::set_var("RUST_LOG", "actix_web=info");
    std::env::set_var("RUST_LOG", "info");
    std::env::set_var("RUST_BACKTRACE", "1");
    env_logger::init();
    println!("Starting");

    let state = web::Data::new( AppState {
        hits: Mutex::new(0)
    });

    #[derive(OpenApi)]
    #[openapi(
        paths(health, hello),
        components(
            schemas(Person)
        )
    )]
    struct ApiDoc;

    let swagger = ApiDoc::openapi();
    let serialized = serde_yaml::to_string(&swagger).unwrap();
    println!("{}", serialized);

    HttpServer::new(move || {
        let logger = Logger::default();

        App::new()
            .wrap(logger)
            .app_data(state.clone())
            .service(health)
            .service(hello)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}