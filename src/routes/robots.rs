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
            actix_web::http::StatusCode::BAD_REQUEST,
            format!(
                "Robot with unique id ({:?}) doesn't exist",
                robot_id.unique.unwrap()
            ),
        ))?;

    let robot_config = rusoto
        .get_robot(&robot_data.id.unique.unwrap())
        .await
        .map_err(|error| {
            errors::error::Error::new(
                actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
                error.to_string(),
            )
        })?;

    return Ok(
        actix_web::HttpResponse::Ok().json(models::robot::Robot::new(robot_data, robot_config))
    );
}

pub async fn post(
    request: actix_web::HttpRequest,
    robot_json: actix_web::web::Json<models::robot::Robot>,
) -> Result<actix_web::HttpResponse, actix_web::Error> {
    let mut robot = robot_json.into_inner();

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

// pub async fn patch(
//     request: actix_web::HttpRequest,
//     robot: actix_web::web::Json<models::robot::RobotQuery>,
// ) -> Result<actix_web::HttpResponse, Box<dyn std::error::Error>> {
//     return Ok(actix_web::HttpResponse::Ok().json(
//         request
//             .app_data::<actix_web::web::Data<clients::rusoto::Client>>()
//             .ok_or("Rusoto client not found")?
//             .patch_robot(&robot.into_inner())
//             .await?,
//     ));
// }

// pub async fn delete(
//     request: actix_web::HttpRequest,
//     id: mongodb::bson::oid::ObjectId,
// ) -> Result<actix_web::HttpResponse, Box<dyn std::error::Error>> {
//     return Ok(actix_web::HttpResponse::Ok().json(
//         request
//             .app_data::<actix_web::web::Data<clients::rusoto::Client>>()
//             .ok_or("Rusoto client not found")?
//             .delete_robot(id)
//             .await?,
//     ));
// }
