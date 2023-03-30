// Import everything from the crate's root module
use crate::*;
// Import the necessary traits from the rayon crate
use rayon::prelude::*;

async fn get_robots(
    mongodb: &clients::mongodb::Client,
    rusoto: &clients::rusoto::Client,
) -> Result<Vec<models::robot::Robot>, Box<dyn std::error::Error>> {
    let robots_data = mongodb.get_robots().await?;
    let mut robots = Vec::<models::robot::Robot>::with_capacity(robots_data.len());
    for robot_data in robots_data {
        if let Ok(robot_config) = rusoto.get_robot(&robot_data.id.unique.unwrap()).await {
            robots.push(models::robot::Robot::new(robot_data, robot_config));
        }
    }
    return Ok(robots);
}

fn get_active_robots(robots: &mut Vec<models::robot::Robot>) -> Vec<&mut models::robot::Robot> {
    return robots
        .par_iter_mut()
        .filter(|robot| robot.data.active)
        .collect::<Vec<_>>();
}

async fn tick(
    now: chrono::DateTime<chrono::Utc>,
    reqwest: &clients::reqwest::Client,
    mongodb: &clients::mongodb::Client,
    rusoto: &clients::rusoto::Client,
    notification_config: &configs::notification::Config,
) -> Result<(), Box<dyn std::error::Error>> {
    // Call the `get_robots` method on the `Client` object to retrieve a list of robots.
    let mut robots = get_robots(mongodb, rusoto).await?;

    // Filter inactive robots.
    let mut active_robots = get_active_robots(&mut robots);

    // Create a vector to hold async tasks that we'll run concurrently
    let futures = active_robots
        .iter_mut()
        .map(|robot| async move {
            // Get all jira users
            let mut users = reqwest
                .get_jira_users(&robot.config.credential.cloud_session_token)
                .await;

            // Sort all jira users base on created key
            users.sort_by_key(|user| user.created);

            if !robot.is_updated(now) {
                // Change the robot status to become updated
                robot.data.modified = Some(now);
                if let Err(error) = mongodb.patch_robot(robot).await {
                    log::error!("{}", error);
                }

                //routes::report::report(notification_config.email, &robot.config.credential.platform_email, &robot.data.name, )

                // Get all jira users with duplicate attributes
                let duplicate_users = users
                    .par_iter()
                    .enumerate()
                    .flat_map(|(i, user)| {
                        return users
                            .par_iter()
                            .enumerate()
                            .skip(i + 1) // start from the next element
                            .filter_map(|(_j, other_user)| {
                                let mut reasons: std::collections::HashSet<models::purge::PurgeReason> =
                                    std::collections::HashSet::new();
                                if robot.config.scheduler.check_double_name {
                                    let similarity = strsim::normalized_damerau_levenshtein(
                                        &user.display_name,
                                        &other_user.display_name,
                                    );
                                    if similarity >= robot.config.scheduler.double_name_threshold.into() {
                                        reasons.insert(models::purge::PurgeReason::DuplicateName);
                                    }
                                }
                                if robot.config.scheduler.check_double_email {
                                    let similarity = strsim::normalized_damerau_levenshtein(
                                        &user.email,
                                        &other_user.email,
                                    );
                                    if similarity >= robot.config.scheduler.double_email_threshold.into()  {
                                        reasons.insert(models::purge::PurgeReason::DuplicateEmail);
                                    }
                                }
                                if reasons.is_empty() {
                                    return None;
                                }
                                if user.created <= other_user.created {
                                    return Some((other_user, reasons));
                                }
                                return Some((user, reasons));
                            });
                    })
                    .collect::<Vec<_>>();

                // Get all inactive jira users
                let inactive_users = users
                    .par_iter()
                    .flat_map(|user| {
                        let mut reasons: std::collections::HashSet<models::purge::PurgeReason> =
                            std::collections::HashSet::new();

                        // Check active status
                        if robot.config.scheduler.check_active_status && !user.active {
                            reasons.insert(models::purge::PurgeReason::ActiveStatus);
                        }

                        // Check presence
                        if robot.config.scheduler.last_active > 0
                            && user.get_available_presence()
                                <= now - chrono::Duration::days(robot.config.scheduler.last_active)
                        {
                            reasons.insert(models::purge::PurgeReason::LastActive);
                        }

                        if reasons.is_empty() {
                            return None;
                        }

                        return Some((user, reasons));
                    })
                    .collect::<Vec<_>>();

                let mut filtered_users: Vec<(
                    &models::jira::User,
                    std::collections::HashSet<models::purge::PurgeReason>,
                )> = Vec::new();
                filtered_users.extend(duplicate_users);
                filtered_users.extend(inactive_users);

                // Combine or remove duplicate users data since we seperate the loop between duplicate and inactivity
                let unique_filtered_users = filtered_users
                    .into_iter()
                    .fold(
                        std::collections::HashMap::<
                            String,
                            (
                                &models::jira::User,
                                std::collections::HashSet<models::purge::PurgeReason>,
                            ),
                        >::new(),
                        |mut data, (user, reason)| {
                            data.entry(user.id.clone())
                                .and_modify(|(_existing_user, existing_reason)| {
                                    // If a user with the same ID already exists, append their reason
                                    existing_reason.extend(reason.clone());
                                })
                                .or_insert((user, reason));
                            return data;
                        },
                    )
                    .into_iter()
                    .map(|(_, user)| user)
                    .collect::<Vec<_>>();
                
                // Add users to purge users queue
                for (user, reasons) in unique_filtered_users {
                    let purge_data = models::purge::PurgeData::new(
                        robot,
                        user,
                        reasons.into_iter().collect(),
                        now + chrono::Duration::days(7),
                    );

                    match mongodb.add_purge_user(&purge_data).await {
                        Ok(result) => {
                            if result.upserted_id.is_some() {
                                log::info!(
                                    "User {:?} has been queued by robot {:?} for purging because of the following reason(s): {:?}",
                                    user.display_name,
                                    robot.data.name,
                                    purge_data.reasons,
                                );
                            }
                        },
                        Err(error) => {
                            log::error!("Robot {:?} failed to queue user {:?} ({})", robot.data.name, user.display_name, error);
                        }
                    }
                }
            }

            if let Ok(mut purge_data) = mongodb.get_purge_users().await {
                let purge_data = purge_data
                .par_iter_mut()
                .filter(|purge_user| {
                    return purge_user.robot.id == robot.data.id.unique.unwrap();
                })
                .collect::<Vec<_>>();

                for data in purge_data {
                    if let Some(user) = users.iter().find(|user|user.id == data.user.id) {
                        let mut remove = false;
                        for reason in &data.reasons {
                            match *reason {
                                models::purge::PurgeReason::ActiveStatus => {
                                    if !user.active {
                                        remove = true;
                                    }
                                },
                                models::purge::PurgeReason::DuplicateEmail => {
                                    if user.email == data.user.email {
                                        remove = true;
                                    }
                                },
                                models::purge::PurgeReason::DuplicateName => {
                                    if user.display_name == data.user.display_name {
                                        remove = true;
                                    }
                                },
                                models::purge::PurgeReason::LastActive => {
                                    if user.get_available_presence() <= data.user.presence {
                                        remove = true;
                                    }
                                },
                            }
                            if remove == true {
                                break;
                            }
                        }

                        if !remove { // If there isn't any reason to have this user in purging queue anymore, remove it
                            if data.should_remove_user(now) {
                                match mongodb.delete_purge_user(data).await {
                                    Ok(result) => {
                                        if result.deleted_count > 0 {
                                            log::info!("Robot {:?} has remove user {:?} from purging queue (Clean)", robot.data.name, user.display_name);
                                        }
                                    },
                                    Err(error) => {
                                        log::error!("Robot {:?} failed to remove user {:?} from purging queue (Clean)({})", robot.data.name, user.display_name, error);
                                    }
                                }
                            }
                            continue;
                        }

                        if data.should_remove_user(now) {
                            // Log removed user
                            // Remove purge_data from purge_users database
                            // Remove user from jira
                            match mongodb.add_purge_log(&models::purge::PurgeLog::new(robot, user, data.reasons.clone(), now)).await {
                                Ok(_result) => match mongodb.delete_purge_user(data).await {
                                    Ok(result) => {
                                        if result.deleted_count > 0 {
                                            //match reqwest.remove_user_from_jira(robot, data).await {
                                               // Ok(_result) => {
                                                    log::info!("Robot {:?} has remove user {:?} from organization", robot.data.name, user.display_name);
                                                //},
                                                //Err(error) => {
                                                    //log::error!("Robot {:?} failed to remove user {:?} from organization ({:?})", robot.data.name, data.user.display_name, error);
                                                //}
                                            //}
                                        }
                                        else {
                                            log::warn!("Robot {:?} failed to find user {:?} from purging queue", robot.data.name, data.user.display_name);
                                        }
                                    },
                                    Err(error) => {
                                        log::error!("Robot {:?} failed to remove user {:?} from purging queue ({:?})", robot.data.name, data.user.display_name, error);
                                    }
                                },
                                Err(error) => {
                                    log::error!("Robot {:?} failed to log user {:?} removal ({:?})", robot.data.name, data.user.display_name, error);
                                }
                            }
                        }
                        else if data.should_email_user(now, 3) {
                            // Patch purge alert
                            // Email user
                            data.alert = Some(now);
                            if let Ok(result) = mongodb.patch_purge_user(data).await {
                                if result.modified_count > 0 {
                                    if data.email_user(&notification_config.email, &notification_config.password, &robot.config.credential.platform_email) {
                                        log::info!("Robot {:?} has notified user {:?} through {:?}", robot.data.name, user.display_name, user.email);
                                    }
                                }
                            }
                        }
                    }
                    else { // If user doesn't exist in jira anymore, we use purge_data.time to remove from database
                        if data.should_remove_user(now) {
                            match mongodb.delete_purge_user(data).await {
                                Ok(result) => {
                                    if result.deleted_count > 0 {
                                        log::info!("Robot {:?} successfully remove user {:?} from purging queue", robot.data.name, data.user.display_name);
                                    }
                                    else {
                                        log::warn!("Robot {:?} failed to find user {:?} from purging queue", robot.data.name, data.user.display_name);
                                    }
                                },
                                Err(error) => {
                                    log::error!("Robot {:?} failed to remove user {:?} from purging queue ({:?})", robot.data.name, data.user.display_name, error);
                                }
                            }
                        }
                    }
                }
            }
            else {
                log::error!("Robot {:?} failed to retrieve queued users", robot.data.name);
            }
        })
        .collect::<Vec<_>>();

    // Wait for all the task to finish
    futures::future::join_all(futures).await;

    return Ok(());
}

