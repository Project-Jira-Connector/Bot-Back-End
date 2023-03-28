use crate::*;

#[derive(Clone)]
pub struct Client {
    client: rusoto_s3::S3Client,
}

impl Client {
    pub fn new(config: configs::rusoto::Config) -> Result<Self, Box<dyn std::error::Error>> {
        return Ok(Self {
            client: rusoto_s3::S3Client::new_with(
                rusoto_core::HttpClient::new()?,
                rusoto_credential::StaticProvider::from(rusoto_credential::AwsCredentials::new(
                    config.key,
                    config.secret,
                    None,
                    None,
                )),
                rusoto_core::Region::Custom {
                    name: String::from("s3-sgp1"),
                    endpoint: String::from("https://sgp1.digitaloceanspaces.com"),
                },
            ),
        });
    }

    pub async fn add_robot(
        &self,
        robot: &models::robot::Robot,
    ) -> Result<rusoto_s3::PutObjectOutput, Box<dyn std::error::Error>> {
        let request = rusoto_s3::PutObjectRequest {
            bucket: "atlassianbot".to_string(),
            key: format!(
                "robots/robot_{}.yaml",
                robot
                    .data
                    .id
                    .unique
                    .ok_or("Robot unique id is not defined")?
            ),
            content_type: Some("application/octet-stream".to_string()),
            body: Some(rusoto_core::ByteStream::from(
                serde_yaml::to_string(&robot.config)?.into_bytes(),
            )),
            ..Default::default()
        };
        return Ok(rusoto_s3::S3::put_object(&self.client, request).await?);
    }

    pub async fn get_robot(
        &self,
        key: &mongodb::bson::oid::ObjectId,
    ) -> Result<models::robot::RobotConfig, Box<dyn std::error::Error>> {
        let request = rusoto_s3::GetObjectRequest {
            bucket: "atlassianbot".to_string(),
            key: format!("robots/robot_{}.yaml", key),
            ..Default::default()
        };
        let response = rusoto_s3::S3::get_object(&self.client, request)
            .await
            .map_err(|_| "Failed to retrieve robot configuration")?;
        let mut buffer = String::new();
        tokio::io::AsyncReadExt::read_to_string(
            &mut response.body.unwrap().into_async_read(),
            &mut buffer,
        )
        .await?;
        return Ok(serde_yaml::from_str(&buffer)?);
    }

    pub async fn delete_robot(
        &self,
        key: &mongodb::bson::oid::ObjectId,
    ) -> Result<rusoto_s3::DeleteObjectOutput, Box<dyn std::error::Error>> {
        let request = rusoto_s3::DeleteObjectRequest {
            bucket: "atlassianbot".to_string(),
            key: format!("robots/robot_{}.yaml", key),
            ..Default::default()
        };
        return Ok(rusoto_s3::S3::delete_object(&self.client, request).await?);
    }

    pub async fn patch_robot(
        &self,
        robot: &models::robot::Robot,
    ) -> Result<rusoto_s3::PutObjectOutput, Box<dyn std::error::Error>> {
        let request = rusoto_s3::PutObjectRequest {
            bucket: "atlassianbot".to_string(),
            key: format!("robots/robot_{}.yaml", robot.data.id.unique.unwrap()),
            body: Some(rusoto_core::ByteStream::from(
                serde_yaml::to_string(&robot.config)?.into_bytes(),
            )),
            ..Default::default()
        };
        return Ok(rusoto_s3::S3::put_object(&self.client, request).await?);
    }

    // pub async fn get_robots(
    //     &self,
    //     robot_query: &models::robot::RobotQuery,
    // ) -> Result<Vec<models::robot::Robot>, Box<dyn std::error::Error>> {
    //     let request = rusoto_s3::ListObjectsV2Request {
    //         bucket: "atlassianbot".to_owned(),
    //         prefix: Some("robots/".to_owned()),
    //         ..Default::default()
    //     };
    //     let contents = rusoto_s3::S3::list_objects_v2(&self.client, request)
    //         .await?
    //         .contents
    //         .ok_or("Robots directory not found")?;
    //     let mut robots: Vec<models::robot::Robot> = Vec::new();
    //     for object in contents {
    //         if let Some(key) = object.key {
    //             if let Ok(robot) = self.get_robot(key.clone()).await {
    //                 robots.push(robot);
    //             }
    //         }
    //     }
    //     let robots = robots
    //         .into_iter()
    //         .filter(|robot| {
    //             if robot_query.id.is_some() && robot_query.id.unwrap() != robot.id.unwrap() {
    //                 return false;
    //             }
    //             return true;
    //         })
    //         .collect::<Vec<_>>();
    //     return Ok(robots);
    // }
}
