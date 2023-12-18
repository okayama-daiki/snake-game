mod game;
mod messages;
mod websocket_actor;
mod websocket_session;
use actix::{Actor, Addr};
use actix_web::{get, web::Data, web::Payload, App, Error, HttpRequest, HttpResponse, HttpServer};
use actix_web_actors::ws;
use std::env;
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

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenvy::dotenv().ok();

    let host = match env::var("HOST") {
        Ok(val) => val,
        Err(_e) => "0.0.0.0".to_string(),
    };
    let port = match env::var("PORT") {
        Ok(val) => val,
        Err(_e) => "8000".to_string(),
    };

    let websocket_server = WebsocketActor::default().start();

    HttpServer::new(move || {
        App::new()
            .service(handle_connection)
            .app_data(Data::new(websocket_server.clone()))
    })
    .bind(format!("{}:{}", host, port))?
    .run()
    .await
}
