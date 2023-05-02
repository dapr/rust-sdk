# Actor Example

This example demonstrates the Dapr actor framework.  To define an actor, implement the `Actor` trait.  This trait exposes the following methods:
- `on_activate` - Called when an actor is activated on a host
- `on_deactivate` - Called when an actor is deactivated on a host
- `on_invoke` - Called when an actor is invoked from a client
- `on_reminder` - Called when a reminder is recieved from the Dapr sidecar
- `on_timer` - Called when a timer is recieved from the Dapr sidecar

```rust
impl actor::Actor for MyActor {
    
    async fn on_activate(&mut self) -> Result<(), ActorError> {
        println!("on_activate {}", self.id);
        Ok(())
    }

    fn on_deactivate(&mut self) -> Result<(), ActorError> {
        println!("on_deactivate");
        Ok(())
    }

    fn on_invoke(&mut self, method: &str, data: Vec<u8>) -> Result<Vec<u8>, actor::ActorError> {
        println!("on_invoke {} {:?}", method, from_utf8(&data));
               
        match method {
            "do_stuff" => {
                let args = serde_json::from_slice::<MyRequest>(&data);
                if args.is_err() {
                    return Err(ActorError::SerializationError());
                }
                
                match self.do_stuff(args.unwrap()).await {
                    Ok(r) => Ok(serde_json::to_vec(&r).unwrap()),
                    Err(e) => Err(e)
                }
            }
            _ => Err(actor::ActorError::MethodNotFound)
        }
    }

}

impl MyActor {
    fn new(actor_type: &str, id: &str, client: Box<ActorContextClient<TonicClient>>) -> Self {
        println!("creating actor {} {}", id, actor_type);
        MyActor {
            id: id.to_string(),
            client
        }
    }

    fn do_stuff(&mut self, data: MyRequest) -> Result<MyResponse, actor::ActorError> {        
        println!("doing stuff with {}", data.name);        
        Ok(MyResponse { available: true })
    }    
}
```

An actor host requires an Http server to recieve callbacks from the Dapr sidecar.  The `DaprHttpServer` object implements this functionality and also encapsulates the actor runtime to service any hosted actors.  Use the `register_actor` method to register an actor type to be serviced, this method takes the actor type name and a factory to construct a new instance of that actor type when one is required to be activated by the runtime.  The parameters passed to the factory will be the actor type, actor ID, and a Dapr client for managing state, timers and reminders for the actor.

```rust
let mut dapr_server = dapr::server::DaprHttpServer::new();
dapr_server.register_actor("MyActor", Box::new(|actor_type, id, client| Arc::new(Mutex::new(MyActor::new(actor_type, id, client)))));
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
