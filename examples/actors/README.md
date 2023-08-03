# Actor Example

This example demonstrates the Dapr actor framework.  To author an actor, 

1. Create a struct with your custom actor methods that map to [Axum handlers](https://docs.rs/axum/latest/axum/handler/index.html), use [Axum extractors](https://docs.rs/axum/latest/axum/extract/index.html) to access the incoming request and return an [`impl IntoResponse`](https://docs.rs/axum/latest/axum/response/trait.IntoResponse.html).
Use the `DaprJson` extractor to deserialize the request from Json coming from a Dapr sidecar.
    ```rust
    #[derive(Serialize, Deserialize)]
    pub struct MyRequest {
        pub name: String,
    }
    
    #[derive(Serialize, Deserialize)]
    pub struct MyResponse {
        pub available: bool,
    }   

    impl MyActor {
        fn do_stuff(&self, DaprJson(data): DaprJson<MyRequest>) -> Json<MyResponse> {        
            println!("doing stuff with {}", data.name);        
            Json(MyResponse { 
                available: true 
            })
        }    
    }
    ```

    There are many ways to write your actor method signature, using Axum handlers, but you also have access to the actor instance via `self`.  Here is a super simple example:
    ```rust
    pub async fn method_2(&self) -> impl IntoResponse {
        StatusCode::OK
    }
    ```
1. Implement the `Actor` trait.  This trait exposes the following methods:
    - `on_activate` - Called when an actor is activated on a host
    - `on_deactivate` - Called when an actor is deactivated on a host
    - `on_reminder` - Called when a reminder is recieved from the Dapr sidecar
    - `on_timer` - Called when a timer is recieved from the Dapr sidecar


    ```rust
    #[async_trait]
    impl Actor for MyActor {
        
        async fn on_activate(&self) -> Result<(), ActorError> {
            println!("on_activate {}", self.id);
            Ok(())
        }

        async fn on_deactivate(&self) -> Result<(), ActorError> {
            println!("on_deactivate");
            Ok(())
        }
    }
    ```

1. Mark your actor using the `actor!` macro, this enabled methods in your `impl` block to be mapped to Axum handlers.
    ```rust
    dapr::actor!(MyActor);
    ```

1. An actor host requires an Http server to recieve callbacks from the Dapr sidecar.  The `DaprHttpServer` object implements this functionality and also encapsulates the actor runtime to service any hosted actors.  Use the `register_actor` method to register an actor type to be serviced, this method takes an `ActorTypeRegistration` which specifies 
    - The actor type name (used by Actor clients), and concrete struct
    - A factory to construct a new instance of that actor type when one is required to be activated by the runtime.  The parameters passed to the factory will be the actor type, actor ID, and a Dapr client for managing state, timers and reminders for the actor.
    - The methods that you would like to expose to external clients.

    ```rust
    let mut dapr_server = dapr::server::DaprHttpServer::new();

    dapr_server.register_actor(ActorTypeRegistration::new::<MyActor>("MyActor", 
        Box::new(|actor_type, id, client| Arc::new(MyActor{
            actor_type, 
            id, 
            client
        })))
        .register_method("do_stuff", MyActor::do_stuff)
        .register_method("do_other_stuff", MyActor::do_other_stuff));

    dapr_server.start(None).await?;
    ```


## Running

> Before you run the example make sure local redis state store is running by executing:
> ```
> docker ps
> ```

To run this example:

1. Start actor host (expose Http server receiver on port 50051):
```bash
dapr run --app-id actor-host --app-protocol http --app-port 50051 cargo run -- --example actor-server
```

2. Start actor client:
```bash
dapr run --app-id actor-client --dapr-grpc-port 3502 cargo run -- --example actor-client
```
