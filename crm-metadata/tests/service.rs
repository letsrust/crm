use std::{net::SocketAddr, time::Duration};

use anyhow::Result;
use crm_metadata::{
    pb::{metadata_client::MetadataClient, MaterializeRequest},
    AppConfig, MetadataService,
};
use futures::StreamExt;
use tokio::time::sleep;
use tonic::{transport::Server, Request};

#[tokio::test]
async fn test_metadata_should_work() -> Result<()> {
    let addr = start_server().await?;

    let mut client = MetadataClient::connect(format!("http://{addr}")).await?;

    let stream = tokio_stream::iter(vec![
        MaterializeRequest { id: 1 },
        MaterializeRequest { id: 2 },
        MaterializeRequest { id: 3 },
    ]);

    let response = client.materialize(Request::new(stream)).await?.into_inner();
    let result = response
        .then(|r| async { r.unwrap() })
        .collect::<Vec<_>>()
        .await;
    assert_eq!(result.len(), 3);
    Ok(())
}

async fn start_server() -> Result<SocketAddr> {
    let config = AppConfig::load()?;
    let addr = format!("[::1]:{}", config.server.port).parse()?;

    let svc = MetadataService::new(config).into_server();
    tokio::spawn(async move {
        Server::builder()
            .add_service(svc)
            .serve(addr)
            .await
            .unwrap();
    });
    sleep(Duration::from_micros(1)).await;

    Ok(addr)
}
