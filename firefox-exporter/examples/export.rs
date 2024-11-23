use chrono::Utc;
use clap::Parser;
use futures::TryStreamExt;
use std::path::PathBuf;
use std::pin;
use std::time::Duration;

#[derive(Parser)]
struct Args {
    #[clap(long, default_value = "default")]
    profile: String,
    #[clap(long, value_parser = humantime::parse_duration, default_value = "5s")]
    interval: Duration,
    #[clap(long)]
    database: PathBuf,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let args = Args::parse();

    let mut exporter = firefox_exporter::FirefoxExporter::new(&args.profile).await?;
    let pool = sqlx::SqlitePool::connect_with(
        sqlx::sqlite::SqliteConnectOptions::new()
            .create_if_missing(true)
            .filename(&args.database),
    )
    .await?;

    sqlx::query("CREATE TABLE IF NOT EXISTS saves(timestamp UNIQUE, value)")
        .execute(&pool)
        .await?;

    let mut saves = pin::pin!(exporter.watch(args.interval));
    while let Some(save) = saves.try_next().await? {
        sqlx::query("INSERT OR REPLACE INTO saves (timestamp, value) values (?, ?)")
            .bind(Utc::now())
            .bind(&save)
            .execute(&pool)
            .await?;
    }

    Ok(())
}
