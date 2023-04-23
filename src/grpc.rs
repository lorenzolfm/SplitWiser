use futures::{Future, TryFutureExt};

use tonic::{Request, Response, Status};

use self::proto::{CreateExpenseRequest, CreatePaymentRequest, CreateRevenueRequest, Id};

mod user;

pub mod proto {
    tonic::include_proto!("splitwiser");
}

#[allow(unused)]
pub struct Server {
    db: db::Db,
    env: crate::env::Env,
}

#[tonic::async_trait]
impl proto::splitwiser_server::Splitwiser for Server {
    async fn create_user(&self, _request: Request<()>) -> Result<Response<proto::Id>, Status> {
        user::create(&self.db).map_ok(Response::new).await
    }

    async fn create_revenue(
        &self,
        request: Request<CreateRevenueRequest>,
    ) -> Result<Response<Id>, Status> {
        user::create_revenue(&self.db, request.into_inner())
            .map_ok(Response::new)
            .await
    }

    async fn create_payment(
        &self,
        request: Request<CreatePaymentRequest>,
    ) -> Result<Response<Id>, Status> {
        user::create_payment(&self.db, request.into_inner())
            .map_ok(Response::new)
            .await
    }

    async fn create_expense(
        &self,
        request: Request<CreateExpenseRequest>,
    ) -> Result<Response<Id>, Status> {
        user::create_expense(&self.db, request.into_inner())
            .map_ok(Response::new)
            .await
    }
}

pub(super) fn serve(
    deps: &super::Deps,
) -> impl Future<Output = Result<(), tonic::transport::Error>> {
    let socket = deps.env.socket;

    let server = Server {
        db: deps.db.clone(),
        env: deps.env.clone(),
    };

    tonic::transport::Server::builder()
        .add_service(proto::splitwiser_server::SplitwiserServer::new(server))
        .serve(socket)
}
