use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::coordinate::Coordinate;
use crate::engine::{GameEngine, FIELD_SIZE};

pub const ACTION_COUNT: usize = 7;
pub const STATE_COUNT: usize = 216;
const DEFAULT_ACTION: usize = 3;
const AIM_OFFSETS: [f32; ACTION_COUNT] = [-70.0, -30.0, -10.0, 0.0, 10.0, 30.0, 70.0];
const DANGER_ANGLES: [f32; 3] = [-45.0, 0.0, 45.0];
const FORWARD_DANGER: usize = 1 << 1;
const BOT_PELLET_SEARCH_RADIUS: isize = 3;
const DIRECT_CAPTURE_DISTANCE: f32 = 250.0;
const TARGET_TURN_COST: f32 = 80.0;
const UNREACHABLE_TARGET_DISTANCE: f32 = 120.0;
const UNREACHABLE_TARGET_ANGLE: f32 = 60.0 * std::f32::consts::PI / 180.0;
const MIN_ATTACK_LENGTH: usize = 35;
const MIN_ATTACK_DISTANCE: f32 = 50.0;
const MAX_ATTACK_DISTANCE: f32 = 300.0;
const ATTACK_ANGLE: f32 = 30.0 * std::f32::consts::PI / 180.0;

#[derive(Clone, Copy, Debug)]
pub struct BotObservation {
    pub state: usize,
    pub pellet_distance: f32,
    pub target_heading: f32,
    pub target_id: Option<Uuid>,
    pub has_pellet: bool,
    pub danger_mask: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BotPolicy {
    q_values: Vec<Vec<f32>>,
}

impl Default for BotPolicy {
    fn default() -> Self {
        Self {
            q_values: vec![vec![0.0; ACTION_COUNT]; STATE_COUNT],
        }
    }
}

impl BotPolicy {
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        let policy: Self = serde_json::from_str(json)?;
        if policy.q_values.len() != STATE_COUNT
            || policy
                .q_values
                .iter()
                .any(|values| values.len() != ACTION_COUNT)
        {
            return Err(serde_json::Error::io(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "invalid Q-table dimensions",
            )));
        }
        Ok(policy)
    }

    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    pub fn action(&self, state: usize) -> usize {
        let Some(values) = self.q_values.get(state) else {
            return DEFAULT_ACTION;
        };
        let mut best_action = DEFAULT_ACTION;
        let mut best_value = values[best_action];
        for (action, &value) in values.iter().enumerate() {
            if value > best_value {
                best_action = action;
                best_value = value;
            }
        }
        best_action
    }

    pub fn action_for(&self, observation: BotObservation) -> usize {
        if observation.has_pellet
            && (observation.pellet_distance < DIRECT_CAPTURE_DISTANCE
                || observation.danger_mask & FORWARD_DANGER == 0)
        {
            DEFAULT_ACTION
        } else {
            self.action(observation.state)
        }
    }

    pub fn update(
        &mut self,
        state: usize,
        action: usize,
        reward: f32,
        next_state: Option<usize>,
        learning_rate: f32,
        discount: f32,
    ) {
        let next_value = next_state
            .and_then(|state| self.q_values.get(state))
            .map(|values| values.iter().copied().fold(f32::NEG_INFINITY, f32::max))
            .unwrap_or(0.0);
        if let Some(value) = self
            .q_values
            .get_mut(state)
            .and_then(|values| values.get_mut(action))
        {
            *value += learning_rate * (reward + discount * next_value - *value);
        }
    }
}

impl GameEngine {
    pub fn bot_attack_heading(&self, id: &Uuid) -> Option<f32> {
        let snake = self.snakes.get(id)?;
        if snake.bodies.len() < MIN_ATTACK_LENGTH {
            return None;
        }
        let head = *snake.get_head();
        let current_heading = heading(snake.velocity, snake.target_velocity);

        self.snakes
            .iter()
            .filter(|(other_id, _)| *other_id != id)
            .filter_map(|(_, other)| {
                let delta = torus_delta(&head, other.get_head());
                let distance = (delta.x * delta.x + delta.y * delta.y).sqrt();
                let target_heading = delta.y.atan2(delta.x);
                let relative_angle = normalize_angle(target_heading - current_heading);
                ((MIN_ATTACK_DISTANCE..=MAX_ATTACK_DISTANCE).contains(&distance)
                    && relative_angle.abs() <= ATTACK_ANGLE)
                    .then_some((target_heading, distance))
            })
            .min_by(|left, right| left.1.total_cmp(&right.1))
            .map(|(target_heading, _)| target_heading)
    }

