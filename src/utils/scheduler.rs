// Import everything from the crate's root module
use crate::*;
// Import the necessary traits from the rayon crate
use rayon::prelude::*;

async fn tick(client: &utils::client::Client, now: chrono::DateTime<chrono::Utc>) -> Option<()> {
    // Call the `get_robots` method on the `Client` object to retrieve a list of robots.
    let robots = client.get_robots().await.ok()?;

    log::info!("Found {} robot(s)", robots.len());

    // Use `into_iter()` to consume the `robots` vector and `filter()` to create a new vector called `active_robots`
    // that only includes robots that needs to be updated.
    let mut robots = robots
        .par_iter()
        .filter(|robot| {
            // Do not include inactive robot
            if !robot.scheduler.active {
                return false;
            }
            // // Do not include up to date robot
            // if let Some(last_updated) = robot.scheduler.last_updated {
            //     if now <= last_updated + chrono::Duration::days(robot.scheduler.schedule) {
            //         return false;
            //     }
            // }
            return true;
        })
        .collect::<Vec<_>>();

    log::info!("Iterating {} robot(s)", robots.len());

    robots.par_iter_mut().for_each(|robot| {
        // Get all jira users
        let mut users = client.get_jira_users(&robot.credential.cloud_session_token);

        // Sort all jira users base on created key
        users.sort_by_key(|user| user.created);

        // Get all jira users with duplicate attributes
        let duplicate_users = users
            .par_iter()
            .enumerate()
            .flat_map(|(i, user)| {
                return users
                    .par_iter()
                    .enumerate()
                    .skip(i + 1) // start from the next element
                    .filter_map(move |(_j, other_user)| {
                        if strsim::normalized_damerau_levenshtein(
                            &user.display_name,
                            &other_user.display_name,
                        ) > 0.7
                        {
                            if user.created <= other_user.created {
                                return Some((
                                    other_user,
                                    models::purge::PurgeReason::DuplicateName,
                                ));
                            }
                            return Some((user, models::purge::PurgeReason::DuplicateName));
                        } else {
                            if strsim::normalized_damerau_levenshtein(
                                &user.email,
                                &other_user.email,
                            ) > 0.7
                            {
                                if user.created <= other_user.created {
                                    return Some((
                                        other_user,
                                        models::purge::PurgeReason::DuplicateEmail,
                                    ));
                                }
                                return Some((user, models::purge::PurgeReason::DuplicateEmail));
                            } else {
                                return None;
                            }
                        }
                    });
            })
            .collect::<Vec<_>>();

        // Get all inactive jira users
        let inactive_users = users
            .par_iter()
            .flat_map(|user| {
                // Check active status
                if robot.scheduler.check_active_status && !user.active {
                    return Some((user, models::purge::PurgeReason::ActiveStatus));
                }
                // Check presence
                let presence = user.presence.unwrap_or_else(|| {
                    if user.invitation_status.is_some() {
                        return user.invitation_status.as_ref().unwrap().invited_at;
                    }
                    return user.created;
                });
                if robot.scheduler.last_active > 0
                    && presence <= now - chrono::Duration::days(robot.scheduler.last_active)
                {
                    return Some((user, models::purge::PurgeReason::LastActive));
                }
                return None;
            })
            .collect::<Vec<_>>();

        log::info!(
            "Robot {:?} found {:?} user(s) with {:?} duplicate user(s) and {:?} inactive user(s)",
            robot.info.name,
            users.len(),
            duplicate_users.len(),
            inactive_users.len()
        );
    });

    return Some(());
}

pub async fn run(
    client: utils::client::Client,
    schedule: cron::Schedule,
    mut exit_rx: tokio::sync::mpsc::Receiver<()>,
) {
    // Initialize a variable to track the time of the last run.
    let mut last_run: Option<chrono::DateTime<chrono::Utc>> = None;
    loop {
        // Get the current time.
        let now = chrono::Utc::now();

        // If this is the first run or the schedule has elapsed since the last run, run `tick`.
        if last_run.is_none() || schedule.after(&last_run.unwrap()).next().unwrap() <= now {
            tick(&client, now).await;
            last_run = Some(now);
        }

        // Calculate the amount of time to sleep until the next scheduled run.
        let sleep_duration: chrono::Duration = match schedule.after(&last_run.unwrap()).next() {
            Some(next_run) => next_run - now,
            None => chrono::Duration::from_std(std::time::Duration::from_secs(1)).unwrap(),
        };

        // Wait until it's time for the next scheduled run or until the exit receiver receives a message.
        tokio::select! {
            _ = actix_rt::time::sleep(sleep_duration.to_std().unwrap()) => {},
            _ = exit_rx.recv() => {
                log::info!("shutting down idle scheduler");
                break;
            }
        }
    }
}
