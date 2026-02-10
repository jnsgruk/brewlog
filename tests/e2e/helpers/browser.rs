use std::time::Duration;

use thirtyfour::prelude::*;

pub struct BrowserSession {
    pub driver: WebDriver,
    pub base_url: String,
}

impl BrowserSession {
    pub async fn new(base_url: &str) -> WebDriverResult<Self> {
        let port = super::chromedriver::ensure_chromedriver();
        let chromedriver_url = format!("http://localhost:{port}");

        let mut caps = DesiredCapabilities::chrome();
        caps.set_headless()?;
        caps.add_arg("--no-sandbox")?;
        caps.add_arg("--disable-gpu")?;
        caps.add_arg("--disable-dev-shm-usage")?;
        caps.add_arg("--window-size=1280,1024")?;

        let driver = WebDriver::new(&chromedriver_url, caps).await?;
        driver
            .set_implicit_wait_timeout(Duration::from_secs(2))
            .await?;

        Ok(Self {
            driver,
            base_url: base_url.to_string(),
        })
    }

    pub async fn goto(&self, path: &str) -> WebDriverResult<()> {
        self.driver
            .goto(&format!("{}{}", self.base_url, path))
            .await
    }

    pub async fn quit(self) {
        let _ = self.driver.quit().await;
    }
}
