use sqlx::postgres as pg;
use sqlx::types::Uuid;

use plebiscite_types::{ Usergroup };


#[derive(Clone)]
pub struct DbDriver {
    pub db_pool: pg::PgPool
}


#[derive(Clone, Debug)]
pub struct LoggedInUser {
    pub user_id: i64,
    pub user_name: String,
    pub full_name: String,
}


impl DbDriver {
    pub async fn new() -> Self {
        let db_pool = pg::PgPoolOptions::new().connect("postgres://pleb_app:aoeuAOEU@localhost/pleb").await.unwrap();
        Self { db_pool }
    }

    pub async fn get_session_user(&self, session_id: Uuid) -> Option<LoggedInUser> {
        sqlx::query_as!(
            LoggedInUser, 
            r#"SELECT user_id as "user_id!", user_name as "user_name!", full_name as "full_name!" FROM get_session_user($1);"#, 
            session_id
            ).fetch_optional(&self.db_pool)
            .await
            .unwrap()
    }

    pub async fn try_login(&self, username: &str, password: &str) -> Option<Uuid> {
        sqlx::query_scalar!("SELECT try_login($1, $2);", username, password)
            .fetch_one(&self.db_pool)
            .await
            .unwrap()
    }
    
    pub async fn get_assigned_usergroups(&self, user_id: i64) -> Vec<Usergroup> {
        sqlx::query_as!(
            Usergroup,
            r#"SELECT usergroup_id as "usergroup_id!", title as "title!" FROM get_assigned_usergroups($1);"#,
            user_id
            ).fetch_all(&self.db_pool)
            .await
            .unwrap()
    }
}

