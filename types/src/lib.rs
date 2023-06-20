pub mod object_id;

use object_id::ObjectId;

#[derive(serde::Deserialize)]
pub struct LoginInfo {
    pub username: String,
    pub password: String,
}

//-----------------------------------------------------------

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct UserData {
    pub user_name: String,
    pub full_name: String,
}

pub type UserId = ObjectId<UserData>;

//-----------------------------------------------------------

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct UsergroupData {
    pub title: String,
    //pub tags: Option<i64>,
}

pub type UsergroupId = ObjectId<UsergroupData>;

pub type Usergroup = (UsergroupId, UsergroupData);

//-----------------------------------------------------------
