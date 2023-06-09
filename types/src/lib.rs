pub mod object_id;

use object_id::ObjectId;

#[derive(serde::Deserialize)]
pub struct LoginInfo {
    pub username: String,
    pub password: String,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct UserData {
    pub user_name: String,
    pub full_name: String,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct UsergroupData {
    pub title: String,
    //pub tags: Option<i64>,
}

pub type UserId = ObjectId<UserData>;

pub type UsergroupId = ObjectId<UsergroupData>;



#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
//#[cfg_attr(feature = "sqlx", derive(sqlx::FromRow))]
pub struct Usergroup {
    pub id: UsergroupId,
    pub data: UsergroupData,
}
