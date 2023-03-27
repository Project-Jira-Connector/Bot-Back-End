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
            bucket: String::from("atlassianbot"),
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

    // pub async fn delete_robot(
    //     &self,
    //     robot: &models::robot::RobotQuery,
    // ) -> Result<(), Box<dyn std::error::Error>> {
    //     let request = rusoto_s3::DeleteObjectRequest {
    //         bucket: String::from("atlassianbot"),
    //         key: format!(
    //             "robots/robot_{}.yaml",
    //             robot.id.ok_or("Robot id is not specified")?
    //         ),
    //         ..Default::default()
    //     };
    //     let response = rusoto_s3::S3::delete_object(&self.client, request).await?;
    //     return Ok(());
    // }

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

    // pub async fn patch_robot(
    //     &self,
    //     robot_query: &models::robot::RobotQuery,
    // ) -> Result<(), Box<dyn std::error::Error>> {
    //     let mut robots = self.get_robots(robot_query).await?;
    //     if robots.len() != 1 {
    //         return Ok(());
    //     }
    //     let robot = &mut robots[0];

    //     if robot_query.info.name.is_some() {
    //         robot.info.name = robot_query.info.name.as_ref().unwrap().clone();
    //     }
    //     if robot_query.info.description.is_some() {
    //         robot.info.name = robot_query.info.description.as_ref().unwrap().clone();
    //     }

    //     if robot_query.credential.cloud_session_token.is_some() {
    //         robot.credential.cloud_session_token = robot_query
    //             .credential
    //             .cloud_session_token
    //             .as_ref()
    //             .unwrap()
    //             .clone();
    //     }
    //     if robot_query.credential.platform_api_key.is_some() {
    //         robot.credential.platform_api_key = robot_query
    //             .credential
    //             .platform_api_key
    //             .as_ref()
    //             .unwrap()
    //             .clone();
    //     }
    //     if robot_query.credential.platform_email.is_some() {
    //         robot.credential.platform_email = robot_query
    //             .credential
    //             .platform_email
    //             .as_ref()
    //             .unwrap()
    //             .clone();
    //     }
    //     if robot_query.credential.platform_type.is_some() {
    //         robot.credential.platform_type = robot_query
    //             .credential
    //             .platform_type
    //             .as_ref()
    //             .unwrap()
    //             .clone();
    //     }

    //     if robot_query.scheduler.active.is_some() {
    //         robot.credential.platform_type = robot_query
    //             .credential
    //             .platform_type
    //             .as_ref()
    //             .unwrap()
    //             .clone();
    //     }
    //     return Ok(());
    // }
}
