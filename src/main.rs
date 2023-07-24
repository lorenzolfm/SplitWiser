mod env;
mod features;
mod grpc;

struct Deps {
    db: db::Db,
    env: crate::env::Env,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let env = env::Env::load();
    let db = db::build(&env.database_url)?;

    let deps = Deps { db, env };

    grpc::serve(&deps).await?;

    Ok(())
}
