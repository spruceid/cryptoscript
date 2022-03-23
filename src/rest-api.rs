use std::time::SystemTime;
use std::sync::{Arc, Mutex};

use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use indexmap::IndexMap;
use serde_json::{Map, Value};
use serde::{Deserialize, Serialize};

// TODO:
// - post a new api w/
//  + request json
//  + response json
//  + rate limit
// - get an api:
//  + require posted request json
//  + return posted response json
//  + enforce rate limit
// - get all apis as list
// - top level: link to /apis and provide example

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
struct Api {
    request: Value,
    response: Value,
    rate_limit_seconds: u64,
    last_api_call: Option<SystemTime>,
}

#[derive(Clone, Debug)]
struct AppState {
    apis: Arc<Mutex<IndexMap<String, Api>>>,
}

impl AppState {
    fn new() -> Self {
        Self {
            apis: Arc::new(Mutex::new(IndexMap::new())),
        }
    }

    // panics if Mutex lock fails
    fn api(&mut self, name: String, api: Api) {
        self.apis.lock().unwrap().insert(name, api);
    }
}

#[get("/")]
async fn index() -> impl Responder {
    let body_str = r#"
        Routes:
        - /
        - /apis
    "#;
    HttpResponse::Ok().body(body_str)
}

#[get("/apis")]
async fn index_apis(data: web::Data<AppState>) -> impl Responder {
    let json_body: Result<Map<String, Value>, String> = data.apis
        .lock()
        .map_err(|e| format!("{}", e))
        .and_then(|x| { x
            .clone()
            .into_iter()
            .map(|(x, y)| Ok((x, serde_json::to_value(y).map_err(|e| format!("{}", e))?)))
            .collect::<Result<Map<String, Value>, String>>()
    });
    let pretty_json = serde_json::to_string_pretty(&json_body.unwrap()).unwrap();
    // HttpResponse::Ok().json(json_body)
    HttpResponse::Ok().body(pretty_json)
}

#[get("/apis/{api_id}")]
async fn get_api(path: web::Path<String>, data: web::Data<AppState>) -> impl Responder {
    let path_str: String = path.into_inner();
    match data.apis.lock().map_err(|e| format!("{}", e)) {
        Ok(apis) => {
            let json_response = (*apis).get(&path_str)
                .ok_or_else(|| format!("API not found: {:?}", path_str));
            HttpResponse::Ok().json(json_response)
        },
        Err(e) =>
            HttpResponse::NotFound().body(format!("GET /apis/{} failed:\n{}", path_str, e)),
    }
}

// #[post("/apis/{api_id}")]
// async fn echo(req_body: String) -> impl Responder {
//     HttpResponse::Ok().body(req_body)
// }

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let server_root = "127.0.0.1";
    let server_port = 8080;
    let server_address = format!("http://{}:{}", server_root, server_port);
    println!("Starting server..");
    println!("- {}      root", server_address);
    println!("- {}/apis API's root", server_address);

    let mut app_state = AppState::new();
    app_state.api("got_null".to_string(), Api {
        request: Value::Null,
        response: Value::String("Got null?".to_string()),
        rate_limit_seconds: 1,
        last_api_call: None,
    });

    app_state.api("got_number".to_string(), Api {
        request: Value::Number(From::from(0u8)),
        response: Value::String("Got 0, as expected!".to_string()),
        rate_limit_seconds: 1,
        last_api_call: None,
    });

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(app_state.clone()))
            .service(index)
            .service(index_apis)
            .service(get_api)
            // .route("/hey", web::get().to(manual_hello))
    })
    .bind((server_root, server_port))?
    .run()
    .await
}

