use anyhow::anyhow;
use reqwest::{RequestBuilder, Response};
use retry_future::{
    ExponentialRetryStrategy, LinearRetryStrategy, RetryError, RetryFuture, RetryPolicy,
    RetryStrategy,
};
use std::future::Future;
use std::pin::Pin;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = reqwest::Client::new();
    // You can opt for your defaults
    let get = client
        .get("https://google.com")
        .with_retry_strategy(ExponentialRetryStrategy::default())
        .await?;
    eprintln!("get = {:#?}", get);

    let post = client
        .post("http://example.com")
        .body(String::from("hello!"))
        .with_retry_strategy(LinearRetryStrategy::default())
        .await?;
    eprintln!("post = {:#?}", post);

    Ok(())
}

trait WithRetryStrategy {
    type Ok;
    type Err;

    fn with_retry_strategy<RS: RetryStrategy + 'static>(
        self,
        retry_strategy: RS,
    ) -> Pin<Box<dyn Future<Output = Result<Self::Ok, RetryError<Self::Err>>>>>;
}

impl WithRetryStrategy for RequestBuilder {
    type Ok = Response;
    type Err = Response;

    fn with_retry_strategy<RS: RetryStrategy + 'static>(
        self,
        retry_strategy: RS,
    ) -> Pin<Box<dyn Future<Output = Result<Self::Ok, RetryError<Self::Err>>>>> {
        Box::pin(async move {
            RetryFuture::new(
                || async {
                    let resp =
                        self.try_clone().ok_or(anyhow!("RequestBody is a stream!"))?.send().await?;
                    if resp.status().is_success() {
                        Ok(resp)
                    } else if resp.status().is_server_error() {
                        Err(RetryPolicy::Retry(None))
                    } else {
                        Err(RetryPolicy::Fail(resp))
                    }
                },
                retry_strategy,
            )
            .await
        })
    }
}
