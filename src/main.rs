use newsletter_deliverer::run;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    run().await.unwrap().await
}
