use std::{collections::HashMap, sync::Arc};

use chrono::Utc;
use crm_metadata::pb::{Content, MaterializeRequest};
use crm_send::pb::SendRequest;
use futures::StreamExt;
use prost_types::Timestamp;
use tokio::sync::mpsc::{self, Receiver};
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Response, Status, Streaming};
use tracing::{info, warn};
use user_stat::pb::{QueryRequest, TimeQuery, User};

use crate::{
    pb::{
        RecallRequest, RecallResponse, RemindRequest, RemindResponse, WelcomeRequest,
        WelcomeResponse,
    },
    CrmService,
};

impl CrmService {
    pub async fn welcome(
        &self,
        request: WelcomeRequest,
    ) -> Result<Response<WelcomeResponse>, Status> {
        let req_id = request.id;

        let query = self.new_user_stat_query(request.interval, "created_at".to_string());
        info!("query user stats: {:?}", query);
        let user_stat_res = self.user_stats.clone().query(query).await?.into_inner();

        let contents = self.get_contents(&request.content_ids).await;
        let rx = self.build_send_stream(user_stat_res, contents.clone(), "Welcome".to_string());

        info!("call notification");
        let reqs = ReceiverStream::new(rx);
        self.notification.clone().send(reqs).await?;

        Ok(Response::new(WelcomeResponse { id: req_id }))
    }

    pub async fn recall(&self, request: RecallRequest) -> Result<Response<RecallResponse>, Status> {
        let req_id = request.id;

        let query =
            self.new_user_stat_query(request.last_visit_interval, "last_visited_at".to_string());
        info!("query user stats: {:?}", query);
        let user_stat_res = self.user_stats.clone().query(query).await?.into_inner();

        let contents = self.get_contents(&request.content_ids).await;
        let rx = self.build_send_stream(user_stat_res, contents.clone(), "Recall".to_string());

        let reqs = ReceiverStream::new(rx);
        self.notification.clone().send(reqs).await?;

        Ok(Response::new(RecallResponse { id: req_id }))
    }

    pub async fn remind(&self, request: RemindRequest) -> Result<Response<RemindResponse>, Status> {
        let req_id = request.id;

        let query =
            self.new_user_stat_query(request.last_visit_interval, "last_visited_at".to_string());
        info!("query user stats: {:?}", query);
        let user_stat_res = self.user_stats.clone().query(query).await?.into_inner();

        let contents = Arc::new(vec![]);
        let rx = self.build_send_stream(user_stat_res, contents, "Remind".to_string());

        let reqs = ReceiverStream::new(rx);
        self.notification.clone().send(reqs).await?;

        Ok(Response::new(RemindResponse { id: req_id }))
    }

    fn build_send_stream(
        &self,
        mut user_stat_res: Streaming<User>,
        contents: Arc<Vec<Content>>,
        subject: String,
    ) -> Receiver<SendRequest> {
        let (tx, rx) = mpsc::channel(1024);
        let sender = self.config.server.sender_email.clone();
        tokio::spawn(async move {
            while let Some(Ok(user)) = user_stat_res.next().await {
                let contents = contents.clone();
                let sender = sender.clone();
                let tx = tx.clone();

                let req = SendRequest::new(subject.clone(), sender, &[user.email], &contents);
                if let Err(e) = tx.send(req).await {
                    warn!("Failed to send message: {:?}", e);
                }
            }
        });

        rx
    }

    fn new_user_stat_query(&self, interval: u32, query_key: String) -> QueryRequest {
        let d1 = to_ts(interval as _);
        let d2 = to_ts(0);

        let mut timestamps = HashMap::new();
        let time_query = TimeQuery {
            lower: Some(d1),
            upper: Some(d2),
        };
        timestamps.insert(query_key, time_query);

        QueryRequest {
            timestamps,
            ids: HashMap::new(),
        }
    }

    async fn get_contents(&self, content_ids: &[u32]) -> Arc<Vec<Content>> {
        let contents = self
            .metadata
            .clone()
            .materialize(MaterializeRequest::new_with_ids(content_ids))
            .await;

        match contents {
            Ok(c) => {
                let contents: Vec<Content> = c
                    .into_inner()
                    .filter_map(|v| async move { v.ok() })
                    .collect()
                    .await;
                let contents = Arc::new(contents);
                info!("contents size: {}", contents.clone().len());
                contents
            }
            Err(_) => {
                warn!("failed to get contents: {:?}", content_ids);
                Arc::new(vec![])
            }
        }
    }
}

pub fn to_ts(days: i64) -> Timestamp {
    let dt = Utc::now()
        .checked_sub_signed(chrono::Duration::days(days))
        .unwrap();
    Timestamp {
        seconds: dt.timestamp(),
        nanos: dt.timestamp_subsec_nanos() as i32,
    }
}
