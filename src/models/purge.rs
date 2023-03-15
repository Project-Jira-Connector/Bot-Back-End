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
#[serde(rename_all = "camelCase")]
pub struct PurgeUser {
    pub id: String,
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
    pub alert: Option<chrono::DateTime<chrono::Utc>>,
    pub reasons: Vec<PurgeReason>,
}

#[serde_with::skip_serializing_none]
#[derive(PartialEq, Eq, Clone, Default, Debug, serde::Serialize, serde::Deserialize)]
pub struct PurgeLog {
    pub user: models::jira::User,
    pub robot: models::robot::Robot,
    pub reasons: Vec<PurgeReason>,
    pub time: chrono::DateTime<chrono::Utc>,
}

impl PurgeLog {
    pub fn new(
        robot: &models::robot::Robot,
        user: &models::jira::User,
        reasons: Vec<PurgeReason>,
        time: chrono::DateTime<chrono::Utc>,
    ) -> Self {
        return Self {
            user: user.clone(),
            robot: robot.clone(),
            reasons,
            time,
        };
    }
}

impl PurgeData {
    pub fn new(
        robot: &models::robot::Robot,
        user: &models::jira::User,
        reasons: Vec<PurgeReason>,
        time: chrono::DateTime<chrono::Utc>,
    ) -> Self {
        return Self {
            id: None,
            user: PurgeUser {
                id: user.id.clone(),
                display_name: user.display_name.clone(),
                email: user.email.clone(),
                presence: user.get_available_presence(),
            },
            robot: PurgeRobot {
                id: robot.id.unwrap(),
            },
            time,
            alert: None,
            reasons,
        };
    }

    pub fn should_email_user(&self, now: chrono::DateTime<chrono::Utc>, delay: i64) -> bool {
        if self.alert.is_none() {
            return true;
        }
        return self.alert.unwrap() + chrono::Duration::days(delay) < now;
    }

    pub fn should_remove_user(&self, now: chrono::DateTime<chrono::Utc>) -> bool {
        return self.time <= now;
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
