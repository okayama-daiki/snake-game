use crate::messages::{ClientMessage, Connect, Disconnect, WebsocketMessage};
use actix::{Actor, Context, Handler, Recipient};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Default)]
pub struct WebsocketActor {
    sessions: HashMap<Uuid, Recipient<WebsocketMessage>>,
}

impl WebsocketActor {
    fn send_message(&self, message: &str, id_to: &Uuid) {
        if let Some(socket_recipient) = self.sessions.get(id_to) {
            socket_recipient.do_send(WebsocketMessage(message.to_owned()));
        }
    }
}

impl Actor for WebsocketActor {
    type Context = Context<Self>;
}

impl Handler<Connect> for WebsocketActor {
    type Result = ();

    fn handle(&mut self, msg: Connect, _: &mut Context<Self>) -> Self::Result {
        self.sessions.insert(msg.id, msg.addr);
    }
}

impl Handler<Disconnect> for WebsocketActor {
    type Result = ();

    fn handle(&mut self, msg: Disconnect, _: &mut Context<Self>) {
        let client_id = msg.id;
        self.sessions.remove(&client_id);
    }
}

impl Handler<ClientMessage> for WebsocketActor {
    type Result = ();

    fn handle(&mut self, msg: ClientMessage, _: &mut Context<Self>) {
        self.send_message(&msg.msg, &msg.id);
    }
}
