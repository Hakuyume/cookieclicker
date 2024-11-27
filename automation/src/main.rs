mod client;
mod database;
mod locator;
mod save;

use clap::Parser;
use std::path::PathBuf;
use std::time::Duration;
use strum::VariantArray;
use tokio::sync::Mutex;

#[derive(Parser)]
struct Args {
    #[clap(long, default_value = "http://localhost:4444")]
    webdriver: String,
    #[clap(long)]
    database: PathBuf,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    let (handle, registration) = futures::future::AbortHandle::new_pair();
    ctrlc::set_handler(move || handle.abort())?;

    let args = Args::parse();

    let client = fantoccini::ClientBuilder::rustls()?
        .connect(&args.webdriver)
        .await?;
    let output =
        futures::future::Abortable::new(main_impl(args, client.clone()), registration).await;
    client.close().await?;
    tracing::info!("close");

    let output = output?;
    if let Err(e) = &output {
        if let Some(fantoccini::error::CmdError::Standard(e)) =
            e.downcast_ref::<fantoccini::error::CmdError>()
        {
            tracing::error!(error = ?e.error);
        }
    }
    output
}

async fn main_impl(args: Args, client: fantoccini::Client) -> anyhow::Result<()> {
    let database = database::Database::connect(&args.database).await?;
    let (observer, operator) = client::split(client);

    resume(&database, &observer, &operator).await?;
    futures::future::try_join3(
        backup(&database, &observer),
        big_cookie(&operator),
        store(&operator),
    )
    .await?;
    Ok(())
}

#[tracing::instrument(skip_all)]
async fn resume(
    database: &database::Database,
    observer: &client::Observer,
    operator: &Mutex<client::Operator>,
) -> anyhow::Result<()> {
    match futures::future::try_join(observer.fetch_save(), database.fetch_save_best()).await? {
        (None, Some(backup)) => operator.lock().await.import_save(&backup).await?,
        (Some(current), Some(backup))
            if current.cookies_baked_all_time() < backup.cookies_baked_all_time() =>
        {
            operator.lock().await.import_save(&backup).await?
        }
        _ => (),
    }
    Ok(())
}

#[tracing::instrument(skip_all)]
async fn backup(database: &database::Database, observer: &client::Observer) -> anyhow::Result<()> {
    let mut interval = tokio::time::interval(Duration::from_secs(60));
    loop {
        interval.tick().await;
        if let Some(save) = observer.fetch_save().await? {
            database.insert_save(&save).await?;
        }
    }
}

#[tracing::instrument(skip_all)]
async fn big_cookie(operator: &Mutex<client::Operator>) -> anyhow::Result<()> {
    let mut interval = tokio::time::interval(Duration::from_millis(100));
    loop {
        interval.tick().await;
        if let Ok(mut operator) = operator.try_lock() {
            operator.try_click(locator::BIG_COOKIE).await?;
        }
    }
}

#[tracing::instrument(err, skip(operator))]
async fn store(operator: &Mutex<client::Operator>) -> anyhow::Result<()> {
    let mut interval = tokio::time::interval(Duration::from_secs(5));
    loop {
        interval.tick().await;
        operator
            .lock()
            .await
            .try_click(locator::STORE_BUY_ALL_UPGRADES)
            .await?;
        for building in locator::Building::VARIANTS {
            interval.tick().await;
            let mut operator = operator.lock().await;
            operator.try_click(locator::STORE_BUIK10).await?;
            operator
                .try_click(locator::store_building(*building))
                .await?;
        }
    }
}
