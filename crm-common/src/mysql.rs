use std::{path::Path, thread};

use sqlx::{
    migrate::{MigrationSource, Migrator},
    Connection, Executor, MySqlConnection, MySqlPool,
};
use tokio::runtime::Runtime;
use uuid::Uuid;

#[derive(Debug)]
pub struct TestMysql {
    pub host: String,
    pub port: u16,
    pub user: String,
    pub password: String,
    pub dbname: String,
}

impl TestMysql {
    pub fn new<S>(
        host: impl Into<String>,
        port: u16,
        user: impl Into<String>,
        password: impl Into<String>,
        migrations: S,
    ) -> Self
    where
        S: MigrationSource<'static> + Send + Sync + 'static,
    {
        let host = host.into();
        let user = user.into();
        let password = password.into();

        let uuid = Uuid::new_v4();
        let dbname = format!("test_{}", uuid.simple());
        let dbname_cloned = dbname.clone();

        let tdb = Self {
            host,
            port,
            user,
            password,
            dbname,
        };

        let server_url = tdb.server_url();
        let url = tdb.url();

        // create database dbname
        thread::spawn(move || {
            let rt = Runtime::new().unwrap();
            rt.block_on(async move {
                // use server url to create database
                let mut conn = MySqlConnection::connect(&server_url).await.unwrap();
                conn.execute(format!("CREATE DATABASE {}", dbname_cloned).as_str())
                    .await
                    .unwrap();

                // now connect to test database for migration
                let mut conn = MySqlConnection::connect(&url).await.unwrap();
                let m = Migrator::new(migrations).await.unwrap();
                m.run(&mut conn).await.unwrap();
            });
        })
        .join()
        .expect("failed to create database");

        tdb
    }

    pub fn server_url(&self) -> String {
        if self.password.is_empty() {
            format!("mysql://{}@{}:{}", self.user, self.host, self.port)
        } else {
            format!(
                "mysql://{}:{}@{}:{}",
                self.user, self.password, self.host, self.port
            )
        }
    }

    pub fn url(&self) -> String {
        format!("{}/{}", self.server_url(), self.dbname)
    }

    pub async fn get_pool(&self) -> MySqlPool {
        MySqlPool::connect(&self.url()).await.unwrap()
    }
}

impl Drop for TestMysql {
    fn drop(&mut self) {
        let server_url = self.server_url();
        let dbname = self.dbname.clone();
        thread::spawn(move || {
            let rt = Runtime::new().unwrap();
            rt.block_on(async move {
                let mut conn = MySqlConnection::connect(&server_url).await.unwrap();
                // TODO: terminate existing connections
                conn.execute(format!("DROP DATABASE {}", dbname).as_str())
                    .await
                    .expect("Error while querying the drop database");
            });
        })
        .join()
        .expect("failed to drop database");
    }
}

impl Default for TestMysql {
    fn default() -> Self {
        Self::new(
            "localhost",
            5432,
            "mysql",
            "mysql",
            Path::new("./migrations"),
        )
    }
}
