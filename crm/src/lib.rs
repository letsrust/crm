mod abi;
mod config;

pub mod pb;

use anyhow::Result;
pub use config::*;
use crm_metadata::pb::metadata_client::MetadataClient;
use crm_send::pb::notification_client::NotificationClient;
use pb::{
    crm_server::{Crm, CrmServer},
    RecallRequest, RecallResponse, RemindRequest, RemindResponse, WelcomeRequest, WelcomeResponse,
};
use tonic::{async_trait, transport::Channel, Request, Response, Status};
use user_stat::pb::user_stats_client::UserStatsClient;

#[allow(unused)]
pub struct CrmService {
    config: AppConfig,
    user_stats: UserStatsClient<Channel>,
    notification: NotificationClient<Channel>,
    metadata: MetadataClient<Channel>,
}

#[async_trait]
impl Crm for CrmService {
    async fn welcome(
        &self,
        request: Request<WelcomeRequest>,
    ) -> Result<Response<WelcomeResponse>, Status> {
        let request = request.into_inner();
        self.welcome(request).await
    }

    async fn recall(
        &self,
        request: Request<RecallRequest>,
    ) -> Result<Response<RecallResponse>, Status> {
        let request = request.into_inner();
        self.recall(request).await
    }

    async fn remind(
        &self,
        request: Request<RemindRequest>,
    ) -> Result<Response<RemindResponse>, Status> {
        let request = request.into_inner();
        self.remind(request).await
    }
}

impl CrmService {
    pub async fn try_new(config: AppConfig) -> Result<Self> {
        let user_stats = UserStatsClient::connect(config.server.user_stats.clone()).await?;
        let notification = NotificationClient::connect(config.server.notification.clone()).await?;
        let metadata = MetadataClient::connect(config.server.metadata.clone()).await?;
        Ok(Self {
            config,
            user_stats,
            notification,
            metadata,
        })
    }

    pub fn into_server(self) -> CrmServer<Self> {
        CrmServer::new(self)
    }
}
