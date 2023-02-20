mod models;
mod routes;
mod utils;

use crate::models::robots::RobotBuilder;
use actix_rt::{spawn, time::sleep};
use actix_web::{
    middleware::Logger,
    web::{self, resource, Data},
    App, HttpServer,
};
use chrono::Utc;
use cron::Schedule;
use dotenv::dotenv;
use env_logger::{init_from_env, Env};
use routes::{projects, robots};
use std::{str::FromStr, time};
use utils::Client;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    init_from_env(Env::default().default_filter_or("info"));
    dotenv().ok();

    let client_service = Client::new();
    let client_scheduler = client_service.clone();
    let client_purger = client_service.clone();

    spawn(async move {
        let update_schedule = Schedule::from_str("0 * * * * * *").unwrap();
        let mut sleep_duration = time::Duration::from_secs(1);
        let mut last_update_datetime = Utc::now();
        loop {
            let now = Utc::now();
            if let Ok(duration) = last_update_datetime.signed_duration_since(now).to_std() {
                sleep_duration = duration;
            }

            println!(
                "[{:?}] Next robot iteration in {:?}",
                last_update_datetime, sleep_duration
            );

            sleep(sleep_duration).await;

            let update_datetime = update_schedule.upcoming(Utc).take(1).next().unwrap();
            if last_update_datetime >= update_datetime {
                continue;
            } else {
                last_update_datetime = update_datetime;
            }

            let mut robots = client_scheduler
                .get_robots(RobotBuilder::new().finish())
                .await;
            if robots.is_none() {
                continue;
            }

            println!(
                "[{:?}] Iterating {:?} robot(s)...",
                last_update_datetime,
                robots.as_ref().unwrap().len()
            );

            for robot in robots.as_mut().unwrap() {
                if !robot.think(&client_scheduler, now).await {
                    continue;
                }

                println!(
                    "[{:?}] {:?} has been updated.",
                    last_update_datetime, robot.name
                );

                if client_scheduler.patch_robot(&robot.into()).await.is_err() {
                    continue;
                }

                println!(
                    "[{:?}] {:?} has been patched.",
                    last_update_datetime, robot.name
                );
            }
        }
    });

    spawn(async move {
        let update_schedule = Schedule::from_str("0 * * * * * *").unwrap();
        let mut sleep_duration = time::Duration::from_secs(1);
        let mut last_update_datetime = Utc::now();
        loop {
            let now = Utc::now();
            if let Ok(duration) = last_update_datetime.signed_duration_since(now).to_std() {
                sleep_duration = duration;
            }

            println!(
                "[{:?}] Next user iteration in {:?}",
                last_update_datetime, sleep_duration
            );

            sleep(sleep_duration).await;

            let update_datetime = update_schedule.upcoming(Utc).take(1).next().unwrap();
            if last_update_datetime >= update_datetime {
                continue;
            } else {
                last_update_datetime = update_datetime;
            }

            let mut users = client_purger.get_users_to_purge().await;
            if users.is_none() {
                continue;
            }

            println!(
                "[{:?}] Iterating {:?} users(s)...",
                last_update_datetime,
                users.as_ref().unwrap().len()
            );

            for user in users.as_mut().unwrap() {
                if user.should_delete(now).await {
                    if client_purger.log_user(user).await.is_ok() {
                        if client_purger.delete_user(user).await.is_ok() {
                            println!(
                                "[{:?}] {:?} has been purged.",
                                last_update_datetime, user.display_name
                            );
                        }
                    }
                } else if user.should_email(now).await {
                    // client_purger.send_email(
                    //     user.purge_data
                    //         .as_ref()
                    //         .unwrap()
                    //         .robot
                    //         .notification_email
                    //         .clone(),
                    //     user.purge_data
                    //         .as_ref()
                    //         .unwrap()
                    //         .robot
                    //         .notification_password
                    //         .clone(),
                    //     user.display_name.clone(),
                    //     user.email.clone(),
                    // );
                    user.purge_data.as_mut().unwrap().alert = Some(now);
                    println!(
                        "[{:?}] {:?} has been emailed.",
                        last_update_datetime, user.display_name
                    );
                } else {
                    continue;
                }

                if client_purger.patch_user(&user).await.is_err() {
                    continue;
                }

                println!(
                    "[{:?}] {:?} has been patched.",
                    last_update_datetime, user.display_name
                );
            }
        }
    });

    return HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .app_data(Data::new(client_service.clone()))
            .service(
                resource("/robots")
                    .route(web::get().to(robots::get))
                    .route(web::post().to(robots::post))
                    .route(web::patch().to(robots::patch))
                    .route(web::delete().to(robots::delete)),
            )
            .service(resource("/projects").route(web::get().to(projects::get)))
    })
    .bind(("localhost", 8080))?
    .run()
    .await;
}
