mod models;
mod routes;
mod utils;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();

    let bind_addr = std::env::var("BIND_ADDRESS").expect("BIND_ADDRESS must be defined");
    let bind_port = std::env::var("BIND_PORT")
        .expect("BIND_PORT must be defined.")
        .parse()
        .expect("BIND_PORT must be a valid port");

    let mongodb_username =
        std::env::var("MONGODB_USERNAME").expect("MONGODB_USERNAME must be defined");
    let mongodb_password =
        std::env::var("MONGODB_PASSWORD").expect("MONGODB_PASSWORD must be defined.");

    let client = utils::client::Client::new(&mongodb_username, &mongodb_password);

    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));

    utils::scheduler::robots(client.clone()).await;

    return actix_web::HttpServer::new(move || {
        actix_web::App::new()
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
    .bind((bind_addr, bind_port))?
    .run()
    .await;
}
