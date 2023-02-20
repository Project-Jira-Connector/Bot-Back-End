use super::users::{PurgeData, PurgeReason, User};
use crate::utils::Client;
use chrono::{DateTime, Duration, Utc};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use std::collections::{HashMap, HashSet};

#[derive(
    PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, Default, Debug, Serialize, Deserialize,
)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PlatformType {
    #[default]
    Cloud,
    Server,
}

#[skip_serializing_none]
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Hash, Default, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Robot {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub name: String,
    pub description: String,
    pub platform_email: String,
    pub platform_api_key: String,
    pub platform_type: PlatformType,
    pub cloud_session_token: String,
    pub active: bool,
    pub schedule: i64,
    pub last_active: i64,
    pub check_active_status: bool,
    pub check_double_email: bool,
    pub check_double_name: bool,
    pub last_updated: Option<DateTime<Utc>>,
}

impl From<&mut Robot> for RobotQuery {
    fn from(value: &mut Robot) -> Self {
        return RobotBuilder::new()
            .id(Some(value.id))
            .name(Some(value.name.clone()))
            .description(Some(value.description.clone()))
            .platform_email(Some(value.platform_email.clone()))
            .platform_api_key(Some(value.platform_api_key.clone()))
            .platform_type(Some(value.platform_type))
            .cloud_session_token(Some(value.cloud_session_token.clone()))
            .active(Some(value.active))
            .schedule(Some(value.schedule))
            .last_active(Some(value.last_active))
            .check_active_status(Some(value.check_active_status))
            .check_double_email(Some(value.check_double_email))
            .check_double_name(Some(value.check_double_name))
            .last_updated(value.last_updated)
            .finish();
    }
}

impl Robot {
    pub async fn think(&mut self, client: &Client, now: DateTime<Utc>) -> bool {
        if !self.active {
            return false;
        }

        if self.last_updated.is_some() {
            if now <= self.last_updated.unwrap() + Duration::days(self.schedule) {
                return false;
            }
        }

        let users = client.get_users(&self.cloud_session_token).await;
        if users.is_none() {
            return false;
        }

        self.last_updated = Some(now);

        let mut purge_users: HashMap<String, User> = HashMap::new();
        if self.check_double_email {
            let filtered_users = self.double_email(users.as_ref().unwrap());
            for user in filtered_users {
                let entry = purge_users.entry(user.id.clone()).or_insert(user);
                let purge_data = entry.purge_data.as_mut().unwrap();
                purge_data.reason.insert(PurgeReason::DoubleEmail);
            }
        }
        if self.check_double_name {
            let filtered_users = self.double_name(users.as_ref().unwrap());
            for user in filtered_users {
                let entry = purge_users.entry(user.id.clone()).or_insert(user);
                let purge_data = entry.purge_data.as_mut().unwrap();
                purge_data.reason.insert(PurgeReason::DoubleName);
            }
        }
        if self.check_active_status {
            let filtered_users = self.active_status(users.as_ref().unwrap());
            for user in filtered_users {
                let entry = purge_users.entry(user.id.clone()).or_insert(user);
                let purge_data = entry.purge_data.as_mut().unwrap();
                purge_data.reason.insert(PurgeReason::ActiveStatus);
            }
        }
        if self.last_active > 0 {
            let filtered_users = self.last_active(users.as_ref().unwrap());
            for user in filtered_users {
                let entry = purge_users.entry(user.id.clone()).or_insert(user);
                let purge_data = entry.purge_data.as_mut().unwrap();
                purge_data.reason.insert(PurgeReason::LastActive);
            }
        }

        let purge_users = purge_users.into_values().collect::<Vec<_>>();
        for user in purge_users {
            if client.add_user(&user).await.is_ok() {
                println!(
                    "[{:?}] {:?} has been queued for purging. ({:?})",
                    self.last_updated.unwrap(),
                    user.display_name,
                    user.purge_data.unwrap().reason
                )
            }
        }

        return true;
    }

