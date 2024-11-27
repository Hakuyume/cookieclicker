use crate::locator;
use chrono::Utc;
use fantoccini::Locator;
use tokio::sync::Mutex;

pub fn split(client: fantoccini::Client) -> (Observer, Mutex<Operator>) {
    (
        Observer(client.clone()),
        Mutex::new(Operator(client.clone())),
    )
}

pub struct Observer(fantoccini::Client);
pub struct Operator(fantoccini::Client);

impl Observer {
    #[tracing::instrument(err, ret, skip(self))]
    pub async fn fetch_save(&self) -> anyhow::Result<Option<crate::save::Save>> {
        if let serde_json::Value::String(code) = self
            .0
            .execute("return localStorage.CookieClickerGame", Vec::new())
            .await?
        {
            Ok(Some(crate::save::Save::new(Utc::now(), code)?))
        } else {
            Ok(None)
        }
    }
}

impl Operator {
    #[tracing::instrument(err, skip(self))]
    pub async fn click(&mut self, locator: Locator<'_>) -> anyhow::Result<()> {
        use fantoccini::error::CmdError;
        use fantoccini::error::ErrorStatus::*;

        backoff::future::retry(backoff::ExponentialBackoff::default(), || async {
            match async { self.0.find(locator).await?.click().await }.await {
                Ok(()) => Ok(()),
                Err(CmdError::Standard(e))
                    if matches!(
                        e.error,
                        ElementClickIntercepted
                            | ElementNotInteractable
                            | NoSuchElement
                            | StaleElementReference
                    ) =>
                {
                    tracing::warn!(error = e.to_string());
                    Err(backoff::Error::transient(CmdError::Standard(e)))
                }
                Err(e) => Err(backoff::Error::permanent(e)),
            }
        })
        .await?;
        Ok(())
    }

    #[tracing::instrument(err, skip(self))]
    pub async fn try_click(&mut self, locator: Locator<'_>) -> anyhow::Result<bool> {
        use fantoccini::error::CmdError;
        use fantoccini::error::ErrorStatus::*;

        match async { self.0.find(locator).await?.click().await }.await {
            Ok(()) => Ok(true),
            Err(CmdError::Standard(e))
                if matches!(
                    e.error,
                    ElementClickIntercepted
                        | ElementNotInteractable
                        | NoSuchElement
                        | StaleElementReference
                ) =>
            {
                tracing::warn!(error = e.to_string());
                Ok(false)
            }
            Err(e) => Err(e.into()),
        }
    }

    #[tracing::instrument(err, ret, skip(self))]
    pub async fn import_save(&mut self, save: &crate::save::Save) -> anyhow::Result<()> {
        self.clear().await?;

        self.click(locator::OPTIONS).await?;
        self.click(locator::IMPORT_SAVE).await?;
        self.0
            .find(locator::IMPORT_SAVE_TEXT)
            .await?
            .send_keys(save.code())
            .await?;
        self.click(locator::IMPORT_SAVE_LOAD).await?;
        self.click(locator::MENU_CLOSE).await?;

        Ok(())
    }

    #[tracing::instrument(err, skip(self))]
    async fn clear(&mut self) -> anyhow::Result<()> {
        self.try_click(locator::LANG_SELECT_ENGLISH).await?;
        self.try_click(locator::PROMPT_OPTION0).await?;
        self.try_click(locator::MENU_CLOSE).await?;
        Ok(())
    }
}
