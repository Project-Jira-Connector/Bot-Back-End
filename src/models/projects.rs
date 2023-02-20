use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Hash, Default, Debug, Serialize, Deserialize)]
pub struct ProjectScope {
    pub id: String,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Hash, Default, Debug, Serialize, Deserialize)]
pub struct RoleScope {
    #[serde(rename = "type")]
    pub _type: String,
    pub project: ProjectScope,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Hash, Default, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct User {
    pub account_id: String,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Hash, Default, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Actor {
    pub id: i32,
    pub display_name: String,
    #[serde(rename = "type")]
    pub _type: String,
    pub actor_user: User,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Hash, Default, Debug, Serialize, Deserialize)]
pub struct RoleData {
    #[serde(rename = "self")]
    pub url: String,
    pub name: String,
    pub id: i32,
    pub description: String,
    pub actors: Vec<Actor>,
    pub scope: RoleScope,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Hash, Default, Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct UnknownRole {
    pub atlassian_addons_project_access: String,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Hash, Default, Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Role {
    #[serde(flatten)]
    pub unknown: UnknownRole,
    pub administrator: String,
    pub viewer: String,
    pub member: String,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Hash, Default, Debug, Serialize, Deserialize)]
pub struct AvatarUrls {
    #[serde(rename = "48x48")]
    pub very_large: String,
    #[serde(rename = "24x24")]
    pub medium: String,
    #[serde(rename = "16x16")]
    pub small: String,
    #[serde(rename = "32x32")]
    pub large: String,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Hash, Default, Debug, Serialize, Deserialize)]
pub struct Properties {}

#[skip_serializing_none]
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Hash, Default, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Project {
    pub expand: String,
    #[serde(rename = "self")]
    pub url: String,
    pub id: String,
    pub key: String,
    pub name: String,
    pub avatar_urls: AvatarUrls,
    pub project_type_key: String,
    pub simplified: bool,
    pub style: String,
    pub is_private: bool,
    pub properties: Properties,
    pub entity_id: Option<String>,
    pub uuid: Option<String>,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Hash, Default, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Data {
    #[serde(rename = "self")]
    pub url: String,
    pub max_results: i32,
    pub start_at: i32,
    pub total: i32,
    pub is_last: bool,
    pub values: Vec<Project>,
}
