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

struct Session {
    pub addr: Recipient<WebsocketMessage>,
    pub is_started: bool,
    pub window_size: WindowSize,
    pub center_coordinate: Coordinate,
}

pub struct WebsocketActor {
    sessions: HashMap<Uuid, Session>,
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
            for (id, session) in act.sessions.iter_mut() {
                if session.is_started {
                    if let Some(snake) = act.engine.get_snake(id) {
                        session.center_coordinate = snake.get_head().clone();
                    }
                    session.addr.do_send(WebsocketMessage(
                        to_string(&act.engine.view(
                            id,
                            session.center_coordinate.x,
                            session.center_coordinate.y,
                            session.window_size.width,
                            session.window_size.height,
                        ))
                        .unwrap(),
                    ));
                }
            }
        });
    }
}

impl Handler<Connect> for WebsocketActor {
    type Result = ();

    fn handle(&mut self, msg: Connect, _: &mut Context<Self>) -> Self::Result {
        self.sessions.insert(
            msg.id,
            Session {
                addr: msg.addr,
                is_started: false,
                window_size: WindowSize::default(),
                center_coordinate: Coordinate::default(),
            },
        );
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
            "s" => {
                if self.engine.get_snake(id).is_none() {
                    self.engine.add_snake(Coordinate::default(), *id);
                }
                self.sessions.get_mut(id).unwrap().is_started = true;
            }
            "v" => {
                let x = iter.next().unwrap().parse::<f64>().unwrap();
                let y = iter.next().unwrap().parse::<f64>().unwrap();
                self.engine.change_velocity(id, Coordinate { x, y });
            }
            "w" => {
                let width = iter.next().unwrap().parse::<f64>().unwrap();
                let height = iter.next().unwrap().parse::<f64>().unwrap();
                if let Some(session) = self.sessions.get_mut(id) {
                    session.window_size.height = height;
                    session.window_size.width = width;
                }
            }
            _ => {}
        }
    }
}
