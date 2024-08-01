use anyhow::Result;
use crm::pb::{
    crm_client::CrmClient, user_srv_client::UserSrvClient, CreateUserRequest, RecallRequest,
    RemindRequest, WelcomeRequest,
};
use tonic::Request;
use tracing::{info, level_filters::LevelFilter};
use tracing_subscriber::{fmt::Layer, layer::SubscriberExt, util::SubscriberInitExt, Layer as _};

#[tokio::main]
async fn main() -> Result<()> {
    let layer = Layer::new().with_filter(LevelFilter::INFO);
    tracing_subscriber::registry().with(layer).init();

    // call_crm_welcome().await?;
    // call_crm_recall().await?;
    call_crm_remind().await?;

    Ok(())
}

#[allow(dead_code)]
async fn call_crm_welcome() -> Result<()> {
    info!("call crm welcome...");
    let mut client = CrmClient::connect("http://[::1]:50000").await?;

    let req = WelcomeRequest {
        id: uuid::Uuid::new_v4().to_string(),
        interval: 95u32,
        content_ids: vec![1u32],
    };

    let response = client.welcome(req).await?;
    info!("{:?}", response);

    Ok(())
}

#[allow(dead_code)]
async fn call_crm_recall() -> Result<()> {
    info!("call crm recall...");
    let mut client = CrmClient::connect("http://[::1]:50000").await?;

    let req = RecallRequest {
        id: uuid::Uuid::new_v4().to_string(),
        last_visit_interval: 60u32,
        content_ids: vec![1u32],
    };

    let response = client.recall(req).await?;
    info!("{:?}", response);

    Ok(())
}

#[allow(dead_code)]
async fn call_crm_remind() -> Result<()> {
    info!("call crm remind...");
    let mut client = CrmClient::connect("http://[::1]:50000").await?;

    let req = RemindRequest {
        id: uuid::Uuid::new_v4().to_string(),
        last_visit_interval: 60u32,
    };

    let response = client.remind(req).await?;
    info!("{:?}", response);

    Ok(())
}

#[allow(dead_code)]
async fn call_user_service() -> Result<()> {
    let mut client = UserSrvClient::connect("http://[::1]:50051").await?;

    let request = Request::new(CreateUserRequest {
        name: String::from("zhangsan"),
        email: String::from("zhangsan@gmail.com"),
    });

    let resp = client.create_user(request).await?;
    let user = resp.into_inner();
    println!("User created: {user:?}");
    Ok(())
}
