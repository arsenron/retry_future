# Retry Future

The main purpose of the crate is to retry `Futures` which may contain complex scenarios such as
not only handling errors but anything that should be retried. This may include 
retrying 500's errors from http requests or retrying something like "pseudo" successes from
grpc requests.

For examples, please check `examples/` dir, but here is one:
```rust
// imports...
use retry_future::{
    RetryFuture, RetryPolicy, ExponentialRetryStrategy
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let resp = RetryFuture::new(
        || async {
            let resp = reqwest::get("http://localhost:8080").await?;
            match resp.status() {
                StatusCode::OK => Ok(resp),
                StatusCode::BAD_REQUEST | StatusCode::FORBIDDEN => Err(RetryPolicy::Fail(
                    String::from("Cannot recover from these kind of errors ._."),
                )),
                StatusCode::INTERNAL_SERVER_ERROR => Err(RetryPolicy::Retry(None)),
                StatusCode::UNAUTHORIZED => {
                    // What if authorization server lies us?! Retry it to be convinced
                    let maybe_response_text = resp.text().await.ok().map(anyhow::Error::msg);  // debug info
                    Err(RetryPolicy::Retry(maybe_response_text))
                }
                _ => Err(RetryPolicy::Fail(format!("Some unusual response here: {resp:?}"))),
            }
        },
        ExponentialRetryStrategy::new()
            .max_attempts(5)
            .initial_delay(Duration::from_millis(100)),
    )
    .await?;

    eprintln!("resp = {:#?}", resp);

    Ok(())
}

```



### License

Licensed under either of

 * Apache License, Version 2.0
   ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license
   ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.