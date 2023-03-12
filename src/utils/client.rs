use crate::*;

#[derive(Clone, Debug)]
pub struct Client {
    pub reqwest: reqwest::Client,
    pub mongodb: mongodb::Client,
}

impl Client {
    pub fn new(mongodb_username: &String, mongodb_password: &String) -> Self {
        return Self {
            reqwest: reqwest::Client::new(),
            mongodb: mongodb::Client::with_options(
                futures::executor::block_on(mongodb::options::ClientOptions::parse(format!(
                    "mongodb://{}:{}@ac-mt2requ-shard-00-00.pflxmhx.mongodb.net:27017,ac-mt2requ-shard-00-01.pflxmhx.mongodb.net:27017,ac-mt2requ-shard-00-02.pflxmhx.mongodb.net:27017/?ssl=true&replicaSet=atlas-g6x5st-shard-0&authSource=admin&retryWrites=true&w=majority",
                    mongodb_username, mongodb_password
                )))
                .unwrap(),
            )
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

    // pub async fn get_jira_project_roles(
    //     &self,
    //     platform_email: &String,
    //     platform_api_key: &String,
    // ) -> Vec<models::jira::ProjectRole> {
    //     let response = self
    //         .reqwest
    //         .get(format!(
    //             "https://{}.atlassian.net/rest/api/latest/role/",
    //             std::env::var("ORGANIZATION_NAME").unwrap()
    //         ))
    //         .basic_auth(platform_email, Some(platform_api_key))
    //         .send()
    //         .await;
    //     if response.is_err() {
    //         return vec![];
    //     }

    //     let text = response.unwrap().text().await;
    //     if text.is_err() {
    //         return vec![];
    //     }

    //     let data: Result<Vec<models::jira::ProjectRole>, _> =
    //         serde_json::from_str(text.unwrap().as_str());
    //     if data.is_err() {
    //         return vec![];
    //     }

    //     return data.unwrap();
    // }

    // pub async fn get_jira_project_role_actors(
    //     &self,
    //     platform_email: &String,
    //     platform_api_key: &String,
    //     project_id: &String,
    //     role_id: i64,
    // ) -> Vec<models::jira::RoleActor> {
    //     let response = self
    //         .reqwest
    //         .get(format!(
    //             "https://{}.atlassian.net/rest/api/latest/project/{}/role/{}",
    //             std::env::var("ORGANIZATION_NAME").unwrap(),
    //             project_id,
    //             role_id
    //         ))
    //         .basic_auth(platform_email, Some(platform_api_key))
    //         .send()
    //         .await;
    //     if response.is_err() {
    //         return vec![];
    //     }

    //     let text = response.unwrap().text().await;
    //     if text.is_err() {
    //         return vec![];
    //     }

    //     let data: Result<models::jira::ProjectRole, _> =
    //         serde_json::from_str(text.unwrap().as_str());
    //     if data.is_err() || data.as_ref().unwrap().actors.is_none() {
    //         return vec![];
    //     }

    //     return data.unwrap().actors.unwrap();
    // }

    pub async fn add_robot(
        &self,
        robot: &models::robot::Robot,
    ) -> Result<mongodb::results::InsertOneResult, mongodb::error::Error> {
        return self
            .mongodb
            .database("robots")
            .collection::<mongodb::bson::Document>("robots")
            .insert_one(mongodb::bson::to_document(&robot).unwrap(), None)
            .await;
    }

    pub async fn get_robots_with(
        &self,
        robot: &models::robot::RobotQuery,
    ) -> Result<Vec<models::robot::Robot>, mongodb::error::Error> {
        return futures::TryStreamExt::try_collect(
            self.mongodb
                .database("robots")
                .collection::<models::robot::Robot>("robots")
                .find(mongodb::bson::to_document(&robot).unwrap(), None)
                .await?,
        )
        .await;
    }

    pub async fn get_robots(&self) -> Result<Vec<models::robot::Robot>, mongodb::error::Error> {
        return futures::TryStreamExt::try_collect(
            self.mongodb
                .database("robots")
                .collection::<models::robot::Robot>("robots")
                .find(None, None)
                .await?,
        )
        .await;
    }

    pub async fn patch_robot(
        &self,
        robot: &models::robot::RobotQuery,
    ) -> Result<mongodb::results::UpdateResult, mongodb::error::Error> {
        return self
            .mongodb
            .database("robots")
            .collection::<mongodb::bson::Document>("robots")
            .update_one(
                mongodb::bson::doc! {"_id":robot.id},
                mongodb::bson::doc! {"$set":mongodb::bson::to_document(&robot).unwrap()},
                None,
            )
            .await;
    }

    pub async fn delete_robot(
        &self,
        robot: &models::robot::RobotQuery,
    ) -> Result<mongodb::results::DeleteResult, mongodb::error::Error> {
        return self
            .mongodb
            .database("robots")
            .collection::<mongodb::bson::Document>("robots")
            .delete_one(mongodb::bson::to_document(&robot).unwrap(), None)
            .await;
    }

    pub async fn add_purge_user(
        &self,
        purge: &models::purge::PurgeData,
    ) -> Result<mongodb::results::UpdateResult, mongodb::error::Error> {
        return self
            .mongodb
            .database("robots")
            .collection::<mongodb::bson::Document>("purge_users")
            .update_one(
                mongodb::bson::doc! {"user": mongodb::bson::to_document(&purge.user).unwrap()},
                mongodb::bson::doc! {"$setOnInsert": mongodb::bson::to_document(purge).unwrap()},
                mongodb::options::UpdateOptions::builder()
                    .upsert(true)
                    .build(),
            )
            .await;
    }

    pub async fn get_purge_users(
        &self,
    ) -> Result<Vec<models::purge::PurgeData>, mongodb::error::Error> {
        return futures::TryStreamExt::try_collect(
            self.mongodb
                .database("robots")
                .collection::<models::purge::PurgeData>("purge_users")
                .find(None, None)
                .await?,
        )
        .await;
    }

    pub async fn delete_purge_user(
        &self,
        purge: &models::purge::PurgeData,
    ) -> Result<mongodb::results::DeleteResult, mongodb::error::Error> {
        return self
            .mongodb
            .database("robots")
            .collection::<mongodb::bson::Document>("purge_users")
            .delete_one(mongodb::bson::to_document(&purge).unwrap(), None)
            .await;
    }

    pub async fn patch_purge_user(
        &self,
        purge: &models::purge::PurgeData,
    ) -> Result<mongodb::results::UpdateResult, mongodb::error::Error> {
        return self
            .mongodb
            .database("robots")
            .collection::<mongodb::bson::Document>("purge_users")
            .update_one(
                mongodb::bson::doc! {"_id":purge.id},
                mongodb::bson::doc! {"$set":mongodb::bson::to_document(&purge).unwrap()},
                None,
            )
            .await;
    }

    pub async fn remove_user_from_jira(
        &self,
        robot: &models::robot::Robot,
        purge: &models::purge::PurgeData,
    ) -> bool {
        return self
            .reqwest
            .post(format!(
                "https://telkomdevelopernetwork.atlassian.net/rest/api/latest/user?accountId={}",
                purge.user.id
            ))
            .header(
                reqwest::header::AUTHORIZATION,
                base64::encode(format!(
                    "{}:{}",
                    robot.credential.platform_email, robot.credential.platform_api_key
                )),
            )
            .send()
            .await
            .is_ok();
    }
}
