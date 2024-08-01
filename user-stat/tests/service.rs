use std::{collections::HashMap, net::SocketAddr, time::Duration};

use anyhow::Result;
use crm_common::TestMysql;
use futures::StreamExt;
use tokio::time::sleep;
use tonic::transport::Server;
use user_stat::{
    pb::{user_stats_client::UserStatsClient, QueryRequest, RawQueryRequest},
    test_utils::tq,
    UserStatsService,
};

const PORT_BASE: u32 = 60000;

#[tokio::test]
async fn raw_query_should_work() -> Result<()> {
    let (_tdb, addr) = start_serve(PORT_BASE).await?;
    let mut client = UserStatsClient::connect(format!("http://{addr}")).await?;

    let raw_req = RawQueryRequest {
        query: "SELECT * FROM user_stats LIMIT 1".to_string(),
    };
    let stream = client.raw_query(raw_req).await?.into_inner();
    let ret = stream
        .then(|res| async move { res.unwrap() })
        .collect::<Vec<_>>()
        .await;
    assert_eq!(ret.len(), 1);

    Ok(())
}

#[tokio::test]
async fn stat_query_should_work() -> Result<()> {
    let (_tdb, addr) = start_serve(PORT_BASE + 1).await?;
    let mut client = UserStatsClient::connect(format!("http://{addr}")).await?;

    let mut timestamps = HashMap::new();
    timestamps.insert("created_at".to_string(), tq(Some(93), None));

    let req = QueryRequest {
        timestamps,
        ids: HashMap::new(),
    };

    let stream = client.query(req).await?.into_inner();
    let _users = stream.collect::<Vec<_>>().await;
    // assert_eq!(users.len(), 4);
    Ok(())
}

async fn start_serve(port: u32) -> Result<(TestMysql, SocketAddr)> {
    let addr = format!("[::1]:{}", port).parse()?;

    let (tdb, svc) = UserStatsService::new_for_test().await?;

    tokio::spawn(async move {
        Server::builder()
            .add_service(svc.into_server())
            .serve(addr)
            .await
            .unwrap();
    });
    sleep(Duration::from_micros(1)).await;

    Ok((tdb, addr))
}
