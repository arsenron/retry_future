use async_retry::{AsyncRetry, ExponentialRetryStrategy, RetryPolicy};
use reqwest::StatusCode;
use std::time::Duration;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // You can return any error which you want inside RetryPolicy::Fail(E).
    // In example below we opt for String
    let resp = AsyncRetry::new(
        || async {
            let resp = reqwest::get("http://localhost:8085").await?;
            match resp.status() {
                StatusCode::OK => Ok(resp),
                StatusCode::BAD_REQUEST | StatusCode::FORBIDDEN => RetryPolicy::fail(format!(
                    "Cannot recover from these kind of errors ._. - {resp:?}"
                )),
                StatusCode::UNAUTHORIZED => {
                    // What if authorization server lies us?! Repeat it to be convinced
                    let response_text = resp.text().await?;
                    RetryPolicy::repeat(anyhow::anyhow!(response_text))
                }
                StatusCode::INTERNAL_SERVER_ERROR => RetryPolicy::repeat_without_context(),
                e => RetryPolicy::fail(format!("Some unusual status code here: {e:?}")),
            }
        },
        ExponentialRetryStrategy::default()
            .max_attempts(5)
            .initial_delay(Duration::from_millis(100)),
    )
    .await?;

    eprintln!("resp = {:#?}", resp);

    Ok(())
}
