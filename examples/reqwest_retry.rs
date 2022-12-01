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
    let get = client.get("https://google.com").retry(ExponentialRetryStrategy::default()).await?;
    eprintln!("get = {:#?}", get);

    let post = client
        .post("http://example.com")
        .body(String::from("hello!"))
        .retry(LinearRetryStrategy::default())
        .await?;
    eprintln!("post = {:#?}", post);

    Ok(())
}

trait Retry {
    fn retry<RS: RetryStrategy + 'static>(
        self,
        retry_strategy: RS,
    ) -> Pin<Box<dyn Future<Output = Result<Response, RetryError<Response>>>>>;
}

impl Retry for RequestBuilder {
    fn retry<RS: RetryStrategy + 'static>(
        self,
        retry_strategy: RS,
    ) -> Pin<Box<dyn Future<Output = Result<Response, RetryError<Response>>>>> {
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
