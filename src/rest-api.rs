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

// TODO:
// - implement post

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
struct Api {
    request: Value,
    response: Value,
    rate_limit_seconds: u64,
    last_api_call: Option<SystemTime>,
}

impl Api {
    pub fn check_rate_limit(&self) -> Result<(), String> {
        match self.last_api_call {
            None => Ok(()),
            Some(last_call_time) => {
                let elapsed_seconds = last_call_time
                    .elapsed()
                    .map_err(|e| format!("internal SystemTime error: {:?}", e))?
                    .as_secs();
                if self.rate_limit_seconds <= elapsed_seconds {
                    Ok(())
                } else {
                    Err(format!("rate limit exceeded:\n{} seconds since last call, but need {} seconds",
                                elapsed_seconds,
                                self.rate_limit_seconds))
                }
            },
        }
    }

    pub fn called_now(&self) -> Self {
        Api {
            request: self.request.clone(),
            response: self.response.clone(),
            rate_limit_seconds: self.rate_limit_seconds,
            last_api_call: Some(SystemTime::now()),
        }
    }
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

    /// panics if Mutex lock fails
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
    HttpResponse::Ok().body(pretty_json)
}

#[get("/apis/{api_id}")]
async fn get_api(path: web::Path<String>, data: web::Data<AppState>, query: web::Json<Value>) -> impl Responder {
    let path_str: String = path.into_inner();
    match data.apis.lock().map_err(|e| format!("{}", e)) {
        Ok(mut apis) => {
            println!("DEBUG:\npath:\n{}\napis:\n{:?}\nquery\n{}", path_str, apis, query);
            let json_response = apis.clone().get(&path_str)
                .ok_or_else(|| format!("API not found: {:?}", path_str))
                .and_then(|api| api.check_rate_limit().map(|_| api))
                .and_then(|api| {
                    if api.request == query.clone() {
                        let new_api = api.called_now();
                        apis.insert(path_str, new_api);
                        Ok(api.response.clone())
                    } else {
                        Err(format!("unexpected request JSON, expected:\n{}", api.request))
                    }
                });
            match json_response {
                Ok(response) => {
                    println!("response: {}", response);
                    HttpResponse::Ok().json(response)
                },
                Err(ref e) => {
                    println!("error: {}", e);
                    HttpResponse::BadRequest().json((e.clone(), json_response, query))
                },
            }
        },
        Err(e) =>
            HttpResponse::NotFound().body(format!("GET /apis/{} failed:\n{}", path_str, e)),
    }
}

#[post("/apis/{api_id}")]
async fn post_api(_path: web::Path<String>, _data: web::Data<AppState>, request: web::Json<Api>) -> impl Responder {
    HttpResponse::Ok().json(request.into_inner())
}

// async fn post_api(path: ) -> impl Responder {
    // HttpResponse::Ok().body(req_body)
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
        response: Value::String("Got null!".to_string()),
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
            .service(post_api)
    })
    .bind((server_root, server_port))?
    .run()
    .await
}

