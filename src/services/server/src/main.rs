mod messages;
mod ranking;
mod websocket_actor;
mod websocket_session;
use actix::{Actor, Addr};
use actix_web::{
    get,
    web::{Data, Payload, Query},
    App, Error, HttpRequest, HttpResponse, HttpServer,
};
use actix_web_actors::ws;
use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};
use ranking::{RankingStore, SharedRanking};
use serde::Deserialize;
use std::env;
use std::sync::{Arc, RwLock};
use websocket_actor::WebsocketActor;
use websocket_session::WebsocketSession;

#[get("/")]
pub async fn handle_connection(
    req: HttpRequest,
    stream: Payload,
    srv: Data<Addr<WebsocketActor>>,
) -> Result<HttpResponse, Error> {
    let session = WebsocketSession::new(srv.get_ref().clone());
    let response = ws::start(session, &req, stream)?;
    Ok(response)
}

#[get("/health")]
pub async fn health() -> HttpResponse {
    HttpResponse::Ok().finish()
}

#[get("/leaderboard")]
pub async fn leaderboard(
    ranking: Data<SharedRanking>,
    query: Query<LeaderboardQuery>,
) -> HttpResponse {
    let entries = ranking
        .read()
        .map(|ranking| {
            let player_token = query
                .player
                .as_deref()
                .and_then(|value| uuid::Uuid::parse_str(value).ok());
            ranking.leaderboard(player_token)
        })
        .unwrap_or_default();

    HttpResponse::Ok()
        .insert_header(("Access-Control-Allow-Origin", "*"))
        .json(entries)
}

#[derive(Deserialize)]
pub struct LeaderboardQuery {
    player: Option<String>,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenvy::dotenv().ok();

    let host = match env::var("HOST") {
        Ok(val) => val,
        Err(_e) => "0.0.0.0".to_string(),
    };
    let port = match env::var("PORT") {
        Ok(val) => val,
        Err(_e) => "5173".to_string(),
    };

    let ranking = Arc::new(RwLock::new(RankingStore::default()));
    let websocket_server = WebsocketActor::new(ranking.clone()).start();

    println!("Starting server on {}:{}", host, port);
    if std::env::var("PRIVATE_KEY_FILE").is_err()
        || std::env::var("CERTIFICATE_CHAIN_FILE").is_err()
    {
        HttpServer::new(move || {
            App::new()
                .service(handle_connection)
                .service(health)
                .service(leaderboard)
                .app_data(Data::new(ranking.clone()))
                .app_data(Data::new(websocket_server.clone()))
        })
        .bind(format!("{}:{}", host, port))?
        .run()
        .await
    } else {
        let private_key = std::env::var("PRIVATE_KEY_FILE").unwrap();
        let certificate_chain = std::env::var("CERTIFICATE_CHAIN_FILE").unwrap();

        let mut builder = SslAcceptor::mozilla_intermediate(SslMethod::tls()).unwrap();
        builder
            .set_private_key_file(private_key, SslFiletype::PEM)
            .unwrap();
        builder
            .set_certificate_chain_file(certificate_chain)
            .unwrap();

        HttpServer::new(move || {
            App::new()
                .service(handle_connection)
                .service(health)
                .service(leaderboard)
                .app_data(Data::new(ranking.clone()))
                .app_data(Data::new(websocket_server.clone()))
        })
        .bind_openssl(format!("{}:{}", host, port), builder)?
        .run()
        .await
    }
}
