use crate::*;
use rayon::prelude::*;

#[derive(Clone)]
pub struct Client {
    client: mongodb::Client,
}

impl Client {
    pub async fn new(config: configs::mongodb::Config) -> Result<Self, Box<dyn std::error::Error>> {
        return Ok(Self {
            client: mongodb::Client::with_options(
               mongodb::options::ClientOptions::parse(format!(
                    "mongodb://{}:{}@ac-mt2requ-shard-00-00.pflxmhx.mongodb.net:27017,ac-mt2requ-shard-00-01.pflxmhx.mongodb.net:27017,ac-mt2requ-shard-00-02.pflxmhx.mongodb.net:27017/?ssl=true&replicaSet=atlas-g6x5st-shard-0&authSource=admin&retryWrites=true&w=majority",
                    config.username, config.password
                )).await?,
            )?
        });
    }

    pub async fn add_robot(
        &self,
        robot: &mut models::robot::Robot,
    ) -> Result<mongodb::results::InsertOneResult, mongodb::error::Error> {
        if robot.data.created.is_none() {
            robot.data.created = Some(chrono::Utc::now());
        }

        let insert_one_result = self
            .client
            .database("robots")
            .collection::<mongodb::bson::Document>("robots")
            .insert_one(mongodb::bson::to_document(&robot.data)?, None)
            .await?;

        robot.data.id.unique = insert_one_result.inserted_id.as_object_id();

        return Ok(insert_one_result);
    }

    pub async fn get_robot(
        &self,
        robot_id: &models::robot::RobotIdentifier,
    ) -> Result<Option<models::robot::RobotData>, mongodb::error::Error> {
        return Ok(self
            .client
            .database("robots")
            .collection::<models::robot::RobotData>("robots")
            .find_one(mongodb::bson::to_document(&robot_id)?, None)
            .await?);
    }

    pub async fn get_robots(&self) -> Result<Vec<models::robot::RobotData>, mongodb::error::Error> {
        return futures::TryStreamExt::try_collect(
            self.client
                .database("robots")
                .collection::<models::robot::RobotData>("robots")
                .find(None, None)
                .await?,
        )
        .await;
    }

    pub async fn delete_robot(
        &self,
        robot_id: &models::robot::RobotIdentifier,
    ) -> Result<mongodb::results::DeleteResult, mongodb::error::Error> {
        let purge_data = self.get_purge_users().await?;
        let purge_data = purge_data
            .par_iter()
            .filter(|purge_user| {
                return purge_user.robot.id == robot_id.unique.unwrap();
            })
            .collect::<Vec<_>>();

        for data in purge_data {
            self.delete_purge_user(data).await?;
        }

        return Ok(self
            .client
            .database("robots")
            .collection::<models::robot::RobotData>("robots")
            .delete_one(mongodb::bson::to_document(&robot_id)?, None)
            .await?);
    }

    pub async fn patch_robot(
        &self,
        robot: &models::robot::Robot,
    ) -> Result<mongodb::results::UpdateResult, mongodb::error::Error> {
        return self
            .client
            .database("robots")
            .collection::<mongodb::bson::Document>("robots")
            .update_one(
                mongodb::bson::doc! {"_id": robot.data.id.unique.unwrap()},
                mongodb::bson::doc! {"$set": mongodb::bson::to_document(&robot.data).unwrap()},
                None,
            )
            .await;
    }

    pub async fn add_purge_log(
        &self,
        log: &models::purge::PurgeLog,
    ) -> Result<mongodb::results::InsertOneResult, mongodb::error::Error> {
        return self
            .client
            .database("robots")
            .collection::<mongodb::bson::Document>("purge_logs")
            .insert_one(mongodb::bson::to_document(&log).unwrap(), None)
            .await;
    }

    pub async fn add_purge_user(
        &self,
        purge: &models::purge::PurgeData,
    ) -> Result<mongodb::results::UpdateResult, mongodb::error::Error> {
        return self
            .client
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
            self.client
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
            .client
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
            .client
            .database("robots")
            .collection::<mongodb::bson::Document>("purge_users")
            .update_one(
                mongodb::bson::doc! {"_id":purge.id},
                mongodb::bson::doc! {"$set":mongodb::bson::to_document(&purge).unwrap()},
                None,
            )
            .await;
    }
}