    pub fn apply_bot_attack(&mut self, id: &Uuid, target_heading: f32) {
        let Some(snake) = self.snakes.get_mut(id) else {
            return;
        };
        snake.target_velocity = Coordinate {
            x: target_heading.cos(),
            y: target_heading.sin(),
        };
        if snake.acceleration_time_left == 0 {
            snake.accelerate();
        }
    }

    pub fn bot_observation(
        &self,
        id: &Uuid,
        preferred_target: Option<Uuid>,
    ) -> Option<BotObservation> {
        let snake = self.snakes.get(id)?;
        let head = *snake.get_head();
        let heading = heading(snake.velocity, snake.target_velocity);

        let nearby =
            Self::nearby_pellet_ids_with_radius(&self.pellet_grid, &head, BOT_PELLET_SEARCH_RADIUS);
        let candidate = |pellet_id: Uuid| {
            self.pellets.get(&pellet_id).map(|pellet| {
                let delta = torus_delta(&head, &pellet.position);
                let distance = (delta.x * delta.x + delta.y * delta.y).sqrt();
                let target_heading = delta.y.atan2(delta.x);
                let relative_angle = normalize_angle(target_heading - heading);
                let cost = target_cost(distance, relative_angle);
                (pellet_id, delta, distance, cost, relative_angle)
            })
        };
        let preferred = preferred_target.and_then(candidate);
        let released_target = preferred
            .filter(|value| !target_is_reachable(value.2, value.4))
            .map(|value| value.0);
        let nearest = preferred
            .filter(|value| target_is_reachable(value.2, value.4))
            .or_else(|| {
                nearby
                    .iter()
                    .filter(|pellet_id| Some(**pellet_id) != released_target)
                    .filter_map(|pellet_id| candidate(*pellet_id))
                    .filter(|value| target_is_reachable(value.2, value.4))
                    .min_by(|left, right| left.3.total_cmp(&right.3))
            });

        let has_pellet = nearest.is_some();
        let (pellet_bucket, pellet_distance, target_heading, target_id) = nearest
            .map(|(target_id, delta, distance, _, _)| {
                let target_angle = delta.y.atan2(delta.x);
                let relative = normalize_angle(target_angle - heading).to_degrees();
                let bucket = if relative < -100.0 {
                    0
                } else if relative < -45.0 {
                    1
                } else if relative < -15.0 {
                    2
                } else if relative < -5.0 {
                    3
                } else if relative <= 5.0 {
                    4
                } else if relative <= 15.0 {
                    5
                } else if relative <= 45.0 {
                    6
                } else if relative <= 100.0 {
                    7
                } else {
                    8
                };
                (bucket, distance, target_angle, Some(target_id))
            })
            .unwrap_or((4, 300.0, heading, None));

        let mut danger_mask = 0;
        for (index, degrees) in DANGER_ANGLES.iter().enumerate() {
            let angle = heading + degrees.to_radians();
            let lookahead = Coordinate {
                x: (head.x + angle.cos() * 120.0).rem_euclid(FIELD_SIZE),
                y: (head.y + angle.sin() * 120.0).rem_euclid(FIELD_SIZE),
            };
            let danger = self.snakes.iter().any(|(other_id, other)| {
                other.bodies.iter().enumerate().any(|(body_index, body)| {
                    if other_id == id && body_index < 10 {
                        return false;
                    }
                    let clearance = (snake.size + other.size) as f32 + 8.0;
                    lookahead.distance2(body) <= clearance * clearance
                })
            });
            if danger {
                danger_mask |= 1 << index;
            }
        }

        let distance_bucket = if pellet_distance < 75.0 {
            0
        } else if pellet_distance < 180.0 {
            1
        } else {
            2
        };
        let state = (pellet_bucket * 8 + danger_mask) * 3 + distance_bucket;
        Some(BotObservation {
            state,
            pellet_distance,
            target_heading,
            target_id,
            has_pellet,
            danger_mask,
        })
    }

