use axum::{handler::Handler, Router, routing::put};
use tokio::sync::RwLock;
use std::{
    collections::HashMap,
    sync::Arc,
};

use crate::client::TonicClient;

use super::{
    context_client::ActorContextClient,
    Actor, ActorError, ActorFactory
};

#[derive(Clone)]
pub struct ActorState {
    pub actor_type: String,
    pub runtime: Arc<ActorRuntime>,
}

pub struct ActorTypeRegistration {
    name: String,
    factory: ActorFactory,
    method_registrations: HashMap<String, Box<dyn (FnOnce(Router, Arc<ActorRuntime>) -> Router) + Send + Sync>>,
}

impl ActorTypeRegistration {
    pub fn new<TActor>(name: &str) -> Self where TActor: Actor + Send + Sync + 'static {        
        ActorTypeRegistration {
            name: name.to_string(),
            factory: Arc::new(TActor::new),
            method_registrations: HashMap::new(),
        }
    }

    pub fn register_method<T>(mut self, method_name: &str, handler: impl Handler<T, ActorState> + Send + Sync) -> Self
    where T: 'static
    {
        let actor_type = self.name.clone();
        let method_path = format!("/actors/{}/:actor_id/method/{}", actor_type, method_name);

        let reg_func = move |router: Router, runtime: Arc<ActorRuntime>| {
            router
                .route(&method_path, put(handler)
                .with_state(ActorState {
                    actor_type,
                    runtime,
                }))
        };
        
        self.method_registrations.insert(method_name.to_string(), Box::new(reg_func));
        self
    }

    fn create_actor(&self, actor_id: &str, client: TonicClient) -> Arc<dyn Actor> {
        let client = ActorContextClient::new(client, &self.name, actor_id);
        let actor = (self.factory)(&self.name, actor_id, client);
        actor
    }
    
}

pub struct ActorRuntime {
    dapr_client: TonicClient,
    
    registered_actors_types: Arc<RwLock<HashMap<String, ActorTypeRegistration>>>,
    active_actors: Arc<RwLock<HashMap<(String, String), Arc<dyn Actor>>>>,
}

impl ActorRuntime {
    pub fn new(
        dapr_client: TonicClient,
    ) -> Self {
        ActorRuntime {
            dapr_client,
            registered_actors_types: Arc::new(RwLock::new(HashMap::new())),
            active_actors: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn register_actor(&self, registration: ActorTypeRegistration) {        
        let name = registration.name.clone();
        let mut g = self.registered_actors_types.write().await;
        g.insert(name.clone(), registration);
        log::info!("registered actor {}", name);
    }

    pub async fn configure_method_routes(&self, router: Router, runtime: Arc<ActorRuntime>) -> Router {
        let mut router = router;
        let mut types = self.registered_actors_types.write().await;
        
        for (_, registration) in types.iter_mut() {            
            for (_, reg_func) in registration.method_registrations.drain() {
                router = reg_func(router, runtime.clone());
            }
        }
        router
    }
    
    pub async fn deactivate_actor(&self, name: &str, id: &str) -> Result<(), ActorError> {
        let mut actors = self.active_actors.write().await;
        
        let actor = match actors.remove(&(name.to_string(), id.to_string()))
        {
            Some(actor_ref) => actor_ref,
            None => return Err(ActorError::ActorNotFound),
        };
        drop(actors);
        actor.on_deactivate().await?;
        drop(actor);
        Ok(())
    }

    pub async fn deactivate_all(&self) {
        let mut actors = self.active_actors.write().await;
        
        for actor in actors.values() {
            actor.on_deactivate().await;
        }
        actors.clear();
    }

    pub async fn invoke_reminder(&self, name: &str, id: &str, reminder_name: &str, data: Vec<u8>) -> Result<(), ActorError> {
        let actor = self.get_or_create_actor(name, id).await?;
        actor.on_reminder(reminder_name, data).await?;
        Ok(())
    }

    pub async fn invoke_timer(&self, name: &str, id: &str, timer_name: &str, data: Vec<u8>) -> Result<(), ActorError> {
        let actor = self.get_or_create_actor(name, id).await?;        
        actor.on_timer(timer_name, data).await?;
        Ok(())
    }

    pub async fn list_registered_actors(&self) -> Vec<String> {
        let types = self.registered_actors_types.read().await;
        
        types
            .keys()
            .map(|k| k.to_string())
            .collect()
    }

    pub async fn get_or_create_actor(&self, actor_type: &str, id: &str) -> Result<Arc<dyn Actor>, ActorError> {
        let actors = self.active_actors.read().await;        
        match actors.get(&(actor_type.to_string(), id.to_string()))
        {
            Some(actor_ref) => Ok(actor_ref.clone()),
            None => {
                drop(actors);
                self.activate_actor(actor_type, id).await
            },
        }
    }

    async fn activate_actor(&self, actor_type: &str, id: &str) -> Result<Arc<dyn Actor>, ActorError> {
        let types = self.registered_actors_types.read().await;
        let actor = match types.get(actor_type) {
            Some(f) => f.create_actor(id, self.dapr_client.clone()),
            None => Err(ActorError::NotRegistered)?,
        };

        actor.on_activate().await?;
        
        let actor_key = (actor_type.to_string(), id.to_string());        
        let mut actors = self.active_actors.write().await;
        actors.insert(actor_key, actor.clone());

        Ok(actor)
    }
}

#[cfg(test)]
mod tests;
