use dapr::actors::*;
use dapr::error::{ActorErrorType, Error};
use std::sync::Arc;

struct HelloActor {}

impl HelloActor {
    pub fn say_hi(&self) -> String {
        "hi there".to_string()
    }

    pub fn say_ho(&self) -> String {
        "ho there".to_string()
    }
}

impl Actor for HelloActor {
    fn invoke(&self, method: &str, _args: &str) -> Result<String, Error> {
        match method {
            "say_hi" => Ok(self.say_hi()),
            "say_ho" => Ok(self.say_ho()),
            _ => Err(Error::from(ActorErrorType::NoSuchMethod)),
        }
    }

    fn register(manager: &mut dyn ActorManager) {
        manager.register("HelloActor", || Arc::new(Box::new(HelloActor {})));
    }
}

#[tokio::main]
async fn main() {
    if std::env::var_os("RUST_LOG").is_none() {
        std::env::set_var("RUST_LOG", "debug")
    }
    let mut manager = ActorManagerImpl::new();
    HelloActor::register(&mut manager);
    serve(manager, "0.0.0.0:3500").await.unwrap();
}
