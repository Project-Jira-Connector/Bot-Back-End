use crate::*;

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
}
