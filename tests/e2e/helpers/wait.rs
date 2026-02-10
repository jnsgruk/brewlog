use std::time::{Duration, Instant};

use thirtyfour::prelude::*;

const DEFAULT_TIMEOUT: Duration = Duration::from_secs(10);
const POLL_INTERVAL: Duration = Duration::from_millis(100);

/// Wait until an element matching the CSS selector is present in the DOM.
pub async fn wait_for_element(driver: &WebDriver, selector: &str) -> WebDriverResult<WebElement> {
    driver
        .query(By::Css(selector))
        .wait(DEFAULT_TIMEOUT, POLL_INTERVAL)
        .first()
        .await
}

/// Wait until an element is both present AND displayed.
/// Needed because Datastar uses `style="display:none"` + `data-show`.
pub async fn wait_for_visible(driver: &WebDriver, selector: &str) -> WebDriverResult<WebElement> {
    let start = Instant::now();
    loop {
        if let Ok(el) = driver.find(By::Css(selector)).await {
            if el.is_displayed().await.unwrap_or(false) {
                return Ok(el);
            }
        }
        if start.elapsed() > DEFAULT_TIMEOUT {
            return Err(WebDriverError::Timeout(format!(
                "Element '{selector}' not visible within {DEFAULT_TIMEOUT:?}",
            )));
        }
        tokio::time::sleep(POLL_INTERVAL).await;
    }
}

/// Wait until an element with specific text content appears.
pub async fn wait_for_text(
    driver: &WebDriver,
    selector: &str,
    expected_text: &str,
) -> WebDriverResult<WebElement> {
    let start = Instant::now();
    loop {
        if let Ok(el) = driver.find(By::Css(selector)).await {
            if let Ok(text) = el.text().await {
                if text.contains(expected_text) {
                    return Ok(el);
                }
            }
        }
        if start.elapsed() > DEFAULT_TIMEOUT {
            return Err(WebDriverError::Timeout(format!(
                "Element '{selector}' with text '{expected_text}' not found within {DEFAULT_TIMEOUT:?}",
            )));
        }
        tokio::time::sleep(POLL_INTERVAL).await;
    }
}

/// Wait until no element matches the selector.
pub async fn wait_for_element_removed(driver: &WebDriver, selector: &str) -> WebDriverResult<()> {
    let start = Instant::now();
    loop {
        if driver.find(By::Css(selector)).await.is_err() {
            return Ok(());
        }
        if start.elapsed() > DEFAULT_TIMEOUT {
            return Err(WebDriverError::Timeout(format!(
                "Element '{selector}' still present after {DEFAULT_TIMEOUT:?}",
            )));
        }
        tokio::time::sleep(POLL_INTERVAL).await;
    }
}

/// Wait until the current URL contains a specific substring.
pub async fn wait_for_url_contains(driver: &WebDriver, substring: &str) -> WebDriverResult<()> {
    let start = Instant::now();
    while start.elapsed() < DEFAULT_TIMEOUT {
        let url = driver.current_url().await?;
        if url.as_str().contains(substring) {
            return Ok(());
        }
        tokio::time::sleep(POLL_INTERVAL).await;
    }
    Err(WebDriverError::Timeout(format!(
        "URL did not contain '{substring}' within {DEFAULT_TIMEOUT:?}",
    )))
}
