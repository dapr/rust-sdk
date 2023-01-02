use crate::hello_world::greeter_server::{Greeter, GreeterServer};
use crate::hello_world::{HelloReply, HelloRequest};
use tonic::{transport::Server, Request, Response, Status};

pub mod hello_world {
    tonic::include_proto!("helloworld"); // The string specified here must match the proto package name
}

#[derive(Debug, Default)]
pub struct GreeterService {}

#[tonic::async_trait]
impl Greeter for GreeterService {
    async fn say_hello(
        &self,
        request: Request<HelloRequest>,
    ) -> Result<Response<HelloReply>, Status> {
        let req = request.into_inner();

        let name = req.name;

        let response = HelloReply {
            message: format!("Hello {name}!"),
        };

        Ok(Response::new(response))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let server_address = "[::]:50052".parse().unwrap();

    let greeter_service = GreeterService::default();

    println!("AppCallback server listening on: {}", server_address);
    // Create a gRPC server with the callback_service.
    Server::builder()
        .add_service(GreeterServer::new(greeter_service))
        .serve(server_address)
        .await?;

    Ok(())
}
