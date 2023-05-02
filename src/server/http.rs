use std::{sync::{Arc, Mutex}};
use actix_web::{web, App, HttpRequest, HttpResponse, HttpServer, get, delete, put, middleware};
use super::actor::{runtime::{ActorRuntime}, ActorFactory, context_client::{GrpcDaprClient, ActorContextClient}, ActorBuilder};
use super::super::client::TonicClient;

type GrpcActorRuntime = ActorRuntime<TonicClient>;

pub struct DaprHttpServer {
    actor_runtime: Arc<Mutex<GrpcActorRuntime>>,
}

impl DaprHttpServer {
    pub async fn new() -> Self {
        let dapr_port: u16 = std::env::var("DAPR_GRPC_PORT").unwrap().parse().unwrap();
        let dapr_addr = format!("https://127.0.0.1:{}", dapr_port);
        
        let cc = match TonicClient::connect(dapr_addr).await {
            Ok(c) => c,
            Err(err) => panic!("failed to connect to dapr: {}", err)
        };
                
        DaprHttpServer {
            actor_runtime: Arc::new(Mutex::new(GrpcActorRuntime::new(cc, Box::new(ActorContextClient::<TonicClient>::new)))),
        }
    }

    pub fn register_actor(&mut self, actor_type: &str, factory: ActorFactory<GrpcDaprClient>) {
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
                .wrap(middleware::Logger::default())
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
    log::debug!("recieved health check request");
    HttpResponse::Ok().finish()
}

#[get("/dapr/config")]
async fn registered_actors(runtime: web::Data<Arc<Mutex<GrpcActorRuntime>>>) -> HttpResponse {
    log::debug!("daprd requested registered actors");
    let ra = runtime.lock().unwrap().list_registered_actors();
    let result = super::models::RegisteredActorsResponse { 
        entities: ra 
    };
    
    HttpResponse::Ok().json(result)
}

#[delete("/actors/{actor_type}/{actor_id}")]
async fn deactivate_actor(runtime: web::Data<Arc<Mutex<GrpcActorRuntime>>>, request: HttpRequest) -> HttpResponse {
    let actor_type = request.match_info().get("actor_type").unwrap();
    let actor_id = request.match_info().get("actor_id").unwrap();
    match runtime.lock().unwrap().deactivate_actor(&actor_type, &actor_id).await {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(err) => match err {
            super::actor::ActorError::ActorNotFound => HttpResponse::NotFound().finish(),
            _ => HttpResponse::InternalServerError().finish(),
        },
    }    
}

#[put("/actors/{actor_type}/{actor_id}/method/{method_name}")]
async fn invoke_actor(runtime: web::Data<Arc<Mutex<GrpcActorRuntime>>>, request: HttpRequest, body: web::Bytes) -> HttpResponse {   
    let actor_type = request.match_info().get("actor_type").unwrap();
    let actor_id = request.match_info().get("actor_id").unwrap();
    let method_name = request.match_info().get("method_name").unwrap();
    log::debug!("invoke_actor: {} {} {}", actor_type, actor_id, method_name);
    match runtime.lock().unwrap().invoke_actor(&actor_type, &actor_id, &method_name, body.to_vec()).await {
        Ok(output) => HttpResponse::Ok().body(output),
        Err(err) => match err {
            super::actor::ActorError::ActorNotFound => HttpResponse::NotFound().finish(),
            _ => HttpResponse::InternalServerError().finish(),
        },
    }    
}

#[put("/actors/{actor_type}/{actor_id}/method/remind/{reminder_name}")]
async fn invoke_reminder(runtime: web::Data<Arc<Mutex<GrpcActorRuntime>>>, request: HttpRequest, body: web::Bytes) -> HttpResponse {   
    let actor_type = request.match_info().get("actor_type").unwrap();
    let actor_id = request.match_info().get("actor_id").unwrap();
    let reminder_name = request.match_info().get("reminder_name").unwrap();
    let payload = serde_json::from_slice::<ReminderPayload>(&body.to_vec()).unwrap();
    log::debug!("invoke_reminder: {} {} {} {:?}", actor_type, actor_id, reminder_name, payload);    

    match runtime.lock().unwrap().invoke_reminder(&actor_type, &actor_id, &reminder_name, payload.data.unwrap_or_default().into_bytes()).await {
        Ok(output) => HttpResponse::Ok().body(output),
        Err(err) => match err {
            super::actor::ActorError::ActorNotFound => HttpResponse::NotFound().finish(),
            _ => HttpResponse::InternalServerError().finish(),
        },
    }    
}

#[put("/actors/{actor_type}/{actor_id}/method/timer/{timer_name}")]
async fn invoke_timer(runtime: web::Data<Arc<Mutex<GrpcActorRuntime>>>, request: HttpRequest, body: web::Bytes) -> HttpResponse {   
    let actor_type = request.match_info().get("actor_type").unwrap();
    let actor_id = request.match_info().get("actor_id").unwrap();
    let timer_name = request.match_info().get("timer_name").unwrap();
    let payload = serde_json::from_slice::<TimerPayload>(&body.to_vec()).unwrap();
    log::debug!("invoke_timer: {} {} {}, {:?}", actor_type, actor_id, timer_name, payload);

    match runtime.lock().unwrap().invoke_timer(&actor_type, &actor_id, &timer_name, payload.data.unwrap_or_default().into_bytes()).await {
        Ok(output) => HttpResponse::Ok().body(output),
        Err(err) => match err {
            super::actor::ActorError::ActorNotFound => HttpResponse::NotFound().finish(),
            _ => HttpResponse::InternalServerError().finish(),
        },
    }    
}

#[derive(serde::Deserialize, Debug)]
struct ReminderPayload {
    data: Option<String>,
}

#[derive(serde::Deserialize, Debug)]
struct TimerPayload {
    data: Option<String>,
}