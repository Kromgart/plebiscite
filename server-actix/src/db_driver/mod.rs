use std::fmt;
use deadpool_postgres::{Config, Object, Pool};
use tokio_postgres::{types::ToSql, Row, Statement};
use uuid::Uuid;

use plebiscite_types::{UserData, UserId, Usergroup, UsergroupId, UsergroupData};

#[macro_use]
mod macros;

type PgError = tokio_postgres::Error;
type PoolError = deadpool_postgres::PoolError;

#[derive(Debug)]
pub enum DbError {
    Pool(PoolError),
    Postgres(PgError),
    NoResult,
}

impl fmt::Display for DbError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DbError::Pool(e) => write!(f, "Pool error: {}", e),
            DbError::Postgres(e) => write!(f, "Db error: {}", e),
            DbError::NoResult => write!(f, "No result from database"),
        }
    }
}

pub type DbResult<T> = Result<T, DbError>;

#[derive(Clone)]
pub struct DbDriver {
    db_pool: Pool,
}

#[derive(Clone, Debug)]
pub struct User {
    pub user_id: UserId,
    pub data: UserData,
}

impl DbDriver {

    pub async fn new() -> Self {
        // "postgres://pleb_app:aoeuAOEU@localhost/pleb"

        let mut cfg = Config::new();
        cfg.user = Some(String::from("pleb_app"));
        cfg.password = Some(String::from("aoeuAOEU"));
        cfg.dbname = Some(String::from("pleb"));
        cfg.host = Some(String::from("localhost"));

        let db_pool = cfg
            .create_pool(
                Some(deadpool_postgres::Runtime::Tokio1),
                tokio_postgres::NoTls,
            )
            .unwrap();

        Self { db_pool }
    }

    async fn prepare_pool_query(&self, query: &'static str) -> DbResult<(Object, Statement)> {
        let client = self.db_pool.get().await.map_err(DbError::Pool)?;
        let stmt = client
            .prepare_cached(query)
            .await
            .map_err(DbError::Postgres)?;

        Ok((client, stmt))
    }

    async fn query_opt(
        &self,
        str_query: &'static str,
        args: &[&(dyn ToSql + Sync)],
    ) -> DbResult<Option<Row>> {
        let (client, stmt) = self.prepare_pool_query(str_query).await?;

        client
            .query_opt(&stmt, args)
            .await
            .map_err(DbError::Postgres)
    }

    async fn query_vector(
        &self,
        str_query: &'static str,
        args: &[&(dyn ToSql + Sync)],
    ) -> DbResult<Vec<Row>> {
        let (client, stmt) = self.prepare_pool_query(str_query).await?;

        client
            .query(&stmt, args)
            .await
            .map_err(DbError::Postgres)
    }

    pub async fn get_session_user(&self, session_id: Uuid) -> DbResult<Option<User>> {
        pg_fn_option!(
            self,
            "get_session_user",
            [&session_id],
            User { 
                user_id, 
                data: UserData { 
                    user_name, 
                    full_name 
                }
            }
        )
    }

    pub async fn try_login(&self, username: &str, password: &str) -> DbResult<Option<Uuid>> {
        pg_fn_option!(self, "try_login", [&username, &password])
    }

    pub async fn try_register_login( &self, username: &str, password: &str) -> DbResult<Option<Uuid>> {
        pg_fn_option!(self, "try_register_login", [&username, &password])
    }

    pub async fn get_assigned_usergroups(&self, user_id: UserId) -> DbResult<Vec<Usergroup>> {
        pg_fn_vector!(
            self, 
            "get_assigned_usergroups", 
            [&user_id], 
            (
                "usergroup_id",
                UsergroupData { title }
            )
        )
    }

    pub async fn create_usergroup(&self, creator: UserId, group: UsergroupData) -> DbResult<UsergroupId> {
        pg_fn_one!(self, "create_assign_usergroup", [&creator, &group.title])
    }
}