    pub fn apply_bot_action(&mut self, id: &Uuid, observation: BotObservation, action: usize) {
        let Some(snake) = self.snakes.get_mut(id) else {
            return;
        };
        let offset = AIM_OFFSETS[action.min(ACTION_COUNT - 1)].to_radians();
        let target = observation.target_heading + offset;
        snake.target_velocity = Coordinate {
            x: target.cos(),
            y: target.sin(),
        };
    }
}

fn heading(velocity: Coordinate, target: Coordinate) -> f32 {
    let velocity_norm = velocity.x * velocity.x + velocity.y * velocity.y;
    if velocity_norm > f32::EPSILON {
        velocity.y.atan2(velocity.x)
    } else {
        let target_norm = target.x * target.x + target.y * target.y;
        if target_norm > f32::EPSILON {
            target.y.atan2(target.x)
        } else {
            0.0
        }
    }
}

fn torus_delta(origin: &Coordinate, target: &Coordinate) -> Coordinate {
    Coordinate {
        x: signed_axis_delta(target.x - origin.x),
        y: signed_axis_delta(target.y - origin.y),
    }
}

fn signed_axis_delta(delta: f32) -> f32 {
    (delta + FIELD_SIZE / 2.0).rem_euclid(FIELD_SIZE) - FIELD_SIZE / 2.0
}

fn normalize_angle(angle: f32) -> f32 {
    (angle + std::f32::consts::PI).rem_euclid(std::f32::consts::TAU) - std::f32::consts::PI
}

fn target_cost(distance: f32, relative_angle: f32) -> f32 {
    distance + relative_angle.abs() * TARGET_TURN_COST
}

