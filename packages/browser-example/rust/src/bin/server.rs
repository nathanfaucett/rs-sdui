use std::{env, error::Error, net::SocketAddr};

use sdui_browser_example::server;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let address = env::var("SERVER_WS_ADDR").unwrap_or_else(|_| "127.0.0.1:8081".to_owned());
    server::run(address.parse::<SocketAddr>()?).await?;
    Ok(())
}
