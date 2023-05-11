#[derive(serde::Deserialize)]
pub struct LoginInfo {
    pub username: String,
    pub password: String,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Usergroup {
    pub usergroup_id: i64,
    pub title: String,
}
