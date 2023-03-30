use crate::*;
use rayon::prelude::*;

// pub fn report(
//     sender_email: &String,
//     recipient_email: &String,
//     recipient_name: &String,
//     body: String,
// ) {
//     let message = lettre::Message::builder()
//         .from(
//             format!("Telkom Developer Network <{}>", sender_email)
//                 .parse()
//                 .unwrap(),
//         )
//         .to(format!("{} <{}>", recipient_name, recipient_email)
//             .parse()
//             .unwrap())
//         .subject("[LOG] Jira Report")
//         .singlepart(
//             lettre::message::Attachment::new(String::from("data.json"))
//                 .body(body, "text/json".parse().unwrap()),
//         )
//         .unwrap();
// }

pub async fn get_report(
    mongodb: &clients::mongodb::Client,
    generator: &models::report::Generator,
) -> Result<models::report::Report, Box<dyn std::error::Error>> {
    let purge_data = mongodb
        .get_purge_users()
        .await?
        .into_par_iter()
        .filter(|purge_user| {
            return purge_user.robot.id == generator.robot_id.unique.unwrap();
        })
        .collect::<Vec<_>>();

    let purge_log = mongodb.get_purge_log().await?;

    return Ok(models::report::Report::new(purge_data, purge_log));
}

pub async fn get(
    request: actix_web::HttpRequest,
    generator_query: actix_web::web::Query<models::report::Generator>,
) -> Result<actix_web::HttpResponse, actix_web::Error> {
    let generator = generator_query.into_inner();

    generator.robot_id.unique.ok_or(errors::error::Error::new(
        actix_web::http::StatusCode::BAD_REQUEST,
        "'_id' can't be 'None'".to_string(),
    ))?;

    let mongodb = request
        .app_data::<actix_web::web::Data<clients::mongodb::Client>>()
        .ok_or(errors::error::Error::new(
            actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
            "MongoDB client not found".to_string(),
        ))?;

    let report = get_report(mongodb, &generator).await.map_err(|error| {
        errors::error::Error::new(
            actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
            error.to_string(),
        )
    })?;

    return Ok(actix_web::HttpResponse::Created().json(report));
}
