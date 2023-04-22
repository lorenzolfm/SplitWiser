use time::OffsetDateTime;
use tonic::Status;

use super::proto;

pub(super) async fn create(db: &db::Db) -> Result<proto::Id, Status> {
    let id = db
        .write::<_, db::Error, _>(move |conn| {
            db::queries::users::create(conn, OffsetDateTime::now_utc())
        })
        .await
        .map_err(|_| Status::internal("db error"))?;

    Ok(proto::Id { id: *id })
}
