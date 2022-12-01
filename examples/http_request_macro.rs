use reqwest::StatusCode;
use retry_future::{fail, retry, ExponentialRetryStrategy, RetryFuture};
use std::time::Duration;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let resp = RetryFuture::new(
        || async {
            let resp = reqwest::get("http://localhost:8085").await?;
            match resp.status() {
                StatusCode::OK => Ok(resp),
                StatusCode::BAD_REQUEST | StatusCode::FORBIDDEN => {
                    fail!(String::from("Cannot recover from these kind of errors ._."))
                }
                StatusCode::INTERNAL_SERVER_ERROR => retry!(),
                StatusCode::UNAUTHORIZED => {
                    retry!(resp.text().await?)
                }
                e => fail!(format!("Some unusual status code here: {e:?}")),
            }
        },
        ExponentialRetryStrategy::default()
            .max_attempts(5)
            .initial_delay(Duration::from_millis(100))
            .retry_early_returned_errors(false), // abort on early errors, i.e. "?"
    )
    .await?;

    eprintln!("resp = {:#?}", resp);

    Ok(())
}
