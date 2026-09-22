use crate::error::Result;
use reqwest::{Client, Proxy};
use std::time::Duration;

pub fn build_client(proxy: Option<&str>) -> Result<Client> {
    let mut builder = Client::builder()
        .connect_timeout(Duration::from_secs(5))
        .timeout(Duration::from_secs(10))
        .user_agent(concat!("tran/", env!("CARGO_PKG_VERSION")));

    if let Some(proxy_url) = proxy {
        if !proxy_url.trim().is_empty() {
            builder = builder.proxy(Proxy::all(proxy_url)?);
        }
    }

    let client = builder.build()?;
    Ok(client)
}
