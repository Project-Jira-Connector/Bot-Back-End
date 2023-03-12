use crate::*;

pub async fn get(
    request: actix_web::HttpRequest,
    robot: actix_web::web::Query<models::robot::RobotQuery>,
) -> actix_web::HttpResponse {
    return match request
        .app_data::<actix_web::web::Data<utils::client::Client>>()
        .unwrap()
        .get_robots_with(&robot)
        .await
    {
        Ok(robots) => actix_web::HttpResponse::Ok().json(robots),
        Err(error) => actix_web::HttpResponse::InternalServerError().body(format!("{:?}", error)),
    };
}

pub async fn post(
    request: actix_web::HttpRequest,
    robot: actix_web::web::Json<models::robot::Robot>,
) -> actix_web::HttpResponse {
    return match request
        .app_data::<actix_web::web::Data<utils::client::Client>>()
        .unwrap()
        .add_robot(&robot)
        .await
    {
        Ok(result) => actix_web::HttpResponse::Ok().json(result),
        Err(error) => actix_web::HttpResponse::InternalServerError().body(format!("{:?}", error)),
    };
}

pub async fn patch(
    request: actix_web::HttpRequest,
    robot: actix_web::web::Json<models::robot::RobotQuery>,
) -> actix_web::HttpResponse {
    let robot = robot.into_inner();
    if robot.id.is_none() {
        return actix_web::HttpResponse::BadRequest().finish();
    }

    return match request
        .app_data::<actix_web::web::Data<utils::client::Client>>()
        .unwrap()
        .patch_robot(&robot)
        .await
    {
        Ok(result) => actix_web::HttpResponse::Ok().json(result),
        Err(error) => actix_web::HttpResponse::InternalServerError().body(format!("{:?}", error)),
    };
}

pub async fn delete(
    request: actix_web::HttpRequest,
    robot: actix_web::web::Query<models::robot::RobotQuery>,
) -> actix_web::HttpResponse {
    let robot = robot.into_inner();
    if robot.id.is_none() {
        return actix_web::HttpResponse::BadRequest().finish();
    }

    return match request
        .app_data::<actix_web::web::Data<utils::client::Client>>()
        .unwrap()
        .delete_robot(&robot)
        .await
    {
        Ok(result) => actix_web::HttpResponse::Ok().json(result),
        Err(error) => actix_web::HttpResponse::InternalServerError().body(format!("{:?}", error)),
    };
}