pub async fn run(
    mut exit_receiver: tokio::sync::mpsc::Receiver<()>,
    scheduler_config: configs::scheduler::Config,
    notification_config: configs::notification::Config,
    reqwest: clients::reqwest::Client,
    mongodb: clients::mongodb::Client,
    rusoto: clients::rusoto::Client,
) {
    // Initialize a variable to track the time of the last run.
    let mut last_run: Option<chrono::DateTime<chrono::Utc>> = None;
    loop {
        // Get the current time.
        let now = chrono::Utc::now();

        // If this is the first run or the schedule has elapsed since the last run, run `tick`.
        if last_run.is_none()
            || scheduler_config
                .schedule
                .after(&last_run.unwrap())
                .next()
                .unwrap()
                <= now
        {
            if let Err(error) = tick(now, &reqwest, &mongodb, &rusoto, &notification_config).await {
                log::error!("{}", error)
            }
            last_run = Some(now);
        }

        // Calculate the amount of time to sleep until the next scheduled run.
        let sleep_duration: chrono::Duration =
            match scheduler_config.schedule.after(&last_run.unwrap()).next() {
                Some(next_run) => next_run - now,
                None => chrono::Duration::from_std(std::time::Duration::from_secs(1)).unwrap(),
            };

        // Wait until it's time for the next scheduled run or until the exit receiver receives a message.
        tokio::select! {
            _ = actix_rt::time::sleep(sleep_duration.to_std().unwrap()) => {},
            _ = exit_receiver.recv() => {
                log::info!("shutting down idle scheduler");
                break;
            }
        }
    }
}
