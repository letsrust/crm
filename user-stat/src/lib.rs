mod abi;
mod config;
pub mod pb;

use std::pin::Pin;
use std::{ops::Deref, sync::Arc};

pub use config::AppConfig;

use futures::Stream;
use pb::user_stats_server::{UserStats, UserStatsServer};
use pb::{QueryRequest, RawQueryRequest, User};
use sqlx::MySqlPool;
use tonic::{async_trait, Request, Response, Status};

type ServiceResult<T> = Result<Response<T>, Status>;
type ResponseStream = Pin<Box<dyn Stream<Item = Result<User, Status>> + Send>>;

#[derive(Clone)]
pub struct UserStatsService {
    inner: Arc<UserStatsServiceInner>,
}

#[allow(unused)]
pub struct UserStatsServiceInner {
    config: AppConfig,
    pool: MySqlPool,
}

#[async_trait]
impl UserStats for UserStatsService {
    type QueryStream = ResponseStream;
    type RawQueryStream = ResponseStream;

    async fn query(&self, request: Request<QueryRequest>) -> ServiceResult<Self::QueryStream> {
        let query = request.into_inner();
        self.query(query).await
    }

    async fn raw_query(
        &self,
        request: Request<RawQueryRequest>,
    ) -> ServiceResult<Self::RawQueryStream> {
        let query = request.into_inner();
        self.raw_query(query).await
    }
}

impl UserStatsService {
    pub async fn new(config: AppConfig) -> Self {
        let pool = MySqlPool::connect(&config.server.db_url)
            .await
            .expect("Failed to connect to db");

        let inner = UserStatsServiceInner { config, pool };

        Self {
            inner: Arc::new(inner),
        }
    }

    pub fn into_server(self) -> UserStatsServer<Self> {
        UserStatsServer::new(self)
    }
}

impl Deref for UserStatsService {
    type Target = UserStatsServiceInner;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

#[cfg(feature = "test_utils")]
pub mod test_utils {
    use std::{env, path::Path, sync::Arc};

    use anyhow::Result;
    use chrono::Utc;
    use crm_common::TestMysql;
    use prost_types::Timestamp;
    use sqlx::{Executor, MySqlPool};

    use crate::{pb::TimeQuery, AppConfig, UserStatsService, UserStatsServiceInner};

    impl UserStatsService {
        pub async fn new_for_test() -> Result<(TestMysql, Self)> {
            let config = AppConfig::load()?;
            let (tdb, pool) = get_test_pool().await;
            let svc = Self {
                inner: Arc::new(UserStatsServiceInner { config, pool }),
            };
            Ok((tdb, svc))
        }
    }

    pub async fn get_test_pool() -> (TestMysql, MySqlPool) {
        // let url = match url {
        //     Some(url) => url.to_string(),
        //     None => "mysql://root:123456@localhost:3306".to_string(),
        // };
        let p = Path::new(&env::var("CARGO_MANIFEST_DIR").unwrap()).join("migrations");
        let tdb = TestMysql::new("localhost", 3306, "root", "123456", p);
        let pool = tdb.get_pool().await;

        // run prepared sql to insert test dat
        let sql = include_str!("../fixtures/data.sql").split(';');
        let mut ts = pool.begin().await.expect("begin transaction failed");
        for s in sql {
            if s.trim().is_empty() {
                continue;
            }
            ts.execute(s).await.expect("execute sql failed");
            // println!("actual sql -> {}", s);
        }
        ts.commit().await.expect("commit transaction failed");

        (tdb, pool)
    }

    pub fn tq(lower: Option<i64>, upper: Option<i64>) -> TimeQuery {
        TimeQuery {
            lower: lower.map(to_ts),
            upper: upper.map(to_ts),
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
}
