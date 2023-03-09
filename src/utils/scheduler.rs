use crate::*;

pub async fn robots(client: utils::client::Client, schedule: cron::Schedule) {
    const DEFAULT_DURATION: i64 = 1;
    let mut sleep_duration = std::time::Duration::from_secs(DEFAULT_DURATION as u64);
    let mut last_upcoming_datetime = chrono::Utc::now();
    actix_rt::spawn(async move {
        loop {
            if let Ok(new_duration) = last_upcoming_datetime
                .signed_duration_since(chrono::Utc::now())
                .to_std()
            {
                sleep_duration = new_duration;
            }

            println!(
                "[{:?}] Next robot iteration in {:?}",
                last_upcoming_datetime, sleep_duration
            );

            actix_rt::time::sleep(sleep_duration).await;

            if let Some(upcoming_datetime) = schedule.upcoming(chrono::Utc).take(1).next() {
                if last_upcoming_datetime >= upcoming_datetime {
                    continue;
                } else {
                    last_upcoming_datetime = upcoming_datetime;
                }
            } else {
                let now = chrono::Utc::now();
                println!("[{:?}] Failed to determine next schedule", now);
                last_upcoming_datetime = now + chrono::Duration::seconds(DEFAULT_DURATION);
                continue;
            }

            let robots = client.get_robots(&models::robot::RobotQuery::new()).await;
            if robots.is_err() {
                continue;
            }
            let mut robots = robots.unwrap();

            println!(
                "[{:?}] Iterating {:?} robot(s)...",
                last_upcoming_datetime,
                robots.len()
            );

            for robot in &mut robots {
                if !robot.update(&client, last_upcoming_datetime).await {
                    continue;
                }

                println!(
                    "[{:?}] {:?} has been updated.",
                    last_upcoming_datetime, robot.info.name
                );

                match client.patch_robot(&robot.into()).await {
                    Ok(result) => {
                        if result.modified_count > 0 {
                            println!(
                                "[{:?}] {:?} has been patched.",
                                last_upcoming_datetime, robot.info.name
                            );
                        } else {
                            println!(
                                "[{:?}] Failed to patch {:?} (Not Found).",
                                last_upcoming_datetime, robot.info.name
                            );
                        }
                    }
                    Err(error) => {
                        println!(
                            "[{:?}] Failed to patch {:?} ({}).",
                            last_upcoming_datetime, robot.info.name, error
                        );
                    }
                }
            }
        }
    });
}

pub async fn purger(
    client: utils::client::Client,
    schedule: cron::Schedule,
    notification_email: String,
    notification_password: String,
) {
    const DEFAULT_DURATION: i64 = 1;
    let mut sleep_duration = std::time::Duration::from_secs(DEFAULT_DURATION as u64);
    let mut last_upcoming_datetime = chrono::Utc::now();
    actix_rt::spawn(async move {
        loop {
            if let Ok(new_duration) = last_upcoming_datetime
                .signed_duration_since(chrono::Utc::now())
                .to_std()
            {
                sleep_duration = new_duration;
            }

            println!(
                "[{:?}] Next users iteration in {:?}",
                last_upcoming_datetime, sleep_duration
            );

            actix_rt::time::sleep(sleep_duration).await;

            if let Some(update_datetime) = schedule.upcoming(chrono::Utc).take(1).next() {
                if last_upcoming_datetime >= update_datetime {
                    continue;
                } else {
                    last_upcoming_datetime = update_datetime;
                }
            } else {
                let now = chrono::Utc::now();
                println!("[{:?}] Failed to determine next schedule", now);
                last_upcoming_datetime = now + chrono::Duration::seconds(DEFAULT_DURATION);
                continue;
            }

            let purges = client.get_purges().await;
            if purges.is_err() {
                continue;
            }
            let mut purges = purges.unwrap();

            println!(
                "[{:?}] Iterating {:?} user(s)...",
                last_upcoming_datetime,
                purges.len()
            );

            for purge in &mut purges {
                if purge.should_remove_user(last_upcoming_datetime) {
                    let robot = client
                        .get_robot(&models::robot::RobotQuery {
                            id: Some(purge.robot.id),
                            info: models::robot::RobotInfoQuery {
                                name: None,
                                description: None,
                            },
                            credential: models::robot::RobotCredentialQuery {
                                platform_email: None,
                                platform_api_key: None,
                                platform_type: None,
                                cloud_session_token: None,
                            },
                            scheduler: models::robot::RobotSchedulerQuery {
                                active: None,
                                delay: None,
                                last_active: None,
                                check_double_name: None,
                                check_double_email: None,
                                check_active_status: None,
                                last_updated: None,
                            },
                        })
                        .await;
                    if robot.is_err() || robot.as_ref().unwrap().is_none() {
                        continue;
                    }
                    //let robot = robot.unwrap().unwrap();
                    if purge.remove_user() {}
                } else if purge.should_email_user(last_upcoming_datetime) {
                    //if purge.email_user(&notification_email, &notification_password) {
                    purge.alert = last_upcoming_datetime;
                    if client.patch_purge(purge).await.is_ok() {
                        println!(
                            "[{:?}] {:?} has been notified.",
                            last_upcoming_datetime, purge.user.email
                        );
                    }
                    //}
                }
            }
        }
    });
}
