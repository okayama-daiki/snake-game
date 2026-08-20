use crate::messages::{ClientMessage, Connect, Disconnect, WebsocketMessage};
use actix::{Actor, AsyncContext, Context, Handler, Recipient};
use game::coordinate::Coordinate;
use game::engine::GameEngine;
use std::collections::HashMap;
use std::time::Duration;
use uuid::Uuid;

const FPS: u64 = 30;
const FRAME_INTERVAL: Duration = Duration::from_millis(1000 / FPS);
const MAP_INTERVAL: Duration = Duration::from_millis(1000);
const MAX_WINDOW_SIZE: u16 = 8192;

#[derive(Debug, PartialEq)]
enum ClientCommand {
    Start,
    Accelerate,
    Velocity(Coordinate),
    WindowSize { width: u16, height: u16 },
}

#[derive(Default)]
struct WindowSize {
    pub width: u16,
    pub height: u16,
}

struct Session {
    pub addr: Recipient<WebsocketMessage>,
    pub is_playing: bool,
    pub additional_send_frame_count: u32, // after died, send additional frames
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
                if let Some(snake) = act.engine.get_snake(id) {
                    session.center_coordinate = snake.get_head().to_owned();
                    session.additional_send_frame_count = 150;
                }
                if act.engine.get_snake(id).is_none() && session.is_playing {
                    session.is_playing = false;
                }

                if session.additional_send_frame_count > 0 {
                    session.additional_send_frame_count -= 1;
                    session.addr.do_send(WebsocketMessage(
                        act.engine
                            .view(
                                id,
                                session.center_coordinate.x,
                                session.center_coordinate.y,
                                (session.window_size.width + 100).into(),
                                (session.window_size.height + 100).into(),
                            )
                            .to_bytes(),
                    ));
                }
            }
        });
        ctx.run_interval(MAP_INTERVAL, |act, _| {
            let mut map = act.engine.map(0.0, 0.0);
            for (_, session) in act.sessions.iter_mut() {
                if session.is_playing {
                    map.self_coordinate = GameEngine::map_coordinate(
                        session.center_coordinate.x,
                        session.center_coordinate.y,
                    );
                    session.addr.do_send(WebsocketMessage(map.to_bytes()));
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
                is_playing: false,
                additional_send_frame_count: 0,
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
        let Some(command) = parse_client_message(&msg.msg) else {
            return;
        };

        match command {
            ClientCommand::Start => {
                if self.engine.get_snake(id).is_none() {
                    self.engine.add_snake(*id);
                }
                if let Some(snake) = self.sessions.get_mut(id) {
                    snake.is_playing = true;
                }
            }
            ClientCommand::Accelerate => {
                if let Some(snake) = self.engine.get_snake_mut(id) {
                    snake.accelerate();
                }
            }
            ClientCommand::Velocity(velocity) => {
                self.engine.change_velocity(id, velocity);
            }
            ClientCommand::WindowSize { width, height } => {
                if let Some(session) = self.sessions.get_mut(id) {
                    session.window_size.height = height.clamp(1, MAX_WINDOW_SIZE);
                    session.window_size.width = width.clamp(1, MAX_WINDOW_SIZE);
                }
            }
        }
    }
}

fn parse_client_message(message: &str) -> Option<ClientCommand> {
    let mut parts = message.split_whitespace();
    let command = parts.next()?;

    let parsed = match command {
        "s" => ClientCommand::Start,
        "a" => ClientCommand::Accelerate,
        "v" => {
            let x = parts.next()?.parse::<f32>().ok()?;
            let y = parts.next()?.parse::<f32>().ok()?;
            if !x.is_finite() || !y.is_finite() {
                return None;
            }
            ClientCommand::Velocity(Coordinate { x, y })
        }
        "w" => ClientCommand::WindowSize {
            width: parts.next()?.parse::<u16>().ok()?,
            height: parts.next()?.parse::<u16>().ok()?,
        },
        _ => return None,
    };

    if parts.next().is_some() {
        return None;
    }

    Some(parsed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_valid_client_messages() {
        assert_eq!(parse_client_message("s"), Some(ClientCommand::Start));
        assert_eq!(
            parse_client_message("v 1 -0.5"),
            Some(ClientCommand::Velocity(Coordinate { x: 1.0, y: -0.5 }))
        );
        assert_eq!(
            parse_client_message("w 1920 1080"),
            Some(ClientCommand::WindowSize {
                width: 1920,
                height: 1080,
            })
        );
    }

    #[test]
    fn rejects_malformed_or_non_finite_client_messages() {
        assert_eq!(parse_client_message(""), None);
        assert_eq!(parse_client_message("v 1"), None);
        assert_eq!(parse_client_message("v NaN 1"), None);
        assert_eq!(parse_client_message("w large 1080"), None);
        assert_eq!(parse_client_message("s extra"), None);
    }
}
