use std::sync::{Arc, Mutex};
use actix_web::{web, App, HttpRequest, HttpResponse, HttpServer, get, delete, put};
use super::actor::{ActorRuntime, ActorFactory};

pub struct DaprHttpServer {
    actor_runtime: Arc<Mutex<ActorRuntime>>,
}

impl DaprHttpServer {
    pub fn new() -> Self {
        DaprHttpServer {
            actor_runtime: Arc::new(Mutex::new(ActorRuntime::new())),
        }
    }

    pub fn register_actor(&mut self, actor_type: &str, factory: ActorFactory) {
        let mut rt = self.actor_runtime.lock().unwrap();
        rt.register_actor(actor_type, factory);
    }

    pub async fn start(&mut self, addr: Option<&str>, port: Option<u16>) -> Result<(), std::io::Error> {
        
        let rt =  self.actor_runtime.clone();

        let default_port: u16 = std::env::var("APP_PORT")
            .unwrap_or(String::from("8080"))
            .parse()
            .unwrap_or(8080);
    
        
        HttpServer::new(move || {
            App::new()
                .app_data(web::Data::new(rt.clone()))
                .service(health_check)
                .service(registered_actors)
                .service(deactivate_actor)
                .service(invoke_actor)
                .service(invoke_reminder)
                .service(invoke_timer)
        })
        .bind((addr.unwrap_or("127.0.0.1"), port.unwrap_or(default_port)))?
        .run()
        .await
    }
}

#[get("/healthz")]
async fn health_check() -> HttpResponse {
    print!("health check");
    HttpResponse::Ok().finish()
}

#[get("/dapr/config")]
async fn registered_actors(runtime: web::Data<Arc<Mutex<ActorRuntime>>>) -> HttpResponse {
    print!("get actors");
    let ra = runtime.lock().unwrap().list_registered_actors();
    let result = super::models::RegisteredActorsResponse { 
        entities: ra 
    };
    
    HttpResponse::Ok().json(result)
}

#[delete("/actors/{actor_type}/{actor_id}")]
async fn deactivate_actor(runtime: web::Data<Arc<Mutex<ActorRuntime>>>, request: HttpRequest) -> HttpResponse {
    let actor_type = request.match_info().get("actor_type").unwrap();
    let actor_id = request.match_info().get("actor_id").unwrap();
    match runtime.lock().unwrap().deactivate_actor(&actor_type, &actor_id) {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(err) => match err {
            super::actor::ActorError::ActorNotFound => HttpResponse::NotFound().finish(),
            _ => HttpResponse::InternalServerError().finish(),
        },
    }    
}

#[put("/actors/{actor_type}/{actor_id}/method/{method_name}")]
async fn invoke_actor(runtime: web::Data<Arc<Mutex<ActorRuntime>>>, request: HttpRequest, body: web::Bytes) -> HttpResponse {   
    let actor_type = request.match_info().get("actor_type").unwrap();
    let actor_id = request.match_info().get("actor_id").unwrap();
    let method_name = request.match_info().get("method_name").unwrap();
    match runtime.lock().unwrap().invoke_actor(&actor_type, &actor_id, &method_name, body.to_vec()) {
        Ok(output) => HttpResponse::Ok().body(output),
        Err(err) => match err {
            super::actor::ActorError::ActorNotFound => HttpResponse::NotFound().finish(),
            _ => HttpResponse::InternalServerError().finish(),
        },
    }    
}

#[put("/actors/{actor_type}/{actor_id}/method/remind/{reminder_name}")]
async fn invoke_reminder(runtime: web::Data<Arc<Mutex<ActorRuntime>>>, request: HttpRequest) -> HttpResponse {   
    let actor_type = request.match_info().get("actor_type").unwrap();
    let actor_id = request.match_info().get("actor_id").unwrap();
    let reminder_name = request.match_info().get("reminder_name").unwrap();
    match runtime.lock().unwrap().invoke_reminder(&actor_type, &actor_id, &reminder_name) {
        Ok(output) => HttpResponse::Ok().body(output),
        Err(err) => match err {
            super::actor::ActorError::ActorNotFound => HttpResponse::NotFound().finish(),
            _ => HttpResponse::InternalServerError().finish(),
        },
    }    
}

#[put("/actors/{actor_type}/{actor_id}/method/timer/{timer_name}")]
async fn invoke_timer(runtime: web::Data<Arc<Mutex<ActorRuntime>>>, request: HttpRequest) -> HttpResponse {   
    let actor_type = request.match_info().get("actor_type").unwrap();
    let actor_id = request.match_info().get("actor_id").unwrap();
    let timer_name = request.match_info().get("timer_name").unwrap();
    match runtime.lock().unwrap().invoke_timer(&actor_type, &actor_id, &timer_name) {
        Ok(output) => HttpResponse::Ok().body(output),
        Err(err) => match err {
            super::actor::ActorError::ActorNotFound => HttpResponse::NotFound().finish(),
            _ => HttpResponse::InternalServerError().finish(),
        },
    }    
}