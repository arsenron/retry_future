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
                StatusCode::BAD_REQUEST | StatusCode::FORBIDDEN => Err(RetryPolicy::Fail(
                    String::from("Cannot recover from these kind of errors ._."),
                )),
                StatusCode::INTERNAL_SERVER_ERROR => Err(RetryPolicy::Repeat(None)),
                // What if authorization server lies us?! Repeat it to be convinced
                StatusCode::UNAUTHORIZED => {
                    // Get error message as debug info
                    let maybe_response_text = resp.text().await.ok().map(anyhow::Error::msg);
                    Err(RetryPolicy::Repeat(maybe_response_text))
                }
                e => Err(RetryPolicy::Fail(format!("Some unusual error here: {e:?}"))),
            }
        },
        ExponentialRetryStrategy::default()
            .max_attempts(2)
            .initial_delay(Duration::from_millis(100)),
    )
    .await?;

    eprintln!("resp = {:#?}", resp);

    Ok(())
}
