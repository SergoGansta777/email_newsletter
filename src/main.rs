use std::net::{Ipv4Addr, SocketAddr};

use newsletter_deliverer::run;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let addr = SocketAddr::from((Ipv4Addr::UNSPECIFIED, 8080));
    let listener = TcpListener::bind(addr)
        .await
        .expect("Failed to bind to addr");

    run(listener).await.unwrap().await
}
