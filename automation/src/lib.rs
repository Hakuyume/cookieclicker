pub mod fantoccini;

use std::future::Future;
use std::time::Duration;

pub trait WebDriver {
    type Error;
    type Element;
    fn find<'a>(
        &'a self,
        locator: (LocatorStrategy, &'a str),
    ) -> impl Future<Output = Result<Self::Element, Self::Error>> + Send + 'a;
    fn text<'a>(
        &'a self,
        element: &'a Self::Element,
    ) -> impl Future<Output = Result<String, Self::Error>> + Send + 'a;
    fn click<'a>(
        &'a self,
        element: &'a Self::Element,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send + 'a;
    fn send_keys<'a>(
        &'a self,
        element: &'a Self::Element,
        text: &'a str,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send + 'a;
    fn is_displayed<'a>(
        &'a self,
        element: &'a Self::Element,
    ) -> impl Future<Output = Result<bool, Self::Error>> + Send + 'a;
}

pub trait Error {
    fn is_not_found(&self) -> bool;
}

#[derive(Clone, Copy, Debug)]
pub enum LocatorStrategy {
    CSSSelector,
    LinkText,
}

pub struct Client<T>(T);

impl<T> Client<T> {
    pub fn new(inner: T) -> Self {
        Self(inner)
    }
}

impl<T> Client<T>
where
    T: WebDriver + Send + Sync,
    T::Error: Error + Send,
    T::Element: Send,
{
    pub async fn export_save(&mut self) -> Result<String, T::Error> {
        self.clear().await?;

        self.click_ensure(OPTIONS).await?;
        self.click_ensure(EXPORT_SAVE).await?;
        let save = self
            .retry(|this| async move {
                this.retry_finish(
                    async {
                        let element = this.0.find(EXPORT_SAVE_TEXT).await?;
                        Ok(Some(this.0.text(&element).await?))
                    }
                    .await,
                )
            })
            .await?;
        self.click_ensure(EXPORT_SAVE_ALL_DONE).await?;
        self.click_ensure(MENU_CLOSE).await?;

        Ok(save)
    }

    pub async fn import_save(&mut self, save: &str) -> Result<(), T::Error> {
        self.clear().await?;

        self.click_ensure(OPTIONS).await?;
        self.click_ensure(IMPORT_SAVE).await?;
        self.retry(|this| async move {
            this.retry_finish(
                async {
                    let element = this.0.find(IMPORT_SAVE_TEXT).await?;
                    this.0.send_keys(&element, save).await?;
                    Ok(Some(()))
                }
                .await,
            )
        })
        .await?;
        self.click_ensure(IMPORT_SAVE_LOAD).await?;
        self.click_ensure(MENU_CLOSE).await?;

        Ok(())
    }

    pub async fn buy_all_upgrades(&mut self) -> Result<bool, T::Error> {
        self.clear().await?;
        self.click_maybe(STORE_BUY_ALL_UPGRADES).await
    }

    async fn clear(&mut self) -> Result<(), T::Error> {
        self.click_maybe(LANG_SELECT_ENGLISH).await?;
        self.retry(|this| async move {
            this.retry_finish(
                async {
                    let element = this.0.find(BIG_COOKIE).await?;
                    if this.0.is_displayed(&element).await? {
                        Ok(Some(()))
                    } else {
                        Ok(None)
                    }
                }
                .await,
            )
        })
        .await?;
        self.click_maybe(EXPORT_SAVE_ALL_DONE).await?;
        self.click_maybe(MENU_CLOSE).await?;
        Ok(())
    }

    async fn click_ensure(&mut self, locator: (LocatorStrategy, &str)) -> Result<(), T::Error> {
        self.retry(|this| async move {
            this.retry_finish(
                async {
                    let element = this.0.find(locator).await?;
                    if this.0.is_displayed(&element).await? {
                        this.0.click(&element).await?;
                        Ok(Some(()))
                    } else {
                        Ok(None)
                    }
                }
                .await,
            )
        })
        .await
    }

    async fn click_maybe(&mut self, locator: (LocatorStrategy, &str)) -> Result<bool, T::Error> {
        let output: Result<_, T::Error> = async {
            let element = self.0.find(locator).await?;
            if self.0.is_displayed(&element).await? {
                self.0.click(&element).await?;
            }
            Ok(())
        }
        .await;
        match output {
            Ok(_) => Ok(true),
            Err(e) if e.is_not_found() => Ok(false),
            Err(e) => Err(e),
        }
    }

    async fn retry<'a, F, Fut, U>(&'a mut self, mut f: F) -> Result<U, T::Error>
    where
        F: FnMut(&'a mut Self) -> Fut,
        Fut: Future<Output = Result<(Option<U>, &'a mut Self), T::Error>>,
    {
        let mut this = self;
        loop {
            let (v, next) = f(this).await?;
            if let Some(v) = v {
                break Ok(v);
            }
            this = next;
            tokio::time::sleep(Duration::from_millis(250)).await;
        }
    }

    fn retry_finish<U>(
        &mut self,
        output: Result<Option<U>, T::Error>,
    ) -> Result<(Option<U>, &mut Self), T::Error> {
        match output {
            Ok(v) => Ok((v, self)),
            Err(e) if e.is_not_found() => Ok((None, self)),
            Err(e) => Err(e),
        }
    }
}

type Locator = (LocatorStrategy, &'static str);

const PROMPT_TEXTAREA: Locator = (LocatorStrategy::CSSSelector, "#textareaPrompt");
const PROMPT_OPTION0: Locator = (LocatorStrategy::CSSSelector, "#promptOption0");

const LANG_SELECT_ENGLISH: Locator = (LocatorStrategy::CSSSelector, "#langSelect-EN");

const BIG_COOKIE: Locator = (LocatorStrategy::CSSSelector, "#bigCookie");
const MENU_CLOSE: Locator = (LocatorStrategy::CSSSelector, "#menu > .menuClose");

const OPTIONS: Locator = (LocatorStrategy::CSSSelector, "#prefsButton");
const EXPORT_SAVE: Locator = (LocatorStrategy::LinkText, "Export save");
const EXPORT_SAVE_TEXT: Locator = PROMPT_TEXTAREA;
const EXPORT_SAVE_ALL_DONE: Locator = PROMPT_OPTION0;
const IMPORT_SAVE: Locator = (LocatorStrategy::LinkText, "Import save");
const IMPORT_SAVE_TEXT: Locator = PROMPT_TEXTAREA;
const IMPORT_SAVE_LOAD: Locator = PROMPT_OPTION0;

// const LEGACY: Locator = (LocatorStrategy::CSSSelector, "#legacyButton");
// const LEGACY_ACEND: Locator = PROMPT_OPTION0;

const STORE_BUY_ALL_UPGRADES: Locator = (LocatorStrategy::CSSSelector, "#storeBuyAllButton");
// const STORE_BULK1: Locator = (LocatorStrategy::CSSSelector, "#storeBulk1");

// const REINCARNATE: Locator = (LocatorStrategy::CSSSelector, "#ascendButton");
// const REINCARNATE_YES: Locator = PROMPT_OPTION0;
