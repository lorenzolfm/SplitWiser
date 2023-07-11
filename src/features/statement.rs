use db::Db;
use time::OffsetDateTime;

#[derive(Debug, thiserror::Error)]
pub enum GetError {
    #[error("Time conversion error: {0:?}")]
    TimeError(time::error::ComponentRange),
}

pub async fn get(
    db: &Db,
    user_ids: Vec<i32>,
    from_timestamp: u64,
    to_timestamp: u64,
) -> Result<(), GetError> {
    let from =
        OffsetDateTime::from_unix_timestamp(from_timestamp as i64).map_err(GetError::TimeError)?;
    let to =
        OffsetDateTime::from_unix_timestamp(to_timestamp as i64).map_err(GetError::TimeError)?;

    db.read::<_, db::Error, _>(move |conn| Ok(()))
        .await
        .unwrap();

    Ok(())
}
