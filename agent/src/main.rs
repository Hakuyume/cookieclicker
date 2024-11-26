use chrono::Utc;
use clap::Parser;
use std::path::PathBuf;
use std::time::Duration;
use tokio::sync::Mutex;

#[derive(Parser)]
struct Args {
    #[clap(long, default_value = "http://127.0.0.1:4444")]
    webdriver: String,
    #[clap(long)]
    database: PathBuf,
    #[clap(long, value_parser = humantime::parse_duration, default_value = "5s")]
    interval: Duration,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let args = Args::parse();

    let pool = sqlx::SqlitePool::connect_with(
        sqlx::sqlite::SqliteConnectOptions::new()
            .create_if_missing(true)
            .filename(&args.database),
    )
    .await?;
    sqlx::query("CREATE TABLE IF NOT EXISTS saves(timestamp UNIQUE, value)")
        .execute(&pool)
        .await?;

    automation::fantoccini::with(&args.webdriver, |client| async {
        let client = Mutex::new(automation::Client::new(client));

        let current = client.lock().await.export_save().await?;
        let backup = sqlx::query_scalar::<_, String>(
            "SELECT value FROM saves ORDER BY timestamp DESC LIMIT 1",
        )
        .fetch_optional(&pool)
        .await?;
        if let Some(backup) = backup {
            if cookies_baked_all_time(&cookieclicker_save::decode(&backup)?)
                > cookies_baked_all_time(&cookieclicker_save::decode(&current)?)
            {
                client.lock().await.import_save(&backup).await?;
            }
        }

        futures::future::try_join(
            async {
                loop {
                    let current = client.lock().await.export_save().await?;
                    sqlx::query("INSERT OR REPLACE INTO saves (timestamp, value) values (?, ?)")
                        .bind(Utc::now())
                        .bind(&current)
                        .execute(&pool)
                        .await?;
                    tokio::time::sleep(Duration::from_secs(300)).await
                }

                #[allow(unreachable_code)]
                anyhow::Ok(())
            },
            async {
                loop {
                    client.lock().await.big_cookie().await?;
                    tokio::time::sleep(Duration::from_millis(100)).await
                }

                #[allow(unreachable_code)]
                anyhow::Ok(())
            },
        )
        .await?;

        Ok(())
    })
    .await
}

fn cookies_baked_all_time(save: &cookieclicker_save::Save) -> f64 {
    save.miscellaneous_game_data.cookies_baked
        + save.miscellaneous_game_data.cookies_forfeited_by_ascending
}
