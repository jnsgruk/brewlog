use std::time::Duration;

use thirtyfour::error::no_such_element;
use thirtyfour::prelude::*;

/// Find the first visible element matching a CSS selector.
/// The `/add` page has duplicate `name` fields across tabbed forms (roaster, roast, etc.)
/// hidden via Datastar `data-show`. This ensures we interact with the active form's fields.
async fn find_visible(driver: &WebDriver, css: &str) -> WebDriverResult<WebElement> {
    let elements = driver.find_all(By::Css(css)).await?;
    for el in elements {
        if el.is_displayed().await.unwrap_or(false) {
            return Ok(el);
        }
    }
    Err(no_such_element(format!(
        "No visible element found for selector: {css}"
    )))
}

/// Fill a text input identified by its `name` attribute.
/// Finds the first *visible* match to avoid hidden duplicate fields on tabbed pages.
pub async fn fill_input(driver: &WebDriver, name: &str, value: &str) -> WebDriverResult<()> {
    let input = find_visible(driver, &format!("input[name='{name}']")).await?;
    input.clear().await?;
    input.send_keys(value).await?;
    Ok(())
}

/// Fill a textarea identified by its `name` attribute.
/// Finds the first *visible* match to avoid hidden duplicate fields on tabbed pages.
pub async fn fill_textarea(driver: &WebDriver, name: &str, value: &str) -> WebDriverResult<()> {
    let textarea = find_visible(driver, &format!("textarea[name='{name}']")).await?;
    textarea.clear().await?;
    textarea.send_keys(value).await?;
    Ok(())
}

/// Select an option from a native `<select>` by name and value.
/// Finds the first *visible* select to avoid hidden duplicates on tabbed pages.
pub async fn select_option(driver: &WebDriver, name: &str, value: &str) -> WebDriverResult<()> {
    let select = find_visible(driver, &format!("select[name='{name}']")).await?;
    let option = select
        .find(By::Css(&format!("option[value='{value}']")))
        .await?;
    option.click().await?;
    Ok(())
}

/// Interact with a `<searchable-select>` web component:
/// 1. Find the search input (role=combobox) inside the component
/// 2. Type to filter options
/// 3. Click the first visible option button
pub async fn select_searchable(
    driver: &WebDriver,
    name: &str,
    search_text: &str,
) -> WebDriverResult<()> {
    let component = driver
        .find(By::Css(&format!("searchable-select[name='{name}']")))
        .await?;

    let search_input = component.find(By::Css("input[role='combobox']")).await?;
    search_input.send_keys(search_text).await?;

    // Wait for the dropdown to appear and options to filter
    tokio::time::sleep(Duration::from_millis(300)).await;

    // Click the first visible option
    let options = component.find_all(By::Css("button[role='option']")).await?;
    for option in options {
        if option.is_displayed().await.unwrap_or(false) {
            option.click().await?;
            return Ok(());
        }
    }

    Err(no_such_element(format!(
        "No visible option found in searchable-select[name='{name}'] after typing '{search_text}'"
    )))
}

/// Click the visible submit button on the page.
pub async fn submit_visible_form(driver: &WebDriver) -> WebDriverResult<()> {
    let buttons = driver.find_all(By::Css("button[type='submit']")).await?;
    for button in buttons {
        if button.is_displayed().await.unwrap_or(false) {
            button.click().await?;
            return Ok(());
        }
    }
    Err(no_such_element(
        "No visible submit button found".to_string(),
    ))
}
