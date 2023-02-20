use crate::*;

#[derive(Clone, Debug)]
pub struct Client {
    pub reqwest: reqwest::Client,
    pub mongodb: mongodb::Client,
}

impl Client {
    pub fn new() -> Self {
        return Self {
            reqwest: reqwest::Client::new(),
            mongodb: futures::executor::block_on(mongodb::Client::with_uri_str(format!(
                "mongodb://{}:{}",
                std::env::var("MONGODB_ADDRESS").unwrap(),
                std::env::var("MONGODB_PORT").unwrap()
            )))
            .unwrap(),
        };
    }

    pub async fn get_jira_users(&self, cloud_session_token: &String) -> Vec<models::jira::User> {
        let mut users: Vec<models::jira::User> = vec![];
        let mut start_index = 1;
        loop {
            let response = self
            .reqwest
            .get(format!("https://admin.atlassian.com/gateway/api/adminhub/um/org/{}/users?count=100&start-index={}", std::env::var("ORGANIZATION_ID").unwrap(), start_index))
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

    pub async fn get_jira_project_roles(
        &self,
        platform_email: &String,
        platform_api_key: &String,
    ) -> Vec<models::jira::User> {
        let mut users: Vec<models::jira::User> = vec![];
        let mut start_index = 1;
        loop {
            let response = self
                .reqwest
                .get(format!(
                    "https://{}.atlassian.net/rest/api/latest/role/",
                    std::env::var("ORGANIZATION_NAME").unwrap()
                ))
                .basic_auth(platform_email, Some(platform_api_key))
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
}
