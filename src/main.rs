use tonic::{Request, Response, Status};

pub mod proto {
    tonic::include_proto!("splitwiser");
}

pub struct Server {}

#[tonic::async_trait]
impl proto::splitwiser_server::Splitwiser for Server {
    async fn say_hello(
        &self,
        _request: Request<()>,
    ) -> Result<Response<proto::HelloResponse>, Status> {
        Ok(Response::new(proto::HelloResponse {
            message: "Hello World".into(),
        }))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let server = Server {};
    let addr = "0.0.0.0:8000".parse()?;

    tonic::transport::Server::builder()
        .add_service(proto::splitwiser_server::SplitwiserServer::new(server))
        .serve(addr)
        .await?;

    Ok(())
}
