mod models;
mod utils;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));

    let robot = models::robot::Robot {
        id: mongodb::bson::oid::ObjectId::new(),
        info: models::robot::RobotInfo {
            name: String::from("Peter"),
            description: String::from("Help"),
        },
        credential: models::robot::RobotCredential {
            platform_email: std::env::var("PLATFORM_EMAIL").unwrap(),
            platform_api_key: std::env::var("PLATFORM_API_KEY").unwrap(),
            platform_type: models::robot::PlatformType::Cloud,
            cloud_session_token: std::env::var("CLOUD_SESSION_TOKEN").unwrap(),
            project_id: String::from("10025"),
        },
        scheduler: models::robot::RobotScheduler {
            active: false,
            delay: 14,
            last_active: 14,
            check_double_name: false,
            check_active_status: false,
            check_double_email: false,
            last_updated: chrono::Utc::now(),
        },
    };

    let client_service = utils::client::Client::new();

    let project_roles = client_service
        .get_jira_project_roles(
            &robot.credential.platform_email,
            &robot.credential.platform_api_key,
        )
        .await;

    println!("{:?}", project_roles);

    // let mut users = client_service
    //     .get_jira_users(&std::env::var("CLOUD_SESSION_TOKEN").unwrap())
    //     .await;
    // users.sort_by_key(|e| e.created);

    // let filtered_users = robot.filter_jira_user(&users);

    // println!("{:?}", filtered_users);

    return actix_web::HttpServer::new(move || {
        actix_web::App::new()
            .wrap(actix_web::middleware::Logger::default())
            .app_data(actix_web::web::Data::new(client_service.clone()))
    })
    .bind(format!(
        "{}:{}",
        std::env::var("BIND_ADDRESS").unwrap(),
        std::env::var("BIND_PORT").unwrap()
    ))?
    .run()
    .await;
}
