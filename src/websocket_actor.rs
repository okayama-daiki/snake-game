use crate::game::coordinate::Coordinate;
use crate::game::engine::GameEngine;
use crate::messages::{ClientMessage, Connect, Disconnect, WebsocketMessage};
use actix::{Actor, AsyncContext, Context, Handler, Recipient};
use serde_json::to_string;
use std::collections::HashMap;
use std::time::Duration;
use uuid::Uuid;

const FPS: u64 = 30;
const FRAME_INTERVAL: Duration = Duration::from_millis(1000 / FPS);

#[derive(Default)]
struct WindowSize {
    pub width: f64,
    pub height: f64,
}

pub struct WebsocketActor {
    sessions: HashMap<Uuid, (Recipient<WebsocketMessage>, WindowSize, Coordinate)>,
    engine: GameEngine,
}

impl Default for WebsocketActor {
    fn default() -> Self {
        WebsocketActor {
            sessions: HashMap::new(),
            engine: GameEngine::new(),
        }
    }
}

impl Actor for WebsocketActor {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        ctx.run_interval(FRAME_INTERVAL, |act, _| {
            act.engine.forward();
            for (id, (client, window, center)) in act.sessions.iter_mut() {
                if let Some(snake) = act.engine.get_snake(id) {
                    *center = snake.get_head().clone();
                }
                client.do_send(WebsocketMessage(
                    to_string(&act.engine.view(
                        id,
                        center.x,
                        center.y,
                        window.width,
                        window.height,
                    ))
                    .unwrap(),
                ));
            }
        });
    }
}

impl Handler<Connect> for WebsocketActor {
    type Result = ();

    fn handle(&mut self, msg: Connect, _: &mut Context<Self>) -> Self::Result {
        self.sessions.insert(
            msg.id,
            (msg.addr, WindowSize::default(), Coordinate::default()),
        );
        self.engine.add_snake(Coordinate { x: 0., y: 0. }, msg.id);
    }
}

impl Handler<Disconnect> for WebsocketActor {
    type Result = ();

    fn handle(&mut self, msg: Disconnect, _: &mut Context<Self>) {
        let client_id = msg.id;
        self.sessions.remove(&client_id);
        self.engine.remove_snake(&client_id);
    }
}

impl Handler<ClientMessage> for WebsocketActor {
    type Result = ();

    fn handle(&mut self, msg: ClientMessage, _: &mut Context<Self>) {
        let id = &msg.id;
        let mut iter = msg.msg.split(' ');

        let query = iter.next().unwrap();

        match query {
            "v" => {
                let x = iter.next().unwrap().parse::<f64>().unwrap();
                let y = iter.next().unwrap().parse::<f64>().unwrap();
                self.engine.change_velocity(id, Coordinate { x, y });
            }
            "w" => {
                let width = iter.next().unwrap().parse::<f64>().unwrap();
                let height = iter.next().unwrap().parse::<f64>().unwrap();
                if let Some((_, window, _)) = self.sessions.get_mut(id) {
                    window.height = height;
                    window.width = width;
                }
            }
            _ => {}
        }
    }
}
