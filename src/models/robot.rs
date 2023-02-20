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
    pub project_id: String,
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
pub struct RobotScheduler {
    pub active: bool,
    pub delay: i64,
    pub last_active: i64,
    pub check_double_name: bool,
    pub check_double_email: bool,
    pub check_active_status: bool,
    pub last_updated: chrono::DateTime<chrono::Utc>,
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
pub struct Robot {
    #[serde(rename = "_id")]
    pub id: mongodb::bson::oid::ObjectId,
    #[serde(flatten)]
    pub info: RobotInfo,
    #[serde(flatten)]
    pub credential: RobotCredential,
    #[serde(flatten)]
    pub scheduler: RobotScheduler,
}

impl Robot {
    pub fn filter_jira_user(&self, users: &Vec<models::jira::User>) -> models::purge::PurgeUsers {
        let mut purge_users = models::purge::PurgeUsers::new();
        self.filter_inactivity(users, &mut purge_users);
        self.filter_duplicate(users, &mut purge_users);
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

                if self.filter_email(user, other_user, 0.8) {}

                if self.filter_name(user, other_user, 0.8) {}
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
            let mut purge_data: Option<&mut models::purge::PurgeData> = None;

            if self.filter_last_active(user) {
                purge_data =
                    Some(purge_users.push(user, self, models::purge::PurgeReason::LastActive));
            }

            if self.filter_active_status(user) {
                match purge_data {
                    Some(purge_data) => {
                        purge_data
                            .reason
                            .push(models::purge::PurgeReason::ActiveStatus);
                    }
                    None => {
                        purge_users.push(user, self, models::purge::PurgeReason::ActiveStatus);
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
            > self.scheduler.last_updated
        {
            return false;
        }

        return true;
    }

    fn filter_active_status(&self, user: &models::jira::User) -> bool {
        return self.scheduler.check_active_status && !user.active;
    }
}
