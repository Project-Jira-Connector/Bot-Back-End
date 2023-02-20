use crate::models::{
    projects::{self, Actor, Project, Role, RoleData},
    robots::{Robot, RobotQuery},
    users::{self, User},
};
use futures::executor::block_on;
use lettre::{transport::smtp::authentication::Credentials, Message, SmtpTransport};
use mongodb::{
    bson::{doc, to_document, Document},
    error::Error,
    options::UpdateOptions,
    results::{DeleteResult, InsertOneResult, UpdateResult},
};
use reqwest::header::COOKIE;
use serde_json::{from_str, to_string};
use std::env::var;

#[derive(Clone, Debug)]
pub struct Client {
    pub reqwest: reqwest::Client,
    pub mongodb: mongodb::Client,
}

impl Client {
    pub fn new() -> Self {
        return Self {
            reqwest: reqwest::Client::new(),
            mongodb: block_on(mongodb::Client::with_uri_str("mongodb://localhost:27017")).unwrap(),
        };
    }

    pub async fn get_robots(&self, robot: RobotQuery) -> Option<Vec<Robot>> {
        let results = self
            .mongodb
            .database("robots")
            .collection::<Document>("robots")
            .find(to_document(&robot).unwrap(), None)
            .await;
        if results.is_err() {
            return None;
        }

        let documents = futures::TryStreamExt::try_collect::<Vec<_>>(results.unwrap()).await;
        if documents.is_err() {
            return None;
        }

        let json = to_string(&documents.unwrap());
        if json.is_err() {
            return None;
        }

        let robots = from_str(json.unwrap().as_str());
        if robots.is_err() {
            return None;
        }

        return Some(robots.unwrap());
    }

    pub async fn add_robot(&self, robot: &RobotQuery) -> Result<InsertOneResult, Error> {
        return self
            .mongodb
            .database("robots")
            .collection::<Document>("robots")
            .insert_one(to_document(&robot).unwrap(), None)
            .await;
    }

    pub async fn patch_robot(&self, robot: &RobotQuery) -> Result<UpdateResult, Error> {
        return self
            .mongodb
            .database("robots")
            .collection::<Document>("robots")
            .update_one(
                doc! {"_id":robot.id},
                doc! {"$set":to_document(&robot).unwrap()},
                None,
            )
            .await;
    }

    pub async fn delete_robot(&self, robot: &RobotQuery) -> Result<DeleteResult, Error> {
        return self
            .mongodb
            .database("robots")
            .collection::<Document>("robots")
            .delete_one(to_document(&robot).unwrap(), None)
            .await;
    }

    pub async fn get_projects(&self, email: &String, api_key: &String) -> Option<Vec<Project>> {
        let organization_name = var("ORGANIZATION_NAME");
        if organization_name.is_err() {
            return None;
        }

        let response = self
        .reqwest
        .get(format!("https://{}.atlassian.net/rest/api/latest/project/search?startAt=0&maxResults=2147483647", organization_name.unwrap()))
        .basic_auth(email, Some(api_key))
        .send()
        .await;
        if response.is_err() {
            return None;
        }

        let text = response.unwrap().text().await;
        if text.is_err() {
            return None;
        }

        let data: Result<projects::Data, _> = from_str(text.unwrap().as_str());
        if data.is_err() {
            return None;
        }

        return Some(data.unwrap().values);
    }

    pub async fn get_project_role(
        &self,
        email: &String,
        api_key: &String,
        project: &Project,
    ) -> Option<Role> {
        let response = self
            .reqwest
            .get(format!("{}/role", project.url))
            .basic_auth(email, Some(api_key))
            .send()
            .await;
        if response.is_err() {
            return None;
        }

        let text = response.unwrap().text().await;
        if text.is_err() {
            return None;
        }

        let data: Result<Role, _> = from_str(text.unwrap().as_str());
        if data.is_err() {
            return None;
        }

        return Some(data.unwrap());
    }

