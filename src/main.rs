use tonic::{Request, Response, Status};

mod env;

pub mod proto {
    tonic::include_proto!("splitwiser");
}

#[allow(unused)]
pub struct Server {
    db: db::Db,
    env: env::Env,
}

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
    let addr = "0.0.0.0:8000".parse()?;
    let env = env::Env::load();
    let db = db::build(&env.database_url)?;

    let server = Server { db, env };

    tonic::transport::Server::builder()
        .add_service(proto::splitwiser_server::SplitwiserServer::new(server))
        .serve(addr)
        .await?;

    Ok(())
}
