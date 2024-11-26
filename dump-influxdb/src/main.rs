use chrono::{DateTime, Utc};
use clap::Parser;
use std::path::PathBuf;
use tokio::fs::File;
use tokio::io::{AsyncWriteExt, BufWriter};

#[derive(Parser)]
struct Args {
    #[clap(long)]
    database: String,
    #[clap(long)]
    out: PathBuf,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let args = Args::parse();

    let pool = sqlx::SqlitePool::connect_with(
        sqlx::sqlite::SqliteConnectOptions::new().filename(&args.database),
    )
    .await?;
    let mut out = BufWriter::new(File::create(&args.out).await?);

    let mut prev = DateTime::<Utc>::MIN_UTC;
    loop {
        let rows = sqlx::query_as::<_, (DateTime<Utc>, String)>(
            "SELECT timestamp, value FROM saves WHERE timestamp > ? ORDER BY timestamp LIMIT 256",
        )
        .bind(prev)
        .fetch_all(&pool)
        .await?;

        if let Some((last, _)) = rows.last() {
            prev = *last;
        } else {
            break;
        }

        let builder = rows.into_iter().try_fold(
            influxdb_line_protocol::LineProtocolBuilder::new(),
            |builder, (timestamp, value)| {
                let save = cookieclicker_save::decode(&value)?;
                anyhow::Ok(
                    builder
                        .measurement("save")
                        .field(
                            "cookies_in_bank",
                            save.miscellaneous_game_data.cookies_in_bank,
                        )
                        .field("cookies_baked", save.miscellaneous_game_data.cookies_baked)
                        .field(
                            "cookies_forfeited_by_ascending",
                            save.miscellaneous_game_data.cookies_forfeited_by_ascending,
                        )
                        .timestamp(
                            timestamp
                                .timestamp_nanos_opt()
                                .ok_or_else(|| anyhow::format_err!("timestamp out of range"))?,
                        )
                        .close_line(),
                )
            },
        )?;

        out.write_all(&builder.build()).await?;
    }

    out.flush().await?;

    Ok(())
}
