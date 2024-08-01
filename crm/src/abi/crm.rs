use std::{collections::HashMap, sync::Arc};

use chrono::Utc;
use crm_metadata::pb::{Content, MaterializeRequest};
use crm_send::pb::SendRequest;
use futures::StreamExt;
use prost_types::Timestamp;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Response, Status};
use tracing::{info, warn};
use user_stat::pb::{QueryRequest, TimeQuery};

use crate::{
    pb::{WelcomeRequest, WelcomeResponse},
    CrmService,
};

impl CrmService {
    pub async fn welcome(
        &self,
        request: WelcomeRequest,
    ) -> Result<Response<WelcomeResponse>, Status> {
        let req_id = request.id;

        // let d1 = Utc::now() - Duration::days(request.interval as _);
        // let d2 = d1 + Duration::days(1);

        let d1 = to_ts(request.interval as _);
        let d2 = to_ts(0);

        let mut timestamps = HashMap::new();

        let time_query = TimeQuery {
            lower: Some(d1),
            upper: Some(d2),
        };
        timestamps.insert("created_at".to_string(), time_query);

        let query = QueryRequest {
            timestamps,
            ids: HashMap::new(),
        };

        info!("query user stats: {:?}", query);
        let mut user_stat_res = self.user_stats.clone().query(query).await?.into_inner();

        let contents = self
            .metadata
            .clone()
            .materialize(MaterializeRequest::new_with_ids(&request.content_ids))
            .await?
            .into_inner();
        let contents: Vec<Content> = contents
            .filter_map(|v| async move { v.ok() })
            .collect()
            .await;
        let contents = Arc::new(contents);
        info!("contents size: {}", contents.clone().len());

        let (tx, rx) = mpsc::channel(1024);
        let sender = self.config.server.sender_email.clone();
        tokio::spawn(async move {
            while let Some(Ok(user)) = user_stat_res.next().await {
                let contents = contents.clone();
                let sender = sender.clone();
                let tx = tx.clone();

                let req = SendRequest::new("Welcome".to_string(), sender, &[user.email], &contents);
                if let Err(e) = tx.send(req).await {
                    warn!("Failed to send message: {:?}", e);
                }

                // info!("sent {:?}", req_clone);
            }
        });

        info!("call notification");
        let reqs = ReceiverStream::new(rx);
        self.notification.clone().send(reqs).await?;

        Ok(Response::new(WelcomeResponse { id: req_id }))
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
