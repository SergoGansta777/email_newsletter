use std::net::{Ipv4Addr, SocketAddr};

use anyhow::Context;
use newsletter_deliverer::{configuration::get_configuration, run};
use sqlx::postgres::PgPoolOptions;
use tokio::net::TcpListener;
use tracing_subscriber::fmt::format::FmtSpan;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();

    tracing_subscriber::fmt()
        .with_span_events(FmtSpan::FULL)
        .init();

    let configuration = get_configuration().expect("Failed to read configuration.");
    let addr = SocketAddr::from((Ipv4Addr::UNSPECIFIED, configuration.application_port));
    let listener = TcpListener::bind(addr)
        .await
        .expect("Failed to bind to addr");

    let connection_pool = PgPoolOptions::new()
        .max_connections(50)
        .connect(&configuration.database.connection_string())
        .await
        .context("Could not connect to database url")?;

    run(listener, connection_pool)
        .await
        .unwrap()
        .await
        .context("Error running HTTP server")?;
    Ok(())
}
