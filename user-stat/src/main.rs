use anyhow::Result;
use tonic::transport::Server;
use tracing::{info, level_filters::LevelFilter};
use tracing_subscriber::{fmt::Layer, layer::SubscriberExt, util::SubscriberInitExt, Layer as _};
use user_stat::{AppConfig, UserStatsService};

#[tokio::main]
async fn main() -> Result<()> {
    let layer = Layer::new().with_filter(LevelFilter::INFO);
    tracing_subscriber::registry().with(layer).init();

    let config = AppConfig::load().expect("failed to load config");
    // info!("config: {:?}", config);

    let addr = config.server.port;
    let addr = format!("[::1]:{}", addr).parse().unwrap();
    info!("user-stats service listening on {}", addr);

    let user_stats_srv = UserStatsService::new(config).await.into_server();

    Server::builder()
        .add_service(user_stats_srv)
        .serve(addr)
        .await?;
    Ok(())
}
