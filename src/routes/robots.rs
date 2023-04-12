use crate::*;

pub async fn get(
    request: actix_web::HttpRequest,
    robot_id_query: actix_web::web::Query<models::robot::RobotIdentifier>,
) -> Result<actix_web::HttpResponse, actix_web::Error> {
    let robot_id = robot_id_query.into_inner();

    let mongodb = request
        .app_data::<actix_web::web::Data<clients::mongodb::Client>>()
        .ok_or(errors::error::Error::new(
            actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
            "MongoDB client not found".to_string(),
        ))?;

    let rusoto = request
        .app_data::<actix_web::web::Data<clients::rusoto::Client>>()
        .ok_or(errors::error::Error::new(
            actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
            "Rusoto client not found".to_string(),
        ))?;

    if let Some(id) = robot_id.unique {
        let robot_data = mongodb
            .get_robot(&robot_id)
            .await
            .map_err(|error| {
                errors::error::Error::new(
                    actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
                    error.to_string(),
                )
            })?
            .ok_or(errors::error::Error::new(
                actix_web::http::StatusCode::NOT_FOUND,
                format!("Robot with unique id ({:?}) doesn't exist", id.to_string()),
            ))?;

        let robot_config = rusoto.get_robot(&id).await.map_err(|error| {
            errors::error::Error::new(
                actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
                error.to_string(),
            )
        })?;

        return Ok(actix_web::HttpResponse::Found()
            .json(models::robot::Robot::new(robot_data, robot_config)));
    }

    let robots_data = mongodb.get_robots().await.map_err(|error| {
        errors::error::Error::new(
            actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
            error.to_string(),
        )
    })?;

    let mut robots = Vec::<models::robot::Robot>::with_capacity(robots_data.len());

    for robot_data in robots_data {
        if let Ok(robot_config) = rusoto
            .get_robot(&robot_data.id.unique.unwrap())
            .await
            .map_err(|error| {
                errors::error::Error::new(
                    actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
                    error.to_string(),
                )
            })
        {
            robots.push(models::robot::Robot::new(robot_data, robot_config));
        }
    }

    return Ok(actix_web::HttpResponse::Found().json(robots));
}

pub async fn post(
    request: actix_web::HttpRequest,
    robot_json: actix_web::web::Json<models::robot::Robot>,
) -> Result<actix_web::HttpResponse, actix_web::Error> {
    let mut robot = robot_json.into_inner();

    let reqwest = request
        .app_data::<actix_web::web::Data<clients::reqwest::Client>>()
        .ok_or(errors::error::Error::new(
            actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
            "Reqwest client not found".to_string(),
        ))?;

    let valid_credential =
        reqwest
            .check_jira_credentials(&robot)
            .await
            .or(Err(errors::error::Error::new(
                actix_web::http::StatusCode::UNAUTHORIZED,
                "Invalid credentials".to_string(),
            )))?;
    if !valid_credential {
        return Err(errors::error::Error::new(
            actix_web::http::StatusCode::UNAUTHORIZED,
            "Invalid credentials".to_string(),
        )
        .into());
    }

    let users = reqwest
        .get_jira_users(&robot.config.credential.cloud_session_token)
        .await;
    if users.is_empty() {
        return Err(errors::error::Error::new(
            actix_web::http::StatusCode::UNAUTHORIZED,
            "Invalid cloud session token".to_string(),
        )
        .into());
    }

    let mongodb = request
        .app_data::<actix_web::web::Data<clients::mongodb::Client>>()
        .ok_or(errors::error::Error::new(
            actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
            "MongoDB client not found".to_string(),
        ))?;

    mongodb.add_robot(&mut robot).await.map_err(|error| {
        errors::error::Error::new(
            actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
            error.to_string(),
        )
    })?;

    let rusoto = request
        .app_data::<actix_web::web::Data<clients::rusoto::Client>>()
        .ok_or(errors::error::Error::new(
            actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
            "Rusoto client not found".to_string(),
        ))?;

    rusoto.add_robot(&robot).await?;

    return Ok(actix_web::HttpResponse::Created().json(robot));
}

pub async fn patch(
    request: actix_web::HttpRequest,
    robot_json: actix_web::web::Json<models::robot::Robot>,
) -> Result<actix_web::HttpResponse, actix_web::Error> {
    let mut robot = robot_json.into_inner();

    robot.data.id.unique.ok_or(errors::error::Error::new(
        actix_web::http::StatusCode::BAD_REQUEST,
        "'_id' can't be 'None'".to_string(),
    ))?;

    let reqwest = request
        .app_data::<actix_web::web::Data<clients::reqwest::Client>>()
        .ok_or(errors::error::Error::new(
            actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
            "Reqwest client not found".to_string(),
        ))?;

    let valid_credential =
        reqwest
            .check_jira_credentials(&robot)
            .await
            .or(Err(errors::error::Error::new(
                actix_web::http::StatusCode::UNAUTHORIZED,
                "Invalid credentials".to_string(),
            )))?;
    if !valid_credential {
        return Err(errors::error::Error::new(
            actix_web::http::StatusCode::UNAUTHORIZED,
            "Invalid credentials".to_string(),
        )
        .into());
    }

    let users = reqwest
        .get_jira_users(&robot.config.credential.cloud_session_token)
        .await;
    if users.is_empty() {
        return Err(errors::error::Error::new(
            actix_web::http::StatusCode::UNAUTHORIZED,
            "Invalid cloud session token".to_string(),
        )
        .into());
    }

    let mongodb = request
        .app_data::<actix_web::web::Data<clients::mongodb::Client>>()
        .ok_or(errors::error::Error::new(
            actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
            "MongoDB client not found".to_string(),
        ))?;

    mongodb.patch_robot(&mut robot).await.map_err(|error| {
        errors::error::Error::new(
            actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
            error.to_string(),
        )
    })?;

    let rusoto = request
        .app_data::<actix_web::web::Data<clients::rusoto::Client>>()
        .ok_or(errors::error::Error::new(
            actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
            "Rusoto client not found".to_string(),
        ))?;

    rusoto.patch_robot(&robot).await?;

    return Ok(actix_web::HttpResponse::Ok().json(robot));
}

pub async fn delete(
    request: actix_web::HttpRequest,
    robot_id_query: actix_web::web::Query<models::robot::RobotIdentifier>,
) -> Result<actix_web::HttpResponse, actix_web::Error> {
    let robot_id = robot_id_query.into_inner();

    robot_id.unique.ok_or(errors::error::Error::new(
        actix_web::http::StatusCode::BAD_REQUEST,
        "'_id' can't be 'None'".to_string(),
    ))?;

    let mongodb = request
        .app_data::<actix_web::web::Data<clients::mongodb::Client>>()
        .ok_or(errors::error::Error::new(
            actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
            "MongoDB client not found".to_string(),
        ))?;

    let result = mongodb.delete_robot(&robot_id).await.map_err(|error| {
        errors::error::Error::new(
            actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
            error.to_string(),
        )
    })?;

    if result.deleted_count < 1 {
        return Err(errors::error::Error::new(
            actix_web::http::StatusCode::NOT_FOUND,
            format!(
                "Robot with id {} couldn't be found",
                robot_id.unique.unwrap()
            ),
        )
        .into());
    }

    let rusoto = request
        .app_data::<actix_web::web::Data<clients::rusoto::Client>>()
        .ok_or(errors::error::Error::new(
            actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
            "Rusoto client not found".to_string(),
        ))?;

    rusoto.delete_robot(&robot_id.unique.unwrap()).await?;

    return Ok(actix_web::HttpResponse::Ok().finish());
}
