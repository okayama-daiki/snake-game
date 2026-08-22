use game::bot::{BotPolicy, ACTION_COUNT};
use game::coordinate::Coordinate;
use game::engine::GameEngine;
use rand::RngExt;
use std::collections::HashMap;
use std::env;
use std::fs;
use uuid::Uuid;

const BOT_COUNT: usize = 16;
const TRAINING_STEPS: usize = 120_000;
const LEARNING_RATE: f32 = 0.1;
const DISCOUNT: f32 = 0.97;
const SUCCESS_LENGTH: usize = 160;

fn main() {
    let output = env::args()
        .nth(1)
        .unwrap_or_else(|| "bot_policy.json".to_string());
    let mut engine = GameEngine::new();
    let bots: Vec<_> = (0..BOT_COUNT).map(|_| Uuid::new_v4()).collect();
    for id in &bots {
        spawn(&mut engine, *id);
    }
    let mut targets: HashMap<Uuid, Option<Uuid>> = bots.iter().map(|id| (*id, None)).collect();

    let mut policy = BotPolicy::default();
    for step in 0..TRAINING_STEPS {
        let exploration = (0.9 * (1.0 - step as f32 / TRAINING_STEPS as f32)).max(0.03);
        let mut transitions = HashMap::new();
        for id in &bots {
            let preferred_target = targets.get(id).copied().flatten();
            let Some(observation) = engine.bot_observation(id, preferred_target) else {
                continue;
            };
            targets.insert(*id, observation.target_id);
            let action = if rand::rng().random_range(0.0..1.0) < exploration {
                rand::rng().random_range(0..ACTION_COUNT)
            } else {
                policy.action_for(observation)
            };
            engine.apply_bot_action(id, observation, action);
            transitions.insert(*id, (observation, action));
        }

        let events = engine.forward();
        for id in &bots {
            let Some((previous, action)) = transitions.get(id) else {
                continue;
            };
            let eaten = events.pellets_eaten.get(id).copied().unwrap_or(0) as f32;
            let died = events.deaths.iter().any(|death| death.id == *id);
            let preferred_target = targets.get(id).copied().flatten();
            let next = engine.bot_observation(id, preferred_target);
            if let Some(observation) = next {
                targets.insert(*id, observation.target_id);
            }
            let succeeded = engine
                .score(id)
                .is_some_and(|score| score >= SUCCESS_LENGTH);
            let progress = next
                .map(|observation| (previous.pellet_distance - observation.pellet_distance) / 5.0)
                .unwrap_or(0.0)
                .clamp(-1.0, 1.0);
            let reward = eaten * 10.0 + progress * 0.35 - 0.005 - if died { 20.0 } else { 0.0 }
                + if succeeded { 20.0 } else { 0.0 };
            policy.update(
                previous.state,
                *action,
                reward,
                if died || succeeded {
                    None
                } else {
                    next.map(|observation| observation.state)
                },
                LEARNING_RATE,
                DISCOUNT,
            );
            if died || succeeded {
                spawn(&mut engine, *id);
                targets.insert(*id, None);
            }
        }

        if step % 10_000 == 0 {
            eprintln!("training step {step}/{TRAINING_STEPS}");
        }
    }

    fs::write(&output, policy.to_json().expect("policy should serialize"))
        .expect("policy should be written");
    eprintln!("saved policy to {output}");
}

fn spawn(engine: &mut GameEngine, id: Uuid) {
    engine.add_snake_at(
        id,
        Coordinate {
            x: rand::rng().random_range(4_200.0..5_800.0),
            y: rand::rng().random_range(4_200.0..5_800.0),
        },
    );
}
