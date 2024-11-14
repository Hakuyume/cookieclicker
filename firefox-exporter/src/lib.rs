use futures::{Stream, TryStreamExt};
use std::future;
use std::io;
use std::path::PathBuf;
use std::string::FromUtf8Error;
use std::time::Duration;
use tracing_futures::Instrument;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Ini(#[from] ini::ParseError),
    #[error(transparent)]
    Io(#[from] io::Error),
    #[error(transparent)]
    Snap(#[from] snap::Error),
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
    #[error(transparent)]
    Utf8(#[from] FromUtf8Error),

    #[error("home directory not found")]
    HomeDirectoryNotFound,
    #[error("profile not found")]
    ProfileNotFound,
    #[error("unknown compression_type = {0}")]
    UnknownCompressionType(i64),
}

pub struct FirefoxExporter {
    database: PathBuf,
    decoder: snap::raw::Decoder,
}

impl FirefoxExporter {
    #[tracing::instrument(err)]
    pub async fn new(profile: &str) -> Result<Self, Error> {
        let firefox = dirs::home_dir()
            .ok_or_else(|| Error::HomeDirectoryNotFound)?
            .join(".mozilla")
            .join("firefox");
        tracing::info!(?firefox);
        let profiles = ini::Ini::load_from_str(
            &tokio::fs::read_to_string(firefox.join("profiles.ini")).await?,
        )?;
        let path = profiles
            .into_iter()
            .find_map(|(_, mut properties)| {
                if properties.get("Name") == Some(profile) {
                    properties.remove("Path")
                } else {
                    None
                }
            })
            .ok_or_else(|| Error::ProfileNotFound)?;
        let database = firefox
            .join(path)
            .join("storage")
            .join("default")
            .join("https+++orteil.dashnet.org")
            .join("ls")
            .join("data.sqlite");
        tracing::info!(?database);
        Ok(Self {
            database,
            decoder: snap::raw::Decoder::new(),
        })
    }

    #[tracing::instrument(err, ret, skip_all)]
    pub async fn export(&mut self) -> Result<String, Error> {
        let pool = sqlx::SqlitePool::connect_with(
            sqlx::sqlite::SqliteConnectOptions::new()
                .filename(&self.database)
                .immutable(true),
        )
        .await?;
        let (compression_type, value) = sqlx::query_as::<_, (i64, Vec<u8>)>(
            "SELECT compression_type, value FROM data WHERE key = 'CookieClickerGame'",
        )
        .fetch_one(&pool)
        .await?;
        let value = match compression_type {
            0 => value,
            1 => self.decoder.decompress_vec(&value)?,
            _ => return Err(Error::UnknownCompressionType(compression_type)),
        };
        Ok(String::from_utf8(value)?)
    }

    pub fn watch(
        &mut self,
        interval: Duration,
    ) -> impl Stream<Item = Result<String, Error>> + Send + '_ {
        let span = tracing::info_span!("watch", interval = ?interval);
        futures::stream::try_unfold(
            (self, tokio::time::interval(interval)),
            |(this, mut interval)| async move {
                interval.tick().await;
                let value = this.export().await?;
                Ok(Some((value, (this, interval))))
            },
        )
        .try_filter({
            let mut prev = None;
            move |value| future::ready(prev.replace(value.clone()).as_ref() != Some(value))
        })
        .instrument(span)
    }
}
