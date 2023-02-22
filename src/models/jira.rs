#[derive(
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Clone,
    Hash,
    Default,
    Debug,
    serde::Serialize,
    serde::Deserialize,
)]
#[serde(rename_all = "camelCase")]
pub struct ProjectRoleUser {
    pub account_id: String,
}

#[derive(
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Clone,
    Hash,
    Default,
    Debug,
    serde::Serialize,
    serde::Deserialize,
)]
#[serde(rename_all = "camelCase")]
pub struct RoleActor {
    pub display_name: String,
    pub actor_user: ProjectRoleUser,
}

#[derive(
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Clone,
    Hash,
    Default,
    Debug,
    serde::Serialize,
    serde::Deserialize,
)]
pub struct ProjectDetails {
    pub id: String,
}

#[derive(
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Clone,
    Hash,
    Default,
    Debug,
    serde::Serialize,
    serde::Deserialize,
)]
pub struct Scope {
    pub project: ProjectDetails,
}

#[serde_with::skip_serializing_none]
#[derive(
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Clone,
    Hash,
    Default,
    Debug,
    serde::Serialize,
    serde::Deserialize,
)]
pub struct ProjectRole {
    pub id: i64,
    pub actors: Option<Vec<RoleActor>>,
    pub scope: Option<Scope>,
}

#[serde_with::skip_serializing_none]
#[derive(
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Clone,
    Hash,
    Default,
    Debug,
    serde::Serialize,
    serde::Deserialize,
)]
pub struct ManagedStatus {
    pub managed: bool,
    pub owner: Option<String>,
}

#[derive(
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Clone,
    Hash,
    Default,
    Debug,
    serde::Serialize,
    serde::Deserialize,
)]
#[serde(rename_all = "camelCase")]
pub struct InvitationStatus {
    pub invited_at: chrono::DateTime<chrono::Utc>,
    pub status: String,
}

#[serde_with::skip_serializing_none]
#[derive(PartialEq, Eq, Clone, Default, Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct User {
    pub platform_role: Option<String>,
    pub id: String,
    pub email: String,
    pub display_name: String,
    pub active: bool,
    pub has_verified_email: bool,
    pub picture: String,
    pub active_status: String,
    pub nickname: String,
    pub title: Option<String>,
    pub timezone: Option<String>,
    pub location: Option<String>,
    pub company_name: Option<String>,
    pub department: Option<String>,
    pub presence: Option<chrono::DateTime<chrono::Utc>>,
    pub created: chrono::DateTime<chrono::Utc>,
    pub invitation_status: Option<InvitationStatus>,
    pub system: bool,
    pub org_admin: bool,
    pub site_admin: bool,
    pub sys_admin: bool,
    pub trusted_user: bool,
    pub presence_unavailable: Option<bool>,
    pub managed_status: ManagedStatus,
}

#[derive(PartialEq, Eq, Clone, Default, Debug, serde::Serialize, serde::Deserialize)]
pub struct Users {
    pub users: Vec<User>,
    pub total: i32,
}
