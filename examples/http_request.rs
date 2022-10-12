use async_retry::{AsyncRetry, ExponentialRetryStrategy, RetryPolicy};
use reqwest::StatusCode;
use std::time::Duration;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let resp = AsyncRetry::new(
        || async {
            let resp = reqwest::get("http://localhost:8085").await?;
            if resp.status() == StatusCode::BAD_REQUEST {
                Err(RetryPolicy::Fail("Cannot recover from bad request"))
            } else if resp.status() == StatusCode::INTERNAL_SERVER_ERROR {
                Err(RetryPolicy::Repeat)
            } else {
                Ok(resp)
            }
        },
        ExponentialRetryStrategy { max_attempts: 3, starts_with: Duration::from_millis(100) },
    )
    .await?;

    eprintln!("resp = {:#?}", resp);

    Ok(())
}
