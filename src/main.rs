use anyhow::Context;
use newsletter_deliverer::{
    configuration::{get_configuration, DatabaseSettings},
    run,
};
use sqlx::postgres::PgPoolOptions;
use std::net::{Ipv4Addr, SocketAddr};
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

    let connection_pool = get_connection_pool(&configuration.database);
    let email_client = configuration.email_client.client();

    run(listener, connection_pool, email_client)
        .await
        .unwrap()
        .await
        .context("Error running HTTP server")?;
    Ok(())
}

pub fn get_connection_pool(configuration: &DatabaseSettings) -> PgPool {
    PgPoolOptions::new().connect_lazy_with(configuration.with_db())
}