    fn double_email(&mut self, users: &Vec<User>) -> Vec<User> {
        let mut map: HashMap<String, Vec<User>> = HashMap::new();
        for user in users {
            map.entry(user.email.clone())
                .or_insert_with(Vec::new)
                .push(user.clone());
        }

        let vec = map
            .into_iter()
            .filter(|ent| ent.1.len() > 1)
            .collect::<Vec<_>>();

        let mut ret: Vec<User> = Vec::new();
        for (_key, mut val) in vec {
            val.sort_by_key(|user| user.created);
            val.remove(0);
            ret.append(val.as_mut());
        }

        for i in 0..ret.len() {
            ret[i].purge_data = Some(PurgeData {
                robot: self.clone(),
                reason: HashSet::from([PurgeReason::DoubleEmail]),
                time: self.last_updated.unwrap() + Duration::days(7),
                alert: None,
            })
        }

        return ret;
    }

    fn double_name(&mut self, users: &Vec<User>) -> Vec<User> {
        let mut map: HashMap<String, Vec<User>> = HashMap::new();
        for user in users {
            map.entry(user.display_name.clone())
                .or_insert_with(Vec::new)
                .push(user.clone());
        }

        let vec = map
            .into_iter()
            .filter(|ent| ent.1.len() > 1)
            .collect::<Vec<_>>();

        let mut ret: Vec<User> = Vec::new();
        for (_key, mut val) in vec {
            val.sort_by_key(|user| user.created);
            val.remove(0);
            ret.append(val.as_mut());
        }

        for i in 0..ret.len() {
            ret[i].purge_data = Some(PurgeData {
                robot: self.clone(),
                reason: HashSet::from([PurgeReason::DoubleName]),
                time: self.last_updated.unwrap() + Duration::days(7),
                alert: None,
            });
        }

        return ret;
    }

    fn active_status(&mut self, users: &Vec<User>) -> Vec<User> {
        let mut ret: Vec<User> = Vec::new();
        for user in users {
            if !user.active {
                ret.push(user.clone());
            }
        }

        for i in 0..ret.len() {
            ret[i].purge_data = Some(PurgeData {
                robot: self.clone(),
                reason: HashSet::from([PurgeReason::ActiveStatus]),
                time: self.last_updated.unwrap() + Duration::days(7),
                alert: None,
            });
        }
        return ret;
    }

    fn last_active(&mut self, users: &Vec<User>) -> Vec<User> {
        let mut ret: Vec<User> = Vec::new();
        for user in users {
            let mut last_active = user.created;
            if let Some(presence) = user.presence {
                last_active = presence;
            } else if let Some(invitation_status) = user.invitation_status.clone() {
                last_active = invitation_status.invited_at;
            }

            if last_active + Duration::days(self.last_active) > self.last_updated.unwrap() {
                continue;
            }

            ret.push(user.clone());
        }

        for i in 0..ret.len() {
            ret[i].purge_data = Some(PurgeData {
                robot: self.clone(),
                reason: HashSet::from([PurgeReason::LastActive]),
                time: self.last_updated.unwrap() + Duration::days(7),
                alert: None,
            });
        }
        return ret;
    }
}

#[skip_serializing_none]
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Hash, Default, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RobotQuery {
    #[serde(rename = "_id")]
    pub id: Option<ObjectId>,
    pub name: Option<String>,
    pub description: Option<String>,
    pub platform_email: Option<String>,
    pub platform_api_key: Option<String>,
    pub platform_type: Option<PlatformType>,
    pub cloud_session_token: Option<String>,
    pub active: Option<bool>,
    pub schedule: Option<i64>,
    pub last_active: Option<i64>,
    pub check_active_status: Option<bool>,
    pub check_double_email: Option<bool>,
    pub check_double_name: Option<bool>,
    pub last_updated: Option<DateTime<Utc>>,
}

#[skip_serializing_none]
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Hash, Default, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RobotForm {
    pub name: String,
    pub description: String,
    pub platform_email: String,
    pub platform_api_key: String,
    pub platform_type: PlatformType,
    pub cloud_session_token: String,
    pub active: bool,
    pub schedule: i64,
    pub last_active: i64,
    pub check_active_status: bool,
    pub check_double_email: bool,
    pub check_double_name: bool,
}

