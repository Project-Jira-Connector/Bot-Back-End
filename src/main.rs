mod clients;
mod configs;
mod errors;
mod models;
mod routes;
mod utils;

#[actix_web::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Loads environment variables.
    dotenv::dotenv()?;

    // Initializes the logging subsystem for the application.
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));

    // Get our environment variables.
    let mongodb_config = configs::mongodb::Config::new()?;
    let rusoto_config = configs::rusoto::Config::new()?;
    let server_config = configs::server::Config::new()?;

    // Creates a client to communicate with jira, mongodb and digitalocean.
    let reqwest_client = reqwest::Client::new();
    let mongodb_client = clients::mongodb::Client::new(mongodb_config).await?;
    let rusoto_client = clients::rusoto::Client::new(rusoto_config)?;

    // Run scheduler.
    // let (scheduler_exit_sender, scheduler_exit_receiver) = tokio::sync::mpsc::channel(1);
    // let scheduler_handle = actix_rt::spawn(utils::scheduler::run(
    //     client.clone(),
    //     environment.clone(),
    //     scheduler_exit_receiver,
    // ));

    // Run server.
    actix_web::HttpServer::new(move || {
        actix_web::App::new()
            .wrap(actix_cors::Cors::permissive())
            .wrap(actix_web::middleware::Logger::default())
            .app_data(actix_web::web::Data::new(reqwest_client.clone()))
            .app_data(actix_web::web::Data::new(mongodb_client.clone()))
            .app_data(actix_web::web::Data::new(rusoto_client.clone()))
            .app_data(actix_web::web::JsonConfig::default().error_handler(errors::handler::json))
            .app_data(actix_web::web::QueryConfig::default().error_handler(errors::handler::query))
            .service(
                actix_web::web::resource("/robots")
                    .route(actix_web::web::get().to(routes::robots::get))
                    .route(actix_web::web::post().to(routes::robots::post))
                    .route(actix_web::web::to(errors::handler::method_not_allowed)),
            )
            .default_service(actix_web::web::route().to(errors::handler::not_found))
    })
    .bind((server_config.address, server_config.port))?
    .run()
    .await?;

    // Stop scheduler.
    // scheduler_exit_sender.send(()).await.unwrap();
    // scheduler_handle.await.unwrap();

    return Ok(());
}
