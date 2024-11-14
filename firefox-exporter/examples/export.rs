use chrono::{DateTime, Utc};
use clap::Parser;
use futures::TryStreamExt;
use serde::Serialize;
use std::path::PathBuf;
use std::pin;
use std::time::Duration;
use tokio::io::AsyncWriteExt;

#[derive(Parser)]
struct Args {
    #[clap(long, default_value = "default")]
    profile: String,
    #[clap(long)]
    out: PathBuf,
    #[clap(long, value_parser = humantime::parse_duration, default_value = "5s")]
    interval: Duration,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let args = Args::parse();

    let mut exporter = firefox_exporter::FirefoxExporter::new(&args.profile).await?;
    let mut saves = pin::pin!(exporter.watch(args.interval));
    while let Some(save) = saves.try_next().await? {
        #[derive(Serialize)]
        struct Line {
            timestamp: DateTime<Utc>,
            save: String,
        }
        let mut f = tokio::fs::File::options()
            .append(true)
            .create(true)
            .open(&args.out)
            .await?;
        f.write_all(&serde_json::to_vec(&Line {
            timestamp: Utc::now(),
            save,
        })?)
        .await?;
        f.write_all(b"\n").await?;
        f.flush().await?;
    }
    Ok(())
}
