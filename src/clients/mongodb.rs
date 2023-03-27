use mongodb::results::InsertOneResult;

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
        robot: &models::robot::Robot,
    ) -> Result<InsertOneResult, mongodb::error::Error> {
        return Err(mongodb::error::Error::from(
            mongodb::error::ErrorKind::SessionsNotSupported,
        ));
        // return Ok(self
        //     .client
        //     .database("robots")
        //     .collection::<mongodb::bson::Document>("robots")
        //     .insert_one(mongodb::bson::to_document(&robot).unwrap(), None)
        //     .await?);
    }
}