    pub async fn get_project_role_data(
        &self,
        email: &String,
        api_key: &String,
        url: &String,
    ) -> Option<RoleData> {
        let response = self
            .reqwest
            .get(url)
            .basic_auth(email, Some(api_key))
            .send()
            .await;
        if response.is_err() {
            return None;
        }

        let text = response.unwrap().text().await;
        if text.is_err() {
            return None;
        }

        let data: Result<RoleData, _> = from_str(text.unwrap().as_str());
        if data.is_err() {
            return None;
        }

        return Some(data.unwrap());
    }

    pub async fn get_users(&self, cloud_session_token: &String) -> Option<Vec<User>> {
        let organization_id = var("ORGANIZATION_ID");
        if organization_id.is_err() {
            return None;
        }

        let mut users: Vec<User> = vec![];
        let mut start_index = 1;

        loop {
            let response = self
            .reqwest
            .get(format!("https://admin.atlassian.com/gateway/api/adminhub/um/org/{}/users?count=100&start-index={}", organization_id.as_ref().unwrap(), start_index))
            .header(COOKIE, format!("cloud.session.token={}", cloud_session_token))
            .send()
            .await;
            if response.is_err() {
                break;
            }

            let text = response.unwrap().text().await;
            if text.is_err() {
                break;
            }

            let data: Result<users::Data, _> = from_str(text.unwrap().as_str());
            if data.is_err() {
                break;
            }

            users.extend(data.as_ref().unwrap().users.clone());

            if data.unwrap().total <= start_index + 99 {
                break;
            }

            start_index += 100;
        }

        return Some(users);
    }

    pub async fn add_user(&self, user: &User) -> Result<UpdateResult, Error> {
        return self
            .mongodb
            .database("robots")
            .collection::<Document>("users")
            .update_one(
                doc! {"id":user.id.clone()},
                doc! {"$setOnInsert":to_document(user).unwrap()},
                UpdateOptions::builder().upsert(true).build(),
            )
            .await;
    }

    pub async fn log_user(&self, user: &User) -> Result<InsertOneResult, Error> {
        return self
            .mongodb
            .database("robots")
            .collection::<Document>("logs")
            .insert_one(to_document(&user).unwrap(), None)
            .await;
    }

    pub async fn delete_user(&self, user: &User) -> Result<DeleteResult, Error> {
        return self
            .mongodb
            .database("robots")
            .collection::<Document>("users")
            .delete_one(to_document(&user).unwrap(), None)
            .await;
    }

    pub async fn get_users_to_purge(&self) -> Option<Vec<User>> {
        let results = self
            .mongodb
            .database("robots")
            .collection::<Document>("users")
            .find(None, None)
            .await;
        if results.is_err() {
            return None;
        }

        let documents = futures::TryStreamExt::try_collect::<Vec<_>>(results.unwrap()).await;
        if documents.is_err() {
            return None;
        }

        let json = to_string(&documents.unwrap());
        if json.is_err() {
            return None;
        }

        let robots = from_str(json.unwrap().as_str());
        if robots.is_err() {
            return None;
        }

        return Some(robots.unwrap());
    }

    pub async fn patch_user(&self, user: &User) -> Result<UpdateResult, Error> {
        return self
            .mongodb
            .database("robots")
            .collection::<Document>("users")
            .update_one(
                doc! {"id":user.id.clone()},
                doc! {"$set":to_document(&user).unwrap()},
                None,
            )
            .await;
    }

    pub fn send_email(
        &self,
        username: String,
        password: String,
        name: String,
        email: String,
    ) -> bool {
        let message = Message::builder()
            .from(format!("Telkom Atlassian <{}>", username).parse().unwrap())
            .to(format!("{} <{}>", name, email).parse().unwrap())
            .subject("Telkom Atlassian")
            .body(String::from("Account queued for purging!"))
            .unwrap();

        let relay = SmtpTransport::relay("smtp.gmail.com")
            .unwrap()
            .credentials(Credentials::new(username, password))
            .build();
        return lettre::Transport::send(&relay, &message).is_ok();
    }
}
