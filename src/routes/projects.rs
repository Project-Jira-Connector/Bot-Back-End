use crate::{
    models::projects::{Project, RoleData},
    utils::Client,
};
use actix_web::{
    web::{self, Data},
    HttpRequest, HttpResponse,
};
use std::collections::HashMap;

pub async fn get(request: HttpRequest) -> HttpResponse {
    let email = request.headers().get("email");
    if email.is_none() {
        return HttpResponse::Unauthorized().finish();
    }

    let api_key = request.headers().get("apiKey");
    if api_key.is_none() {
        return HttpResponse::Unauthorized().finish();
    }

    match request
        .app_data::<Data<Client>>()
        .unwrap()
        .get_projects(
            &String::from(email.unwrap().to_str().unwrap()),
            &String::from(api_key.unwrap().to_str().unwrap()),
        )
        .await
    {
        Some(projects) => return HttpResponse::Ok().json(projects),
        None => return HttpResponse::InternalServerError().finish(),
    };
}

// pub async fn get(request: HttpRequest) -> HttpResponse {
//     let email = request.headers().get("email");
//     if email.is_none() {
//         return HttpResponse::Unauthorized().finish();
//     }

//     let api_key = request.headers().get("apiKey");
//     if api_key.is_none() {
//         return HttpResponse::Unauthorized().finish();
//     }

//     let query = web::Query::<HashMap<String, String>>::from_query(request.query_string()).unwrap();

//     let account_id = query.get("accountId");
//     if account_id.is_none() {
//         return HttpResponse::BadRequest().finish();
//     }

//     let client = request.app_data::<Data<Client>>().unwrap();

//     let projects = client
//         .get_projects(
//             &String::from(email.unwrap().to_str().unwrap()),
//             &String::from(api_key.unwrap().to_str().unwrap()),
//         )
//         .await;
//     if projects.is_none() {
//         return HttpResponse::InternalServerError().finish();
//     }

//     let mut result: Vec<(Project, RoleData)> = vec![];

//     for project in projects.as_ref().unwrap() {
//         let role = client
//             .get_project_role(
//                 &String::from(email.unwrap().to_str().unwrap()),
//                 &String::from(api_key.unwrap().to_str().unwrap()),
//                 &project,
//             )
//             .await;
//         if role.is_none() {
//             continue;
//         }

//         if let Some(data) = client
//             .get_project_role_data(
//                 &String::from(email.unwrap().to_str().unwrap()),
//                 &String::from(api_key.unwrap().to_str().unwrap()),
//                 &role.as_ref().unwrap().viewer,
//             )
//             .await
//         {
//             for viewer in &data.actors {
//                 if &viewer.actor_user.account_id != account_id.unwrap() {
//                     continue;
//                 }
//                 result.push((project.clone(), data));
//                 break;
//             }
//         }

//         if let Some(data) = client
//             .get_project_role_data(
//                 &String::from(email.unwrap().to_str().unwrap()),
//                 &String::from(api_key.unwrap().to_str().unwrap()),
//                 &role.as_ref().unwrap().member,
//             )
//             .await
//         {
//             for member in &data.actors {
//                 if &member.actor_user.account_id != account_id.unwrap() {
//                     continue;
//                 }
//                 result.push((project.clone(), data));
//                 break;
//             }
//         }

//         if let Some(data) = client
//             .get_project_role_data(
//                 &String::from(email.unwrap().to_str().unwrap()),
//                 &String::from(api_key.unwrap().to_str().unwrap()),
//                 &role.as_ref().unwrap().administrator,
//             )
//             .await
//         {
//             for admin in &data.actors {
//                 if &admin.actor_user.account_id != account_id.unwrap() {
//                     continue;
//                 }
//                 result.push((project.clone(), data));
//                 break;
//             }
//         }
//     }

//     return HttpResponse::Ok().json(result);
// }
