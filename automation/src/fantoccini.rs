use fantoccini::elements::Element;
use fantoccini::error::{CmdError, NewSessionError};
use fantoccini::{Client, ClientBuilder, Locator};
use std::future::Future;
use std::io;

pub async fn with<F, Fut, T, E>(webdriver: &str, f: F) -> Result<T, E>
where
    F: FnOnce(Client) -> Fut,
    Fut: Future<Output = Result<T, E>>,
    E: From<io::Error> + From<NewSessionError>,
{
    let client = ClientBuilder::rustls()?.connect(webdriver).await?;
    let output = f(client.clone()).await;
    let _ = client.close().await;
    output
}

impl crate::WebDriver for Client {
    type Error = CmdError;
    type Element = Element;

    fn find<'a>(
        &'a self,
        locator: (crate::LocatorStrategy, &'a str),
    ) -> impl Future<Output = Result<Self::Element, Self::Error>> + Send + 'a {
        self.find(map_locator(locator))
    }

    fn text<'a>(
        &'a self,
        element: &'a Self::Element,
    ) -> impl Future<Output = Result<String, Self::Error>> + Send + 'a {
        element.text()
    }

    fn click<'a>(
        &'a self,
        element: &'a Self::Element,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send + 'a {
        element.click()
    }

    fn send_keys<'a>(
        &'a self,
        element: &'a Self::Element,
        text: &'a str,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send + 'a {
        element.send_keys(text)
    }

    fn is_displayed<'a>(
        &'a self,
        element: &'a Self::Element,
    ) -> impl Future<Output = Result<bool, Self::Error>> + Send + 'a {
        element.is_displayed()
    }
}

impl crate::Error for CmdError {
    fn is_not_found(&self) -> bool {
        self.is_no_such_element() || self.is_stale_element_reference()
    }
}

fn map_locator((using, value): (crate::LocatorStrategy, &str)) -> Locator<'_> {
    match using {
        crate::LocatorStrategy::CSSSelector => Locator::Css(value),
        crate::LocatorStrategy::LinkText => Locator::LinkText(value),
    }
}
