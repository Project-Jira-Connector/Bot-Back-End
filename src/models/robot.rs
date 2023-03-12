use crate::*;

#[derive(
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Clone,
    Copy,
    Hash,
    Default,
    Debug,
    serde::Serialize,
    serde::Deserialize,
)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PlatformType {
    #[default]
    Cloud,
    Server,
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
pub struct RobotInfo {
    pub name: String,
    pub description: String,
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
pub struct RobotInfoQuery {
    pub name: Option<String>,
    pub description: Option<String>,
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
pub struct RobotCredential {
    pub platform_email: String,
    pub platform_api_key: String,
    pub platform_type: PlatformType,
    pub cloud_session_token: String,
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
#[serde(rename_all = "camelCase")]
pub struct RobotCredentialQuery {
    pub platform_email: Option<String>,
    pub platform_api_key: Option<String>,
    pub platform_type: Option<PlatformType>,
    pub cloud_session_token: Option<String>,
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
#[serde(rename_all = "camelCase")]
pub struct RobotScheduler {
    pub active: bool,
    pub schedule: i64,
    pub last_active: i64,
    pub check_double_name: bool,
    pub check_double_email: bool,
    pub check_active_status: bool,
    pub last_updated: Option<chrono::DateTime<chrono::Utc>>,
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
#[serde(rename_all = "camelCase")]
pub struct RobotSchedulerQuery {
    pub active: Option<bool>,
    pub schedule: Option<i64>,
    pub last_active: Option<i64>,
    pub check_double_name: Option<bool>,
    pub check_double_email: Option<bool>,
    pub check_active_status: Option<bool>,
    pub last_updated: Option<chrono::DateTime<chrono::Utc>>,
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
pub struct Robot {
    #[serde(rename = "_id")]
    pub id: Option<mongodb::bson::oid::ObjectId>,
    #[serde(flatten)]
    pub info: RobotInfo,
    #[serde(flatten)]
    pub credential: RobotCredential,
    #[serde(flatten)]
    pub scheduler: RobotScheduler,
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
pub struct RobotQuery {
    #[serde(rename = "_id")]
    pub id: Option<mongodb::bson::oid::ObjectId>,
    #[serde(flatten)]
    pub info: RobotInfoQuery,
    #[serde(flatten)]
    pub credential: RobotCredentialQuery,
    #[serde(flatten)]
    pub scheduler: RobotSchedulerQuery,
}

impl RobotQuery {
    pub fn new() -> Self {
        return Self {
            id: None,
            info: RobotInfoQuery {
                name: None,
                description: None,
            },
            credential: RobotCredentialQuery {
                platform_email: None,
                platform_api_key: None,
                platform_type: None,
                cloud_session_token: None,
            },
            scheduler: RobotSchedulerQuery {
                active: None,
                schedule: None,
                last_active: None,
                check_double_name: None,
                check_double_email: None,
                check_active_status: None,
                last_updated: None,
            },
        };
    }
}

impl From<&mut Robot> for RobotQuery {
    fn from(robot: &mut Robot) -> Self {
        return RobotQuery {
            id: robot.id,
            info: RobotInfoQuery {
                name: Some(robot.info.name.clone()),
                description: Some(robot.info.description.clone()),
            },
            credential: RobotCredentialQuery {
                platform_email: Some(robot.credential.platform_email.clone()),
                platform_api_key: Some(robot.credential.platform_api_key.clone()),
                platform_type: Some(robot.credential.platform_type),
                cloud_session_token: Some(robot.credential.cloud_session_token.clone()),
            },
            scheduler: RobotSchedulerQuery {
                active: Some(robot.scheduler.active),
                schedule: Some(robot.scheduler.schedule),
                last_active: Some(robot.scheduler.last_active),
                check_double_name: Some(robot.scheduler.check_double_name),
                check_double_email: Some(robot.scheduler.check_double_email),
                check_active_status: Some(robot.scheduler.check_active_status),
                last_updated: robot.scheduler.last_updated,
            },
        };
    }
}

impl Robot {
    pub async fn update(
        &mut self,
        client: &utils::client::Client,
        now: chrono::DateTime<chrono::Utc>,
    ) {
        self.scheduler.last_updated = Some(now);
    }

    fn filter_jira_user(&self, users: &Vec<models::jira::User>) -> models::purge::PurgeUsers {
        let mut purge_users = models::purge::PurgeUsers::new();
        self.filter_duplicate(users, &mut purge_users);
        self.filter_inactivity(users, &mut purge_users);
        return purge_users;
    }

    fn filter_duplicate(
        &self,
        users: &Vec<models::jira::User>,
        purge_users: &mut models::purge::PurgeUsers,
    ) {
        for user_index in 0..users.len() {
            let user = &users[user_index];

            for other_user_index in user_index + 1..users.len() {
                let other_user = &users[other_user_index];

                let mut purge_data_cached: Option<&mut models::purge::PurgeData> = None;

                if self.filter_email(user, other_user, 0.8) {
                    purge_data_cached = Some(purge_users.push(
                        other_user,
                        self,
                        models::purge::PurgeReason::DuplicateEmail,
                        7,
                    ));
                }

                if self.filter_name(user, other_user, 0.8) {
                    match purge_data_cached {
                        Some(purge_data) => {
                            purge_data
                                .reasons
                                .push(models::purge::PurgeReason::DuplicateName);
                        }
                        None => {
                            purge_users.push(
                                other_user,
                                self,
                                models::purge::PurgeReason::DuplicateName,
                                7,
                            );
                        }
                    };
                }
            }
        }
    }

    fn filter_email(
        &self,
        user: &models::jira::User,
        other_user: &models::jira::User,
        threshold: f64,
    ) -> bool {
        return self.scheduler.check_double_email
            && strsim::normalized_damerau_levenshtein(&user.email, &other_user.email) >= threshold;
    }

    fn filter_name(
        &self,
        user: &models::jira::User,
        other_user: &models::jira::User,
        threshold: f64,
    ) -> bool {
        return self.scheduler.check_double_name
            && strsim::normalized_damerau_levenshtein(
                &user.display_name,
                &other_user.display_name,
            ) >= threshold;
    }

    fn filter_inactivity(
        &self,
        users: &Vec<models::jira::User>,
        purge_users: &mut models::purge::PurgeUsers,
    ) {
        for user in users {
            let mut purge_data_cached: Option<&mut models::purge::PurgeData> = None;

            if self.filter_last_active(user) {
                purge_data_cached =
                    Some(purge_users.push(user, self, models::purge::PurgeReason::LastActive, 7));
            }

            if self.filter_active_status(user) {
                match purge_data_cached {
                    Some(purge_data) => {
                        purge_data
                            .reasons
                            .push(models::purge::PurgeReason::ActiveStatus);
                    }
                    None => {
                        purge_users.push(user, self, models::purge::PurgeReason::ActiveStatus, 7);
                    }
                };
            }
        }
    }

    fn filter_last_active(&self, user: &models::jira::User) -> bool {
        if self.scheduler.last_active <= 0 {
            return false;
        }

        let mut last_active = user.created;
        if let Some(presence) = user.presence {
            last_active = presence;
        } else if let Some(invitation_status) = user.invitation_status.clone() {
            last_active = invitation_status.invited_at;
        }

        if last_active + chrono::Duration::days(self.scheduler.last_active)
            > self.scheduler.last_updated.unwrap()
        {
            return false;
        }

        return true;
    }

    fn filter_active_status(&self, user: &models::jira::User) -> bool {
        return self.scheduler.check_active_status && !user.active;
    }
}
