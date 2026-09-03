use std::time::Duration;
use reqwest::{Client, Proxy};
use crate::error::Result;

pub fn build_client(proxy: Option<&str>) -> Result<Client> {
    let mut builder = Client::builder()
        .timeout(Duration::from_secs(10))
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/122.0.0.0 Safari/537.36");

    if let Some(proxy_url) = proxy {
        if !proxy_url.trim().is_empty() {
            builder = builder.proxy(Proxy::all(proxy_url)?);
        }
    }

    let client = builder.build()?;
    Ok(client)
}
