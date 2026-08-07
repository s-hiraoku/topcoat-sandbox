mod app;
mod herdr;

use herdr::HerdrClient;

#[tokio::main]
async fn main() {
    let binary = std::env::var("HERDR_BIN").unwrap_or_else(|_| "herdr".to_owned());
    topcoat::start(app::router(HerdrClient::new(binary)))
        .await
        .unwrap();
}
