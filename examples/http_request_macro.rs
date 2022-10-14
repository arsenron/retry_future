use async_retry::{fail, repeat, AsyncRetry, ExponentialRetryStrategy, RetryPolicy};
use reqwest::StatusCode;
use std::time::Duration;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let resp = AsyncRetry::new(
        || async {
            let resp = reqwest::get("http://localhost:8085").await?;
            match resp.status() {
                StatusCode::OK => Ok(resp),
                StatusCode::BAD_REQUEST | StatusCode::FORBIDDEN => {
                    fail!(String::from("Cannot recover from these kind of errors ._."))
                }
                StatusCode::INTERNAL_SERVER_ERROR => repeat!(),
                StatusCode::UNAUTHORIZED => {
                    let response_text = resp.text().await?;
                    repeat!(response_text)
                }
                e => fail!(format!("Some unusual statusc code here: {e:?}")),
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
