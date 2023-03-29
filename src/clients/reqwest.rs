use crate::*;

#[derive(Clone)]
pub struct Client {
    client: reqwest::Client,
    config: configs::reqwest::Config,
}

impl Client {
    pub fn new(config: configs::reqwest::Config) -> Self {
        return Self {
            client: reqwest::Client::new(),
            config,
        };
    }

    pub async fn get_jira_users(&self, cloud_session_token: &String) -> Vec<models::jira::User> {
        let mut users: Vec<models::jira::User> = vec![];
        let mut start_index = 1;
        loop {
            let response = self
            .client
            .get(format!("https://admin.atlassian.com/gateway/api/adminhub/um/org/{}/users?count=100&start-index={}", self.config.organization_id, start_index))
            .header(reqwest::header::COOKIE, format!("cloud.session.token={}", cloud_session_token))
            .send()
            .await;
            if response.is_err() {
                break;
            }

            let text = response.unwrap().text().await;
            if text.is_err() {
                break;
            }

            let data: Result<models::jira::Users, _> = serde_json::from_str(text.unwrap().as_str());
            if data.is_err() {
                break;
            }

            users.extend(data.as_ref().unwrap().users.clone());
            if data.unwrap().total <= start_index + 99 {
                break;
            }

            start_index += 100;
        }
        return users;
    }

    pub async fn remove_user_from_jira(
        &self,
        robot: &models::robot::Robot,
        purge: &models::purge::PurgeData,
    ) -> Result<reqwest::Response, reqwest::Error> {
        return self
            .client
            .delete(format!(
                "https://telkomdevelopernetwork.atlassian.net/rest/api/latest/user?accountId={}",
                purge.user.id
            ))
            .header(
                reqwest::header::AUTHORIZATION,
                base64::encode(format!(
                    "{}:{}",
                    robot.config.credential.platform_email,
                    robot.config.credential.platform_api_key
                )),
            )
            .send()
            .await;
    }
}
