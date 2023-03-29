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
#[derive(PartialEq, Clone, Default, Debug, serde::Serialize, serde::Deserialize)]
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
                id: robot.data.id.unique.unwrap(),
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

    pub fn email_user(
        &self,
        notification_email: &String,
        notification_password: &String,
        contact: &String,
    ) -> bool {
        let relay = lettre::SmtpTransport::relay("smtp.gmail.com")
            .unwrap()
            .credentials(lettre::transport::smtp::authentication::Credentials::new(
                notification_email.clone(),
                notification_password.clone(),
            ))
            .build();

        //let body = format!("<html><head><title>Notification: Jira Access Removal</title></head><body><p>Dear {},</p><p>We regret to inform you that your access to Jira will be removed on {}. This action has been taken due to {:?}.</p><p>If you have any questions or concerns, please contact {}.</p><p>Thank you for your understanding.</p><a href=\"https://id.atlassian.com/login\" class=\"btn\">Go to Jira Login Page</a></body></html>", self.user.display_name, self.time.format("%Y-%B-%d %H:%M:%S").to_string(), self.reasons, contact);

        let body = format!(
            r#"<!DOCTYPE html>
            <html>
              <head>
                <title>Notification: Jira Access Removal</title>
                <style type="text/css">
                  body {{
                    font-family: Arial, sans-serif;
                    font-size: 14px;
                    line-height: 1.5;
                    margin: 0;
                    padding: 0;
                  }}
            
                  .container {{
                    max-width: 600px;
                    margin: 20px auto;
                    padding: 20px;
                    background-color: #f2f2f2;
                    border: 1px solid #ccc;
                    box-shadow: 0 0 10px #ccc;
                  }}
            
                  h1 {{
                    font-size: 24px;
                    font-weight: bold;
                    margin-top: 0;
                  }}
            
                  p {{
                    margin-bottom: 20px;
                  }}
            
                  .btn {{
                    display: inline-block;
                    padding: 10px 20px;
                    background-color: #4CAF50;
                    color: #fff;
                    text-decoration: none;
                    border-radius: 4px;
                    font-weight: bold;
                  }}
            
                  .btn:hover {{
                    background-color: #3e8e41;
                  }}
                </style>
              </head>
              <body>
                <div class="container">
                  <h1>[ALERT] Jira Access Removal</h1>
                  <p>Dear {},</p>
                  <p>
                    We regret to inform you that your access to Jira will be removed on
                    {}. This action has been taken due to {:?}.
                  </p>
                  <p>
                    If you have any questions or concerns, please contact {}.
                  </p>
                  <p>Thank you for your understanding.</p>
                  <a href="https://id.atlassian.com/login" class="btn">Go to Jira Login Page</a>
                </div>
              </body>
            </html>
            "#,
            self.user.display_name,
            self.time.format("%d-%B-%Y %H:%M:%S").to_string(),
            self.reasons,
            contact
        );

        let message = lettre::Message::builder()
            .from(
                format!("Telkom Developer Network <{}>", notification_email)
                    .parse()
                    .unwrap(),
            )
            .to(format!("{} <{}>", self.user.display_name, self.user.email)
                .parse()
                .unwrap())
            .subject("[ALERT] Jira Access Removal")
            .header(lettre::message::header::ContentType::TEXT_HTML)
            .body(body)
            .unwrap();

        return lettre::Transport::send(&relay, &message).is_ok();
    }
}
