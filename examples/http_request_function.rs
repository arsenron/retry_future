use async_retry::{AsyncRetry, LinearRetryStrategy, RetryPolicy};
use reqwest::{IntoUrl, Response, StatusCode};
use std::time::Duration;

async fn make_request<T: IntoUrl>(url: T) -> Result<Response, RetryPolicy<String>> {
    let resp = reqwest::get(url).await?;
    if resp.status() == StatusCode::BAD_REQUEST {
        Err(RetryPolicy::Fail(String::from("Cannot recover from bad request")))
    } else if resp.status() == StatusCode::INTERNAL_SERVER_ERROR {
        Err(RetryPolicy::Repeat(None))
    } else {
        Ok(resp)
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let url = "http://localhost:8084";
    let resp = AsyncRetry::new(
        || make_request(url),
        LinearRetryStrategy::default()
            .delay_between_repeats(Duration::from_secs(5))
            .max_attempts(2),
    )
    .await?;

    eprintln!("resp = {:#?}", resp);

    Ok(())
}
