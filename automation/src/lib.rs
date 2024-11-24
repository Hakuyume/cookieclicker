pub mod fantoccini;

use std::{future::Future, time::Duration};

use futures::FutureExt;

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

pub trait WebDriverExt: WebDriver + Sync
where
    Self::Error: Error + Send,
    Self::Element: Send,
{
    fn clear(&self) -> impl Future<Output = Result<(), Self::Error>> + Send + '_ {
        async move {
            self.click_maybe(LANG_SELECT_ENGLISH).await?;
            backoff(move || {
                async move {
                    let element = self.find(BIG_COOKIE).await?;
                    if self.is_displayed(&element).await? {
                        Ok(Some(()))
                    } else {
                        Ok(None)
                    }
                }
                .map(|output: Result<_, Self::Error>| match output {
                    Ok(v) => Ok(v),
                    Err(e) if e.is_not_found() => Ok(None),
                    Err(e) => Err(e),
                })
            })
            .await?;
            self.click_maybe(EXPORT_IMPORT_SAVE_DONE).await?;
            self.click_maybe(MENU_CLOSE).await?;
            Ok(())
        }
    }

    fn export_save(&self) -> impl Future<Output = Result<String, Self::Error>> + Send + '_ {
        async move {
            self.clear().await?;

            self.click_ensure(OPTIONS).await?;
            self.click_ensure(EXPORT_SAVE).await?;
            let save = backoff(move || {
                async move {
                    let element = self.find(EXPORT_IMPORT_SAVE_TEXTAREA).await?;
                    Ok(Some(self.text(&element).await?))
                }
                .map(|output: Result<_, Self::Error>| match output {
                    Ok(v) => Ok(v),
                    Err(e) if e.is_not_found() => Ok(None),
                    Err(e) => Err(e),
                })
            })
            .await?;
            self.click_ensure(EXPORT_IMPORT_SAVE_DONE).await?;
            self.click_ensure(MENU_CLOSE).await?;

            Ok(save)
        }
    }

    fn import_save<'a>(
        &'a self,
        save: &'a str,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send + 'a {
        async move {
            self.clear().await?;

            self.click_ensure(OPTIONS).await?;
            self.click_ensure(IMPORT_SAVE).await?;
            backoff(move || {
                async move {
                    let element = self.find(EXPORT_IMPORT_SAVE_TEXTAREA).await?;
                    self.send_keys(&element, save).await?;
                    Ok(Some(()))
                }
                .map(|output: Result<_, Self::Error>| match output {
                    Ok(v) => Ok(v),
                    Err(e) if e.is_not_found() => Ok(None),
                    Err(e) => Err(e),
                })
            })
            .await?;
            self.click_ensure(EXPORT_IMPORT_SAVE_DONE).await?;
            self.click_ensure(MENU_CLOSE).await?;

            Ok(())
        }
    }

    fn click_ensure<'a>(
        &'a self,
        locator: (LocatorStrategy, &'a str),
    ) -> impl Future<Output = Result<(), Self::Error>> + Send + 'a {
        backoff(move || {
            async move {
                let element = self.find(locator).await?;
                if self.is_displayed(&element).await? {
                    self.click(&element).await?;
                    Ok(Some(()))
                } else {
                    Ok(None)
                }
            }
            .map(|output: Result<_, Self::Error>| match output {
                Ok(v) => Ok(v),
                Err(e) if e.is_not_found() => Ok(None),
                Err(e) => Err(e),
            })
        })
    }

    fn click_maybe<'a>(
        &'a self,
        locator: (LocatorStrategy, &'a str),
    ) -> impl Future<Output = Result<(), Self::Error>> + Send + 'a {
        async move {
            let element = self.find(locator).await?;
            if self.is_displayed(&element).await? {
                self.click(&element).await?;
            }
            Ok(())
        }
        .map(|output: Result<_, Self::Error>| match output {
            Ok(_) => Ok(()),
            Err(e) if e.is_not_found() => Ok(()),
            Err(e) => Err(e),
        })
    }
}
impl<T> WebDriverExt for T
where
    Self: WebDriver + Sync,
    Self::Error: Error + Send,
    Self::Element: Send,
{
}

async fn backoff<F, Fut, T, E>(mut f: F) -> Result<T, E>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<Option<T>, E>>,
{
    loop {
        if let Some(v) = f().await? {
            break Ok(v);
        }
        tokio::time::sleep(Duration::from_millis(250)).await;
    }
}

type Locator = (LocatorStrategy, &'static str);

const LANG_SELECT_ENGLISH: Locator = (LocatorStrategy::CSSSelector, "#langSelect-EN");

const BIG_COOKIE: Locator = (LocatorStrategy::CSSSelector, "#bigCookie");
const MENU_CLOSE: Locator = (LocatorStrategy::CSSSelector, "#menu > .menuClose");

const OPTIONS: Locator = (LocatorStrategy::CSSSelector, "#prefsButton");
const EXPORT_SAVE: Locator = (LocatorStrategy::LinkText, "Export save");
const IMPORT_SAVE: Locator = (LocatorStrategy::LinkText, "Import save");
const EXPORT_IMPORT_SAVE_TEXTAREA: Locator = (LocatorStrategy::CSSSelector, "#textareaPrompt");
const EXPORT_IMPORT_SAVE_DONE: Locator = (LocatorStrategy::CSSSelector, "#promptOption0");
