use crate::{
    models::robots::{RobotForm, RobotQuery},
    utils::Client,
};
use actix_web::{
    web::Data,
    web::{Form, Query},
    HttpRequest, HttpResponse,
};

pub async fn get(request: HttpRequest, query: Query<RobotQuery>) -> HttpResponse {
    match request
        .app_data::<Data<Client>>()
        .unwrap()
        .get_robots(query.into_inner())
        .await
    {
        Some(robots) => return HttpResponse::Ok().json(robots),
        None => return HttpResponse::InternalServerError().finish(),
    };
}

pub async fn post(request: HttpRequest, form: Form<RobotForm>) -> HttpResponse {
    match request
        .app_data::<Data<Client>>()
        .unwrap()
        .add_robot(&form.into_inner().into())
        .await
    {
        Ok(result) => return HttpResponse::Ok().body(format!("{:?}", result)),
        Err(error) => return HttpResponse::InternalServerError().body(format!("{:?}", error)),
    };
}

pub async fn patch(request: HttpRequest, query: Query<RobotQuery>) -> HttpResponse {
    match request
        .app_data::<Data<Client>>()
        .unwrap()
        .patch_robot(&query.into_inner())
        .await
    {
        Ok(result) => return HttpResponse::Ok().body(format!("{:?}", result)),
        Err(error) => return HttpResponse::InternalServerError().body(format!("{:?}", error)),
    };
}

pub async fn delete(request: HttpRequest, query: Query<RobotQuery>) -> HttpResponse {
    let robot_query = query.into_inner();
    if robot_query.id.is_none() {
        return HttpResponse::BadRequest().finish();
    }

    match request
        .app_data::<Data<Client>>()
        .unwrap()
        .delete_robot(&robot_query)
        .await
    {
        Ok(result) => return HttpResponse::Ok().body(format!("{:?}", result)),
        Err(error) => return HttpResponse::InternalServerError().body(format!("{:?}", error)),
    };
}