fn target_is_reachable(distance: f32, relative_angle: f32) -> bool {
    distance >= UNREACHABLE_TARGET_DISTANCE || relative_angle.abs() <= UNREACHABLE_TARGET_ANGLE
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero_policy_falls_back_to_straight() {
        assert_eq!(BotPolicy::default().action(0), DEFAULT_ACTION);
    }

    #[test]
    fn q_learning_moves_a_value_towards_its_target() {
        let mut policy = BotPolicy::default();
        policy.update(0, 4, 10.0, None, 0.5, 0.9);

        assert_eq!(policy.action(0), 4);
    }

    #[test]
    fn safe_path_aims_directly_at_the_pellet() {
        let policy = BotPolicy::default();
        let observation = BotObservation {
            state: 0,
            pellet_distance: 100.0,
            target_heading: 0.0,
            target_id: None,
            has_pellet: true,
            danger_mask: 0,
        };

        assert_eq!(policy.action_for(observation), DEFAULT_ACTION);
    }

    #[test]
    fn nearby_pellets_are_collected_instead_of_orbited() {
        let mut policy = BotPolicy::default();
        policy.update(0, 0, 10.0, None, 1.0, 0.0);
        let observation = BotObservation {
            state: 0,
            pellet_distance: DIRECT_CAPTURE_DISTANCE - 1.0,
            target_heading: 0.0,
            target_id: None,
            has_pellet: true,
            danger_mask: FORWARD_DANGER,
        };

        assert_eq!(policy.action_for(observation), DEFAULT_ACTION);
    }

    #[test]
    fn observations_stay_inside_the_q_table() {
        let id = Uuid::new_v4();
        let mut engine = GameEngine::new();
        engine.add_snake_at(id, Coordinate { x: 100.0, y: 100.0 });
        engine.forward();

        let observation = engine.bot_observation(&id, None).unwrap();
        assert!(observation.state < STATE_COUNT);
    }

    #[test]
    fn bot_actions_do_not_consume_length_by_accelerating() {
        let id = Uuid::new_v4();
        let mut engine = GameEngine::new();
        engine.add_snake_at(id, Coordinate { x: 100.0, y: 100.0 });
        let snake = engine.get_snake_mut(&id).unwrap();
        while snake.bodies.len() < 20 {
            snake.bodies.push_back(Coordinate::default());
        }

        let observation = engine.bot_observation(&id, None).unwrap();
        engine.apply_bot_action(&id, observation, ACTION_COUNT - 1);

        assert_eq!(engine.get_snake(&id).unwrap().acceleration_time_left, 0);
    }

    #[test]
    fn direct_action_aims_at_the_selected_pellet() {
        let id = Uuid::new_v4();
        let mut engine = GameEngine::new();
        engine.add_snake_at(id, Coordinate { x: 100.0, y: 100.0 });
        engine.forward();
        let observation = engine.bot_observation(&id, None).unwrap();

        engine.apply_bot_action(&id, observation, DEFAULT_ACTION);

        let target = engine.get_snake(&id).unwrap().target_velocity;
        let target_heading = target.y.atan2(target.x);
        assert!(normalize_angle(target_heading - observation.target_heading).abs() < 1e-5);
    }

    #[test]
    fn target_selection_prefers_ahead_over_slightly_nearer_behind() {
        assert!(
            target_cost(60.0, 0.0) < target_cost(40.0, std::f32::consts::PI),
            "a bot should finish moving forward instead of alternating between nearby targets"
        );
    }

    #[test]
    fn preferred_target_is_kept_until_the_pellet_disappears() {
        let id = Uuid::new_v4();
        let mut engine = GameEngine::new();
        engine.add_snake_at(id, Coordinate { x: 100.0, y: 100.0 });
        engine.forward();
        let first = engine.bot_observation(&id, None).unwrap();

        let second = engine.bot_observation(&id, first.target_id).unwrap();

        assert!(first.target_id.is_some());
        assert_eq!(second.target_id, first.target_id);
    }

    #[test]
    fn close_target_behind_the_turning_circle_is_released() {
        assert!(!target_is_reachable(
            UNREACHABLE_TARGET_DISTANCE - 1.0,
            UNREACHABLE_TARGET_ANGLE + 0.01,
        ));
        assert!(target_is_reachable(
            UNREACHABLE_TARGET_DISTANCE + 1.0,
            std::f32::consts::PI,
        ));
    }

    #[test]
    fn bot_dashes_toward_an_opponent_in_front() {
        let attacker = Uuid::new_v4();
        let opponent = Uuid::new_v4();
        let mut engine = GameEngine::new();
        engine.add_snake_at(attacker, Coordinate { x: 100.0, y: 100.0 });
        engine.add_snake_at(opponent, Coordinate { x: 250.0, y: 100.0 });
        let snake = engine.get_snake_mut(&attacker).unwrap();
        snake.velocity = Coordinate { x: 1.0, y: 0.0 };
        while snake.bodies.len() < MIN_ATTACK_LENGTH {
            snake.bodies.push_back(Coordinate::default());
        }

        let attack_heading = engine.bot_attack_heading(&attacker).unwrap();
        engine.apply_bot_attack(&attacker, attack_heading);

        let snake = engine.get_snake(&attacker).unwrap();
        assert!(attack_heading.abs() < 1e-6);
        assert_eq!(snake.acceleration_time_left, 60);
    }

    #[test]
    fn active_attack_dash_is_not_restarted_every_frame() {
        let attacker = Uuid::new_v4();
        let mut engine = GameEngine::new();
        engine.add_snake_at(attacker, Coordinate { x: 100.0, y: 100.0 });
        let snake = engine.get_snake_mut(&attacker).unwrap();
        while snake.bodies.len() < MIN_ATTACK_LENGTH {
            snake.bodies.push_back(Coordinate::default());
        }
        snake.acceleration_time_left = 42;

        engine.apply_bot_attack(&attacker, 0.0);

        assert_eq!(
            engine.get_snake(&attacker).unwrap().acceleration_time_left,
            42
        );
    }

    #[test]
    fn attack_dash_defeats_a_non_accelerating_opponent() {
        let attacker = Uuid::new_v4();
        let opponent = Uuid::new_v4();
        let mut engine = GameEngine::new();
        engine.add_snake_at(attacker, Coordinate { x: 100.0, y: 100.0 });
        engine.add_snake_at(opponent, Coordinate { x: 160.0, y: 100.0 });
        let snake = engine.get_snake_mut(&attacker).unwrap();
        snake.velocity = Coordinate { x: 1.0, y: 0.0 };
        while snake.bodies.len() < MIN_ATTACK_LENGTH {
            snake.bodies.push_back(Coordinate::default());
        }

        let attack_heading = engine.bot_attack_heading(&attacker).unwrap();
        engine.apply_bot_attack(&attacker, attack_heading);
        let mut deaths = Vec::new();
        for _ in 0..5 {
            deaths.extend(engine.forward().deaths);
            if !deaths.is_empty() {
                break;
            }
        }

        assert!(deaths.iter().any(|death| death.id == opponent));
        assert!(!deaths.iter().any(|death| death.id == attacker));
    }
}
