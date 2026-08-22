use crate::messages::{ClientMessage, Connect, Disconnect, WebsocketMessage};
use crate::ranking::SharedRanking;
use actix::{Actor, AsyncContext, Context, Handler, Recipient};
use game::bot::BotPolicy;
use game::coordinate::Coordinate;
use game::engine::GameEngine;
use std::collections::HashMap;
use std::env;
use std::time::Duration;
use uuid::Uuid;

const FPS: u64 = 30;
const FRAME_INTERVAL: Duration = Duration::from_millis(1000 / FPS);
const MAP_INTERVAL: Duration = Duration::from_millis(1000);
const MAX_WINDOW_SIZE: u16 = 8192;
const DEFAULT_BOT_COUNT: usize = 6;
const MAX_BOT_COUNT: usize = 32;
const BOT_POLICY: &str = include_str!("../assets/bot_policy.json");

#[derive(Debug, PartialEq)]
enum ClientCommand {
    Start(Uuid),
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
    pub name: String,
    pub player_token: Option<Uuid>,
}

struct BotPlayer {
    id: Uuid,
    name: String,
    target_id: Option<Uuid>,
}

pub struct WebsocketActor {
    sessions: HashMap<Uuid, Session>,
    engine: GameEngine,
    ranking: SharedRanking,
    bots: Vec<BotPlayer>,
    bot_policy: BotPolicy,
}

impl WebsocketActor {
    pub fn new(ranking: SharedRanking) -> Self {
        let bot_count = env::var("BOT_COUNT")
            .ok()
            .and_then(|value| value.parse().ok())
            .unwrap_or(DEFAULT_BOT_COUNT)
            .min(MAX_BOT_COUNT);
        let mut engine = GameEngine::new();
        let bots: Vec<_> = (1..=bot_count)
            .map(|number| BotPlayer {
                id: Uuid::new_v4(),
                name: format!("RL Bot {number}"),
                target_id: None,
            })
            .collect();
        for bot in &bots {
            engine.add_snake(bot.id);
        }

        WebsocketActor {
            sessions: HashMap::new(),
            engine,
            ranking,
            bots,
            bot_policy: BotPolicy::from_json(BOT_POLICY)
                .expect("embedded Bot policy must be valid"),
        }
    }
}

impl Actor for WebsocketActor {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        ctx.run_interval(FRAME_INTERVAL, |act, _| {
            for bot in &mut act.bots {
                if let Some(target_heading) = act.engine.bot_attack_heading(&bot.id) {
                    bot.target_id = None;
                    act.engine.apply_bot_attack(&bot.id, target_heading);
                    continue;
                }
                if let Some(observation) = act.engine.bot_observation(&bot.id, bot.target_id) {
                    bot.target_id = observation.target_id;
                    let action = act.bot_policy.action_for(observation);
                    act.engine.apply_bot_action(&bot.id, observation, action);
                }
            }

            let events = act.engine.forward();
            for death in events.deaths {
                if let Some(bot) = act.bots.iter_mut().find(|bot| bot.id == death.id) {
                    bot.target_id = None;
                    if let Ok(mut ranking) = act.ranking.write() {
                        ranking.remove(&bot.id);
                    }
                    act.engine.add_snake(bot.id);
                } else if act.sessions.contains_key(&death.id) {
                    if let Ok(mut ranking) = act.ranking.write() {
                        ranking.remove(&death.id);
                    }
                }
            }

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
            for (id, session) in act.sessions.iter_mut() {
                if session.is_playing {
                    if let Some(score) = act.engine.score(id) {
                        if let Ok(mut ranking) = act.ranking.write() {
                            ranking.update(*id, &session.name, score, false, session.player_token);
                        }
                    }
                    map.self_coordinate = GameEngine::map_coordinate(
                        session.center_coordinate.x,
                        session.center_coordinate.y,
                    );
                    session.addr.do_send(WebsocketMessage(map.to_bytes()));
                }
            }
            for bot in &act.bots {
                if let Some(score) = act.engine.score(&bot.id) {
                    if let Ok(mut ranking) = act.ranking.write() {
                        ranking.update(bot.id, &bot.name, score, true, None);
                    }
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
                name: format!("Player-{}", &msg.id.simple().to_string()[..4]),
                player_token: None,
            },
        );
    }
}

impl Handler<Disconnect> for WebsocketActor {
    type Result = ();

    fn handle(&mut self, msg: Disconnect, _: &mut Context<Self>) {
        let client_id = msg.id;
        if let Ok(mut ranking) = self.ranking.write() {
            ranking.remove(&client_id);
        }
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
            ClientCommand::Start(player_token) => {
                if self.engine.get_snake(id).is_none() {
                    self.engine.add_snake(*id);
                }
                if let Some(session) = self.sessions.get_mut(id) {
                    session.is_playing = true;
                    session.player_token = Some(player_token);
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
        "s" => ClientCommand::Start(parts.next()?.parse::<Uuid>().ok()?),
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
        let player_token = Uuid::new_v4();
        assert_eq!(
            parse_client_message(&format!("s {player_token}")),
            Some(ClientCommand::Start(player_token))
        );
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
        assert_eq!(parse_client_message("s"), None);
        assert_eq!(parse_client_message("s Alice"), None);
        assert_eq!(parse_client_message("status"), None);
    }

    #[test]
    fn embedded_bot_policy_has_valid_dimensions() {
        assert!(BotPolicy::from_json(BOT_POLICY).is_ok());
    }
}
