# Actor Example

This example demonstrates the Dapr actor framework.  To author an actor, 

1. Create a struct with your custom actor methods and annotate your input and output types as serializable.  The SDK will automatically deserialize the incoming parameters from JSON and then serialize your result back to JSON.
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
        fn do_stuff(&mut self, data: MyRequest) -> Result<MyResponse, actor::ActorError> {        
            println!("doing stuff with {}", data.name);        
            Ok(MyResponse { 
                available: true 
            })
        }    
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
        
        async fn on_activate(&mut self) -> Result<(), ActorError> {
            println!("on_activate {}", self.id);
            Ok(())
        }

        async fn on_deactivate(&mut self) -> Result<(), ActorError> {
            println!("on_deactivate");
            Ok(())
        }
    }
    ```

1. An actor host requires an Http server to recieve callbacks from the Dapr sidecar.  The `DaprHttpServer` object implements this functionality and also encapsulates the actor runtime to service any hosted actors.  Use the `register_actor` method to register an actor type to be serviced, this method takes an `ActorTypeRegistration` which specifies 
    - The actor type name
    - A factory to construct a new instance of that actor type when one is required to be activated by the runtime.  The parameters passed to the factory will be the actor type, actor ID, and a Dapr client for managing state, timers and reminders for the actor.
    - The methods that you would like to expose to external clients.

    ```rust
    let mut dapr_server = dapr::server::DaprHttpServer::new();

    dapr_server.register_actor(ActorTypeRegistration::new("MyActor", 
        |actor_type, id, client| Box::new(MyActor{
            actor_type, 
            id, 
            client
        }))
        .register_method("do_stuff", MyActor::do_stuff)
        .register_method("do_other_stuff", MyActor::do_other_stuff));

    dapr_server.start(None, None).await?;
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
