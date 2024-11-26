use chrono::Utc;
use clap::Parser;
use fantoccini::error::CmdError;
use futures::FutureExt;
use std::fmt::Debug;
use std::path::{Path, PathBuf};
use std::time::Duration;
use tokio::sync::Mutex;
use tracing::Instrument;

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
    output??;

    Ok(())
}

async fn main_impl(args: Args, client: fantoccini::Client) -> anyhow::Result<()> {
    let pool = init_sqlite(&args.database).await?;
    let (observer, operator) = split(client);

    resume(&pool, &observer, &operator).await?;
    futures::future::try_join(backup(&pool, &observer), big_cookie(&operator)).await?;
    Ok(())
}

#[tracing::instrument(ret, skip(value))]
fn cookies_baked_all_time(value: &cookieclicker_save::Save) -> f64 {
    value.miscellaneous_game_data.cookies_baked
        + value.miscellaneous_game_data.cookies_forfeited_by_ascending
}

#[tracing::instrument(err)]
async fn init_sqlite<P>(filename: P) -> anyhow::Result<sqlx::SqlitePool>
where
    P: Debug + AsRef<Path>,
{
    let pool = sqlx::SqlitePool::connect_with(
        sqlx::sqlite::SqliteConnectOptions::new()
            .create_if_missing(true)
            .filename(filename),
    )
    .await?;
    sqlx::query(concat!(
        "CREATE TABLE IF NOT EXISTS saves(",
        "id INTEGER PRIMARY KEY AUTOINCREMENT,",
        "timestamp TEXT,",
        "value TEXT,",
        "cookies_baked_all_time REAL)"
    ))
    .execute(&pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS cookies_baked_all_time ON saves(cookies_baked_all_time)",
    )
    .execute(&pool)
    .await?;
    Ok(pool)
}

fn split(client: fantoccini::Client) -> (Observer, Mutex<Operator>) {
    (
        Observer(client.clone()),
        Mutex::new(Operator(client.clone())),
    )
}

#[tracing::instrument(err, skip(pool, observer, operator))]
async fn resume(
    pool: &sqlx::SqlitePool,
    observer: &Observer,
    operator: &Mutex<Operator>,
) -> anyhow::Result<()> {
    let _span = tracing::info_span!("resume").entered();
    let (current, backup) = futures::future::try_join(
        observer
            .export_save()
            .map(|value| -> anyhow::Result<_> {
                if let Some(value) = value? {
                    let cookies = cookies_baked_all_time(&cookieclicker_save::decode(&value)?);
                    Ok(Some((value, cookies)))
                } else {
                    Ok(None)
                }
            })
            .instrument(tracing::info_span!("current")),
        sqlx::query_scalar::<_, String>(
            "SELECT value FROM saves ORDER BY cookies_baked_all_time DESC LIMIT 1",
        )
        .fetch_optional(pool)
        .map(|value| -> anyhow::Result<_> {
            if let Some(value) = value? {
                let cookies = cookies_baked_all_time(&cookieclicker_save::decode(&value)?);
                Ok(Some((value, cookies)))
            } else {
                Ok(None)
            }
        })
        .instrument(tracing::info_span!("backup")),
    )
    .await?;
    match (current, backup) {
        (None, Some((backup_value, _))) => operator.lock().await.import_save(&backup_value).await?,
        (Some((_, current_cookies)), Some((value, cookies))) if current_cookies < cookies => {
            operator.lock().await.import_save(&value).await?
        }
        _ => (),
    }
    Ok(())
}

#[tracing::instrument(err, skip(pool, observer))]
async fn backup(pool: &sqlx::SqlitePool, observer: &Observer) -> anyhow::Result<()> {
    let mut interval = tokio::time::interval(Duration::from_secs(60));
    let mut prev = None;
    loop {
        interval.tick().await;
        if let Some(value) = observer.export_save().await? {
            if prev.as_ref() != Some(&value) {
                sqlx::query(concat!(
                    "INSERT OR REPLACE INTO saves",
                    "(timestamp, value, cookies_baked_all_time)",
                    "values (?, ?, ?)",
                ))
                .bind(Utc::now())
                .bind(&value)
                .bind(cookies_baked_all_time(&cookieclicker_save::decode(&value)?))
                .execute(pool)
                .await?;
                prev = Some(value);
            }
        }
    }
}

#[tracing::instrument(err, skip(operator))]
async fn big_cookie(operator: &Mutex<Operator>) -> anyhow::Result<()> {
    let mut interval = tokio::time::interval(Duration::from_millis(100));
    loop {
        interval.tick().await;
        if let Ok(mut operator) = operator.try_lock() {
            operator.try_click(BIG_COOKIE).await?;
        }
    }
}

struct Observer(fantoccini::Client);
impl Observer {
    #[tracing::instrument(err, skip(self))]
    async fn export_save(&self) -> Result<Option<String>, CmdError> {
        if let serde_json::Value::String(save) = self
            .0
            .execute("return localStorage.CookieClickerGame", Vec::new())
            .await?
        {
            Ok(Some(save))
        } else {
            Ok(None)
        }
    }
}

struct Operator(fantoccini::Client);
impl Operator {
    #[tracing::instrument(err, skip(self))]
    async fn click<'a>(&mut self, locator: fantoccini::Locator<'_>) -> Result<(), CmdError> {
        self.0.find(locator).await?.click().await
    }

    #[tracing::instrument(err, skip(self))]
    async fn try_click(&mut self, locator: fantoccini::Locator<'_>) -> Result<bool, CmdError> {
        match async { self.0.find(locator).await?.click().await }.await {
            Ok(()) => Ok(true),
            Err(e)
                if e.is_no_such_element()
                    || e.is_stale_element_reference()
                    || e.is_element_not_interactable() =>
            {
                tracing::warn!(error = e.to_string());
                Ok(false)
            }
            Err(e) => Err(e),
        }
    }

    #[tracing::instrument(err, skip(self))]
    async fn clear(&mut self) -> Result<(), CmdError> {
        self.try_click(LANG_SELECT_ENGLISH).await?;
        self.try_click(PROMPT_OPTION0).await?;
        self.try_click(MENU_CLOSE).await?;
        Ok(())
    }

    #[tracing::instrument(err, ret, skip(self, value))]
    async fn import_save(&mut self, value: &str) -> Result<(), CmdError> {
        self.clear().await?;

        self.click(OPTIONS).await?;
        self.click(IMPORT_SAVE).await?;
        self.0
            .find(IMPORT_SAVE_TEXT)
            .await?
            .send_keys(value)
            .await?;
        self.click(IMPORT_SAVE_LOAD).await?;
        self.click(MENU_CLOSE).await?;

        Ok(())
    }
}

type Locator = fantoccini::Locator<'static>;

const PROMPT_TEXTAREA: Locator = Locator::Id("textareaPrompt");
const PROMPT_OPTION0: Locator = Locator::Id("promptOption0");

const LANG_SELECT_ENGLISH: Locator = Locator::Id("langSelect-EN");

const BIG_COOKIE: Locator = Locator::Id("bigCookie");
const MENU_CLOSE: Locator = Locator::Css("#menu > .menuClose");

const OPTIONS: Locator = Locator::Id("prefsButton");
const IMPORT_SAVE: Locator = Locator::LinkText("Import save");
const IMPORT_SAVE_TEXT: Locator = PROMPT_TEXTAREA;
const IMPORT_SAVE_LOAD: Locator = PROMPT_OPTION0;
