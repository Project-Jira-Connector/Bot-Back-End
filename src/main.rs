mod models;
mod utils;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));

    let client_service = utils::client::Client::new();

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
            check_double_name: true,
            check_active_status: true,
            check_double_email: true,
            last_updated: chrono::Utc::now(),
        },
    };

    let users = client_service
        .get_jira_users(&robot.credential.cloud_session_token)
        .await;

    let mut robot_users: Vec<models::jira::User> = Vec::new();

    let project_roles = client_service
        .get_jira_project_roles(
            &robot.credential.platform_email,
            &robot.credential.platform_api_key,
        )
        .await;

    for project_role in project_roles {
        if project_role.scope.is_none() {
            continue;
        }

        if robot.credential.project_id != project_role.scope.unwrap().project.id {
            continue;
        }

        let role_actors = client_service
            .get_jira_project_role_actors(
                &robot.credential.platform_email,
                &robot.credential.platform_api_key,
                &robot.credential.project_id,
                project_role.id,
            )
            .await;

        for role_actor in role_actors {
            if !robot_users
                .iter()
                .any(|user| user.id == role_actor.actor_user.account_id)
            {
                let user = users
                    .iter()
                    .find(|user| user.id == role_actor.actor_user.account_id);
                if user.is_none() {
                    continue;
                }
                robot_users.push(user.unwrap().clone());
            }
        }
    }

    robot_users.sort_by_key(|user| user.created);

    let filtered_users = robot.filter_jira_user(&robot_users);

    println!("\n\nValid Users");
    for user in &robot_users {
        let purge_data = filtered_users.users.get(&user.id.clone());
        if purge_data.is_some() {
            continue;
        }

        println!(
            "[{:?} | {:?}] {:?}",
            &user.id, user.created, &user.display_name
        );
    }

    println!("\n\nInvalid Users");
    for user in &robot_users {
        let purge_data = filtered_users.users.get(&user.id.clone());
        if purge_data.is_none() {
            continue;
        }

        println!(
            "[{:?} | {:?}] {:?} | {:?}",
            &user.id,
            user.created,
            &user.display_name,
            purge_data.unwrap().reasons.data
        );
    }

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
