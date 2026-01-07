use lib;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    #[cfg(debug_assertions)]
    dotenvy::from_path(std::env::current_dir()?.join("app/.env")).ok();
    lib::learn_cast().await?;
    Ok(())
}
