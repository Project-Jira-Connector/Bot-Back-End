mod models;
mod routes;
mod utils;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));

    let environment = models::config::Environment::new();
    let client = utils::client::Client::new(
        &environment.database.username,
        &environment.database.password,
    );

    utils::scheduler::robots(client.clone(), environment.schedule.clone()).await;
    utils::scheduler::purger(
        client.clone(),
        environment.schedule.clone(),
        environment.notification.email,
        environment.notification.password,
    )
    .await;

    return actix_web::HttpServer::new(move || {
        actix_web::App::new()
            .wrap(actix_cors::Cors::permissive())
            .app_data(actix_web::web::Data::new(client.clone()))
            .wrap(actix_web::middleware::Logger::default())
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
    .await;
}
