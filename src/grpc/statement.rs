use db::Db;
use tonic::Status;

use crate::features::statement;

use super::proto::GetStatementResponse;

pub(super) async fn get(
    db: &Db,
    payer: i32,
    payee: i32,
    from_timestamp: u64,
    to_timestamp: u64,
) -> Result<GetStatementResponse, Status> {
    match statement::get(&db, payer, payee, from_timestamp, to_timestamp).await {
        Ok(_) => Ok(GetStatementResponse { bills: Vec::new() }),
        Err(_) => Err(Status::unimplemented("treat it")),
    }
}
