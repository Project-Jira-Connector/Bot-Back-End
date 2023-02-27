use crate::*;

pub async fn robots(client: utils::client::Client) {
    actix_rt::spawn(async move {
        let update_schedule =
            <cron::Schedule as std::str::FromStr>::from_str("0 * * * * * *").unwrap();
        let mut sleep_duration = std::time::Duration::from_secs(1);
        let mut last_update_datetime = chrono::Utc::now();
        loop {
            let now = chrono::Utc::now();
            if let Ok(duration) = last_update_datetime.signed_duration_since(now).to_std() {
                sleep_duration = duration;
            }

            println!(
                "[{:?}] Next robot iteration in {:?}",
                last_update_datetime, sleep_duration
            );

            actix_rt::time::sleep(sleep_duration).await;

            let update_datetime = update_schedule
                .upcoming(chrono::Utc)
                .take(1)
                .next()
                .unwrap();
            if last_update_datetime >= update_datetime {
                continue;
            } else {
                last_update_datetime = update_datetime;
            }

            let mut robots = client.get_robots(models::robot::RobotQuery::new()).await;
            if robots.is_none() {
                continue;
            }

            println!(
                "[{:?}] Iterating {:?} robot(s)...",
                last_update_datetime,
                robots.as_ref().unwrap().len()
            );

            for robot in robots.as_mut().unwrap() {
                if !robot.update(&client, update_datetime).await {
                    continue;
                }

                println!(
                    "[{:?}] {:?} has been updated.",
                    update_datetime, robot.info.name
                );

                match client.patch_robot(&robot.into()).await {
                    Ok(result) => {
                        if result.modified_count > 0 {
                            println!(
                                "[{:?}] {:?} has been patched.",
                                update_datetime, robot.info.name
                            );
                        } else {
                            println!(
                                "[{:?}] Failed to patch {:?} (Not Found).",
                                update_datetime, robot.info.name
                            );
                        }
                    }
                    Err(error) => {
                        println!(
                            "[{:?}] Failed to patch {:?} ({}).",
                            update_datetime, robot.info.name, error
                        );
                    }
                }
            }
        }
    });
}
