mod models;
mod routes;
mod utils;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Loads environment variables.
    dotenv::dotenv().ok();

    // Initializes the logging subsystem for the application.
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));

    // Get our environment variables
    let environment = models::config::Environment::new();

    // Creates a client to communicate with jira and mongodb.
    let client = utils::client::Client::new(
        &environment.database.username,
        &environment.database.password,
    ).await;

    // Run scheduler.
    let (scheduler_exit_sender, scheduler_exit_receiver) = tokio::sync::mpsc::channel(1);
    let scheduler_handle = actix_rt::spawn(utils::scheduler::run(
        client.clone(),
        environment.clone(),
        scheduler_exit_receiver,
    ));

    // Run server.
    actix_web::HttpServer::new(move || {
        actix_web::App::new()
            .wrap(actix_cors::Cors::permissive())
            .wrap(actix_web::middleware::Logger::default())
            .app_data(actix_web::web::Data::new(client.clone()))
            .service(
                actix_web::web::resource("/robots")
                    .route(actix_web::web::get().to(routes::robots::get))
                    .route(actix_web::web::post().to(routes::robots::post))
                    .route(actix_web::web::patch().to(routes::robots::patch))
                    .route(actix_web::web::delete().to(routes::robots::delete)),
            )
    })
    .bind((environment.server.address, environment.server.port))?
    .run()
    .await?;

    // Stop scheduler.
    scheduler_exit_sender.send(()).await.unwrap();
    scheduler_handle.await.unwrap();

    return Ok(());
}
