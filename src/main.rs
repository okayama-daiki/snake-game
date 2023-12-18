mod game;
mod messages;
mod websocket_actor;
mod websocket_session;
use actix::{Actor, Addr};
use actix_web::{get, web::Data, web::Payload, App, Error, HttpRequest, HttpResponse, HttpServer};
use actix_web_actors::ws;
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
    let websocket_server = WebsocketActor::default().start();

    HttpServer::new(move || {
        App::new()
            .service(handle_connection)
            .app_data(Data::new(websocket_server.clone()))
    })
    .bind("127.0.0.1:8000")?
    .run()
    .await
}
