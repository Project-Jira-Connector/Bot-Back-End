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
pub enum PurgeReason {
    #[default]
    ActiveStatus,
    LastActive,
    DuplicateEmail,
    DuplicateName,
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
pub struct PurgeReasonsContainer {
    pub data: Vec<PurgeReason>,
}

impl PurgeReasonsContainer {
    pub fn new(reason: PurgeReason) -> Self {
        return Self { data: vec![reason] };
    }

    pub fn push(&mut self, reason: PurgeReason) {
        match self.data.iter().find(|&e| *e == reason) {
            Some(_reason) => {}
            None => {
                self.data.push(reason);
            }
        };
    }
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
pub struct PurgeUser {
    pub user_id: String,
    pub display_name: String,
    pub email: String,
    pub presence: chrono::DateTime<chrono::Utc>,
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
pub struct PurgeRobot {
    pub id: mongodb::bson::oid::ObjectId,
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
pub struct PurgeData {
    #[serde(rename = "_id")]
    pub id: Option<mongodb::bson::oid::ObjectId>,
    pub user: PurgeUser,
    pub robot: PurgeRobot,
    pub time: chrono::DateTime<chrono::Utc>,
    pub alert: chrono::DateTime<chrono::Utc>,
    pub reasons: PurgeReasonsContainer,
}

impl PurgeData {
    pub fn should_email_user(&self, now: chrono::DateTime<chrono::Utc>) -> bool {
        return self.alert + chrono::Duration::days(3) < now;
    }

    pub fn should_remove_user(&self, now: chrono::DateTime<chrono::Utc>) -> bool {
        return self.time < now;
    }

    pub fn email_user(&self, notification_email: &String, notification_password: &String) -> bool {
        return lettre::Transport::send(
            &lettre::SmtpTransport::relay("smtp.gmail.com")
                .unwrap()
                .credentials(lettre::transport::smtp::authentication::Credentials::new(
                    notification_email.clone(),
                    notification_password.clone(),
                ))
                .build(),
            &lettre::Message::builder()
                .from(
                    format!("Telkom Developer Network <{}>", notification_email)
                        .parse()
                        .unwrap(),
                )
                .to(format!("{} <{}>", self.user.display_name, self.user.email)
                    .parse()
                    .unwrap())
                .subject("[ALERT]")
                .body(String::from("Your account has been queued for purging!"))
                .unwrap(),
        )
        .is_ok();
    }
}

#[derive(PartialEq, Eq, Clone, Default, Debug, serde::Serialize, serde::Deserialize)]
pub struct PurgeUsers {
    pub users: std::collections::HashMap<String, PurgeData>,
}

impl PurgeUsers {
    pub fn new() -> Self {
        return Self {
            users: std::collections::HashMap::new(),
        };
    }

    pub fn push(
        &mut self,
        user: &models::jira::User,
        robot: &models::robot::Robot,
        reason: PurgeReason,
        days: i64,
    ) -> &mut PurgeData {
        return match self.users.entry(user.id.clone()) {
            std::collections::hash_map::Entry::Occupied(e) => {
                let purge_data = e.into_mut();
                purge_data.reasons.push(reason);
                return purge_data;
            }
            std::collections::hash_map::Entry::Vacant(e) => e.insert(PurgeData {
                id: None,
                user: PurgeUser {
                    user_id: user.id.clone(),
                    display_name: user.display_name.clone(),
                    email: user.email.clone(),
                    presence: user.presence.unwrap_or_else(move || {
                        return match &user.invitation_status {
                            Some(invitation_status) => invitation_status.invited_at,
                            None => user.created,
                        };
                    }),
                },
                robot: PurgeRobot {
                    id: robot.id.unwrap(),
                },
                time: robot.scheduler.last_updated.unwrap() + chrono::Duration::days(days),
                alert: robot.scheduler.last_updated.unwrap(),
                reasons: PurgeReasonsContainer::new(reason),
            }),
        };
    }

    pub fn get(&self) -> Vec<&PurgeData> {
        return Vec::from_iter(self.users.values());
    }
}
