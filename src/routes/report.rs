use crate::*;
use rayon::prelude::*;

// pub fn purgedata_to_csv(
//     queue_users: &Vec<models::purge::PurgeData>,
// ) -> Result<String, Box<dyn std::error::Error>> {
//     let mut writer = csv::Writer::from_writer(vec![]);
//     for user in queue_users {
//         writer.serialize(user)?;
//         writer.write_record(None::<&[u8]>)?;
//     }
//     return Ok(String::from_utf8(writer.into_inner()?)?);
// }

// pub fn purgelog_to_csv(
//     removed_users: &Vec<models::purge::PurgeLog>,
// ) -> Result<String, Box<dyn std::error::Error>> {
//     let mut writer = csv::Writer::from_writer(vec![]);
//     for user in removed_users {
//         writer.serialize(user)?;
//         writer.write_record(None::<&[u8]>)?;
//     }
//     return Ok(String::from_utf8(writer.into_inner()?)?);
// }

// pub fn report_to(
//     sender_email: String,
//     sender_password: String,
//     recipient_email: &String,
//     queued_users: String,
//     removed_users: String,
// ) -> Result<(), Box<dyn std::error::Error>> {
//     let queued_users_attachment = lettre::message::Attachment::new("queue_users.csv".to_string())
//         .body(queued_users, "text/csv".parse()?);

//     let removed_users_attachment =
//         lettre::message::Attachment::new("removed_users.csv".to_string())
//             .body(removed_users, "text/csv".parse()?);

//     let message = lettre::Message::builder()
//         .from(
//             format!("Telkom Developer Network <{}>", "teamunityfx2020@gmail.com")
//                 .parse()
//                 .unwrap(),
//         )
//         .to(format!("Someone <{}>", recipient_email).parse()?)
//         .subject("[LOG] Jira Purge Users Report")
//         .multipart(
//             lettre::message::MultiPart::related()
//                 .singlepart(queued_users_attachment)
//                 .singlepart(removed_users_attachment),
//         )?;

//     let relay = lettre::SmtpTransport::relay("smtp.gmail.com")?
//         .credentials(lettre::transport::smtp::authentication::Credentials::new(
//             sender_email,
//             sender_password,
//         ))
//         .build();

//     lettre::Transport::send(&relay, &message)?;

//     return Ok(());
// }

pub fn report_to(
    sender_email: String,
    sender_password: String,
    recipient_email: &String,
    attachment: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let attachment = lettre::message::Attachment::new("data.json".to_string())
        .body(attachment, "text/json".parse()?);

    let message = lettre::Message::builder()
        .from(
            format!("Telkom Developer Network <{}>", sender_email)
                .parse()
                .unwrap(),
        )
        .to(format!("Me <{}>", recipient_email).parse()?)
        .subject("[LOG] Jira Purge Users Report")
        .singlepart(attachment)?;

    let relay = lettre::SmtpTransport::relay("smtp.gmail.com")?
        .credentials(lettre::transport::smtp::authentication::Credentials::new(
            sender_email,
            sender_password,
        ))
        .build();

    lettre::Transport::send(&relay, &message)?;

    return Ok(());
}

pub async fn get_report(
    mongodb: &clients::mongodb::Client,
    robot_id: &models::robot::RobotIdentifier,
) -> Result<models::report::Report, Box<dyn std::error::Error>> {
    let purge_data = mongodb
        .get_purge_users()
        .await?
        .into_par_iter()
        .filter(|purge_user| {
            return purge_user.robot.id == robot_id.unique.unwrap();
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

    let report = get_report(mongodb, &generator.robot_id)
        .await
        .map_err(|error| {
            errors::error::Error::new(
                actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
                error.to_string(),
            )
        })?;

    // let queued_users_csv = purgedata_to_csv(&report.queued).map_err(|error| {
    //     errors::error::Error::new(
    //         actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
    //         error.to_string(),
    //     )
    // })?;

    // let removed_users_csv = purgelog_to_csv(&report.removed).map_err(|error| {
    //     errors::error::Error::new(
    //         actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
    //         error.to_string(),
    //     )
    // })?;

    report_to(
        "teamunityfx2020@gmail.com".to_string(),
        "kzazjzklgwthrtju".to_string(),
        &generator.email,
        format!("{:#?}", report),
    )
    .map_err(|error| {
        errors::error::Error::new(
            actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
            error.to_string(),
        )
    })?;

    return Ok(actix_web::HttpResponse::Created().json(report));
}
