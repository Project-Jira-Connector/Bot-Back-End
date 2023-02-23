use crate::*;

pub async fn get(
    request: actix_web::HttpRequest,
    query: actix_web::web::Json<models::robot::RobotQuery>,
) -> actix_web::HttpResponse {
    return match request
        .app_data::<actix_web::web::Data<utils::client::Client>>()
        .unwrap()
        .get_robots(query.into_inner())
        .await
    {
        Some(robots) => actix_web::HttpResponse::Ok().json(robots),
        None => actix_web::HttpResponse::InternalServerError().finish(),
    };
}

pub async fn post(
    request: actix_web::HttpRequest,
    form: actix_web::web::Json<models::robot::Robot>,
) -> actix_web::HttpResponse {
    return match request
        .app_data::<actix_web::web::Data<utils::client::Client>>()
        .unwrap()
        .add_robot(&form.into_inner())
        .await
    {
        Ok(result) => actix_web::HttpResponse::Ok().body(format!("{:?}", result)),
        Err(error) => actix_web::HttpResponse::InternalServerError().body(format!("{:?}", error)),
    };
}

pub async fn patch(
    request: actix_web::HttpRequest,
    query: actix_web::web::Json<models::robot::RobotQuery>,
) -> actix_web::HttpResponse {
    let robot_query = query.into_inner();
    if robot_query.id.is_none() {
        return actix_web::HttpResponse::BadRequest().finish();
    }

    return match request
        .app_data::<actix_web::web::Data<utils::client::Client>>()
        .unwrap()
        .patch_robot(&robot_query)
        .await
    {
        Ok(result) => actix_web::HttpResponse::Ok().body(format!("{:?}", result)),
        Err(error) => actix_web::HttpResponse::InternalServerError().body(format!("{:?}", error)),
    };
}

pub async fn delete(
    request: actix_web::HttpRequest,
    query: actix_web::web::Json<models::robot::RobotQuery>,
) -> actix_web::HttpResponse {
    let robot_query = query.into_inner();
    if robot_query.id.is_none() {
        return actix_web::HttpResponse::BadRequest().finish();
    }

    return match request
        .app_data::<actix_web::web::Data<utils::client::Client>>()
        .unwrap()
        .delete_robot(&robot_query)
        .await
    {
        Ok(result) => actix_web::HttpResponse::Ok().body(format!("{:?}", result)),
        Err(error) => actix_web::HttpResponse::InternalServerError().body(format!("{:?}", error)),
    };
}
