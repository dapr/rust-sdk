use super::*;
use crate::server::actor::Actor;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Clone)]
struct FakeActorClient {}

impl DaprActorInterface for FakeActorClient {
    fn connect<'async_trait>(
        addr: String,
    ) -> core::pin::Pin<
        Box<
            dyn core::future::Future<Output = Result<Self, crate::error::Error>>
                + core::marker::Send
                + 'async_trait,
        >,
    >
    where
        Self: Sized,
        Self: 'async_trait,
    {
        todo!()
    }

    fn get_actor_state<'life0, 'async_trait>(
        &'life0 mut self,
        request: crate::server::actor::context_client::GetActorStateRequest,
    ) -> core::pin::Pin<
        Box<
            dyn core::future::Future<
                    Output = Result<
                        crate::server::actor::context_client::GetActorStateResponse,
                        crate::error::Error,
                    >,
                > + core::marker::Send
                + 'async_trait,
        >,
    >
    where
        'life0: 'async_trait,
        Self: 'async_trait,
    {
        todo!()
    }

    fn register_actor_reminder<'life0, 'async_trait>(
        &'life0 mut self,
        request: crate::server::actor::context_client::RegisterActorReminderRequest,
    ) -> core::pin::Pin<
        Box<
            dyn core::future::Future<Output = Result<(), crate::error::Error>>
                + core::marker::Send
                + 'async_trait,
        >,
    >
    where
        'life0: 'async_trait,
        Self: 'async_trait,
    {
        todo!()
    }

    fn register_actor_timer<'life0, 'async_trait>(
        &'life0 mut self,
        request: crate::server::actor::context_client::RegisterActorTimerRequest,
    ) -> core::pin::Pin<
        Box<
            dyn core::future::Future<Output = Result<(), crate::error::Error>>
                + core::marker::Send
                + 'async_trait,
        >,
    >
    where
        'life0: 'async_trait,
        Self: 'async_trait,
    {
        todo!()
    }

    fn unregister_actor_reminder<'life0, 'async_trait>(
        &'life0 mut self,
        request: crate::server::actor::context_client::UnregisterActorReminderRequest,
    ) -> core::pin::Pin<
        Box<
            dyn core::future::Future<Output = Result<(), crate::error::Error>>
                + core::marker::Send
                + 'async_trait,
        >,
    >
    where
        'life0: 'async_trait,
        Self: 'async_trait,
    {
        todo!()
    }

    fn unregister_actor_timer<'life0, 'async_trait>(
        &'life0 mut self,
        request: crate::server::actor::context_client::UnregisterActorTimerRequest,
    ) -> core::pin::Pin<
        Box<
            dyn core::future::Future<Output = Result<(), crate::error::Error>>
                + core::marker::Send
                + 'async_trait,
        >,
    >
    where
        'life0: 'async_trait,
        Self: 'async_trait,
    {
        todo!()
    }

    fn execute_actor_state_transaction<'life0, 'async_trait>(
        &'life0 mut self,
        request: crate::server::actor::context_client::ExecuteActorStateTransactionRequest,
    ) -> core::pin::Pin<
        Box<
            dyn core::future::Future<Output = Result<(), crate::error::Error>>
                + core::marker::Send
                + 'async_trait,
        >,
    >
    where
        'life0: 'async_trait,
        Self: 'async_trait,
    {
        todo!()
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MyResponse {
    pub available: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MyRequest {
    pub name: String,
}

struct MyActor {
    id: String,
}

#[async_trait]
impl Actor for MyActor {
    async fn on_activate(&mut self) -> Result<(), ActorError> {
        Ok(())
    }

    async fn on_deactivate(&mut self) -> Result<(), ActorError> {
        Ok(())
    }

    async fn on_reminder(&mut self, _reminder_name: &str, _data: Vec<u8>) -> Result<(), ActorError> {
        Ok(())
    }

    async fn on_timer(&mut self, _timer_name: &str, _data: Vec<u8>) -> Result<(), ActorError> {
        Ok(())
    }
}

impl MyActor {
    fn new(id: &str) -> Self {
        MyActor { id: id.to_string() }
    }
    fn do_stuff(&mut self, data: MyRequest) -> Result<MyResponse, ActorError> {
        println!("doing stuff with {}", data.name);
        Ok(MyResponse { available: true })
    }
}

#[test]
fn test_actor_invoke() {
    let mut runtime = ActorRuntime::<FakeActorClient>::new(FakeActorClient {}, Box::new(|c, t, n| ActorContextClient::<FakeActorClient>::new(c, t, n)));
    runtime.register_actor(
        ActorTypeRegistration::new("MyActor", |actor_type, id, client| {
            Box::new(MyActor {
                id,
            })
        })
        .register_method("do_stuff", MyActor::do_stuff),
    );
    let data_str = r#"{ "name": "foo" }"#;
    let data = data_str.as_bytes().to_vec();

    match futures::executor::block_on(runtime.invoke_actor("MyActor", "1", "do_stuff", data)) {
        Ok(response) => {
            let response: MyResponse = serde_json::from_slice(&response).unwrap();
            assert_eq!(response.available, true);
        }
        Err(e) => panic!("error: {:?}", e),
    };
}

#[test]
fn test_actor_deactivate() {
    let mut runtime = ActorRuntime::<FakeActorClient>::new(FakeActorClient {}, Box::new(|c, t, n| ActorContextClient::<FakeActorClient>::new(c, t, n)));
    runtime.register_actor(
        ActorTypeRegistration::new("MyActor", |actor_type, id, client| {
            Box::new(MyActor {
                id,
            })
        })
        .register_method("do_stuff", MyActor::do_stuff),
    );

    let data_str = r#"{ "name": "foo" }"#;
    let data = data_str.as_bytes().to_vec();
    _ = futures::executor::block_on(runtime.invoke_actor("MyActor", "1", "do_stuff", data));

    match futures::executor::block_on(runtime.deactivate_actor("MyActor", "1")) {
        Ok(_) => (),
        Err(e) => panic!("error: {:?}", e),
    };

    match futures::executor::block_on(runtime.deactivate_actor("MyActor", "1")) {
        Ok(_) => panic!("should not be able to deactivate twice"),
        Err(e) => match e {
            ActorError::ActorNotFound => (),
            _ => panic!("wrong error: {:?}", e),
        },
    };
}
