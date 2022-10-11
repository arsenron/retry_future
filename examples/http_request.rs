use async_retry::{AsyncRetry, LinearRetryStrategy, RetryPolicy};
use reqwest::StatusCode;
use std::time::Duration;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let resp = AsyncRetry::new(
        || async {
            let resp = reqwest::get("http://localhost:8084").await?;
            if resp.status() == StatusCode::BAD_REQUEST {
                Err(RetryPolicy::Fail("Cannot recover from bad request"))
            } else if resp.status() == StatusCode::INTERNAL_SERVER_ERROR {
                Err(RetryPolicy::Repeat)
            } else {
                Ok(resp)
            }
        },
        LinearRetryStrategy::default().duration_between_repeats(Duration::from_secs(5)).attempts(1),
    )
    .await?;

    eprintln!("resp = {:#?}", resp);

    Ok(())
}
