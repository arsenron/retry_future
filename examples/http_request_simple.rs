use async_retry::{AsyncRetry, LinearRetryStrategy, RetryPolicy};
use std::time::Duration;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let text = AsyncRetry::new(
        || async { Ok::<_, RetryPolicy>(reqwest::get("http://localhost:8084").await?.text().await?) },
        LinearRetryStrategy::default()
            .max_attempts(10)
            .delay_between_repeats(Duration::from_millis(100)),
    )
    .await?;

    eprintln!("text = {:#?}", text);

    Ok(())
}
