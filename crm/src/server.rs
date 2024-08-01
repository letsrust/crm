use anyhow::Result;
use crm::{
    pb::{
        user_srv_server::{UserSrv, UserSrvServer},
        CreateUserRequest, GetUserRequest, User,
    },
    AppConfig, CrmService,
};
use tonic::{async_trait, transport::Server, Request, Response, Status};
use tracing::{info, level_filters::LevelFilter};
use tracing_subscriber::{fmt::Layer, layer::SubscriberExt, util::SubscriberInitExt, Layer as _};

#[derive(Default)]
pub struct UserService {}

#[async_trait]
impl UserSrv for UserService {
    async fn get_user(&self, request: Request<GetUserRequest>) -> Result<Response<User>, Status> {
        let input = request.into_inner();
        println!("Get user: {input:?}");
        Ok(Response::new(User::default()))
    }

    async fn create_user(
        &self,
        request: Request<CreateUserRequest>,
    ) -> Result<Response<User>, Status> {
        let input = request.into_inner();
        println!("Create user: {input:?}");

        let user = User::new(1, input.name.as_str(), input.email.as_str());
        Ok(Response::new(user))
    }
}

impl UserService {
    pub fn into_server(self) -> UserSrvServer<Self> {
        UserSrvServer::new(self)
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let layer = Layer::new().with_filter(LevelFilter::INFO);
    tracing_subscriber::registry().with(layer).init();

    let config = AppConfig::load()?;

    let addr = format!("[::1]:{}", config.server.port).parse()?;
    let user_svc = UserService::default().into_server();
    let crm_svc = CrmService::try_new(config).await?.into_server();

    info!("Crm server listening on {addr}");

    Server::builder()
        .add_service(user_svc)
        .add_service(crm_svc)
        .serve(addr)
        .await?;
    Ok(())
}