impl From<RobotForm> for RobotQuery {
    fn from(value: RobotForm) -> Self {
        return RobotBuilder::new()
            .name(Some(value.name.clone()))
            .description(Some(value.description.clone()))
            .platform_email(Some(value.platform_email.clone()))
            .platform_api_key(Some(value.platform_api_key.clone()))
            .platform_type(Some(value.platform_type))
            .cloud_session_token(Some(value.cloud_session_token.clone()))
            .active(Some(value.active))
            .schedule(Some(value.schedule))
            .last_active(Some(value.last_active))
            .check_active_status(Some(value.check_active_status))
            .check_double_email(Some(value.check_double_email))
            .check_double_name(Some(value.check_double_name))
            .finish();
    }
}

pub struct RobotBuilder {
    pub id: Option<ObjectId>,
    pub name: Option<String>,
    pub description: Option<String>,
    pub platform_email: Option<String>,
    pub platform_api_key: Option<String>,
    pub platform_type: Option<PlatformType>,
    pub cloud_session_token: Option<String>,
    pub active: Option<bool>,
    pub scheduler: Option<i64>,
    pub last_active: Option<i64>,
    pub check_active_status: Option<bool>,
    pub check_double_email: Option<bool>,
    pub check_double_name: Option<bool>,
    pub last_updated: Option<DateTime<Utc>>,
}

impl RobotBuilder {
    pub fn new() -> Self {
        return Self {
            id: None,
            name: None,
            description: None,
            platform_email: None,
            platform_api_key: None,
            platform_type: None,
            cloud_session_token: None,
            active: None,
            scheduler: None,
            last_active: None,
            check_active_status: None,
            check_double_email: None,
            check_double_name: None,
            last_updated: None,
        };
    }

    pub fn id(&mut self, value: Option<ObjectId>) -> &mut Self {
        self.id = value;
        return self;
    }

    pub fn name(&mut self, value: Option<String>) -> &mut Self {
        self.name = value;
        return self;
    }

    pub fn description(&mut self, value: Option<String>) -> &mut Self {
        self.description = value;
        return self;
    }

    pub fn platform_email(&mut self, value: Option<String>) -> &mut Self {
        self.platform_email = value;
        return self;
    }

    pub fn platform_api_key(&mut self, value: Option<String>) -> &mut Self {
        self.platform_api_key = value;
        return self;
    }

    pub fn platform_type(&mut self, value: Option<PlatformType>) -> &mut Self {
        self.platform_type = value;
        return self;
    }

    pub fn cloud_session_token(&mut self, value: Option<String>) -> &mut Self {
        self.cloud_session_token = value;
        return self;
    }

    pub fn active(&mut self, value: Option<bool>) -> &mut Self {
        self.active = value;
        return self;
    }

    pub fn schedule(&mut self, value: Option<i64>) -> &mut Self {
        self.scheduler = value;
        return self;
    }

    pub fn last_active(&mut self, value: Option<i64>) -> &mut Self {
        self.last_active = value;
        return self;
    }

    pub fn check_active_status(&mut self, value: Option<bool>) -> &mut Self {
        self.check_active_status = value;
        return self;
    }

    pub fn check_double_email(&mut self, value: Option<bool>) -> &mut Self {
        self.check_double_email = value;
        return self;
    }

    pub fn check_double_name(&mut self, value: Option<bool>) -> &mut Self {
        self.check_double_name = value;
        return self;
    }

    pub fn last_updated(&mut self, value: Option<DateTime<Utc>>) -> &mut Self {
        self.last_updated = value;
        return self;
    }

    pub fn finish(&mut self) -> RobotQuery {
        return RobotQuery {
            id: self.id,
            name: self.name.clone(),
            description: self.description.clone(),
            platform_email: self.platform_email.clone(),
            platform_api_key: self.platform_api_key.clone(),
            platform_type: self.platform_type,
            cloud_session_token: self.cloud_session_token.clone(),
            active: self.active,
            schedule: self.scheduler,
            last_active: self.last_active,
            check_active_status: self.check_active_status,
            check_double_email: self.check_double_email,
            check_double_name: self.check_double_name,
            last_updated: self.last_updated,
        };
    }
}
