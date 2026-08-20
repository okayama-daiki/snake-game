use rand::Rng;
use std::collections::{HashMap, HashSet, VecDeque};
// TODO: Consider using BuildHasher trait
use std::cmp::Ordering;
use uuid::Uuid;

use super::coordinate::Coordinate;
use super::map::Map;
use super::pellet::Pellet;
use super::snake::Snake;
use super::view::View;

static FIELD_SIZE: f32 = 10000.0;
const MAX_PELLET_COUNT: usize = 5_000;
const MAP_SIZE: usize = 100;

#[derive(Default)]
pub struct GameEngine {
    frame_count: u32,
    snakes: HashMap<Uuid, Snake>,
    pellets: HashMap<Uuid, Pellet>,
}

impl GameEngine {
    pub fn new() -> GameEngine {
        GameEngine {
            frame_count: 0,
            snakes: HashMap::new(),
            pellets: HashMap::new(),
        }
    }

    pub fn get_random_coordinate(&self) -> Coordinate {
        let rx = rand::thread_rng().gen_range((0.)..1.);
        let ry = rand::thread_rng().gen_range((0.)..1.);
        let x = FIELD_SIZE * rx;
        let y = FIELD_SIZE * ry;
        Coordinate { x, y }
    }

    pub fn get_snake(&self, id: &Uuid) -> Option<&Snake> {
        self.snakes.get(id)
    }

    pub fn get_snake_mut(&mut self, id: &Uuid) -> Option<&mut Snake> {
        self.snakes.get_mut(id)
    }

    pub fn add_snake(&mut self, id: Uuid) {
        let snake: Snake = Snake::new(self.get_random_coordinate(), 5.0);
        self.snakes.insert(id, snake);
    }

    pub fn remove_snake(&mut self, id: &Uuid) {
        if let Some(snake) = self.snakes.get(id) {
            for body in snake.bodies.iter() {
                if rand::thread_rng().gen_range(0..10) < 5 {
                    let dx = rand::thread_rng().gen_range((-10.)..10.);
                    let dy = rand::thread_rng().gen_range((-10.)..10.);
                    let pellet = Pellet::new_with_color_and_size(
                        Coordinate {
                            x: body.x + dx,
                            y: body.y + dy,
                        },
                        snake.color.clone(),
                        3,
                    );
                    let id = Uuid::new_v4();
                    self.pellets.insert(id, pellet);
                }
            }
        }
        self.snakes.remove(id);
    }

    fn fill_pellet(&mut self) {
        while self.pellets.len() < MAX_PELLET_COUNT {
            let new_pellet = Pellet::new(self.get_random_coordinate());
            let id = Uuid::new_v4();
            self.pellets.insert(id, new_pellet);
        }
    }

    pub fn forward(&mut self) {
        //! Forward one frame of the game.

        let mut touched_pellets: HashSet<Uuid> = HashSet::new();

        // Update snakes
        for (_, snake) in self.snakes.iter_mut() {
            let mut accelerate_factor = 1.;

            if snake.acceleration_time_left > 0 {
                snake.acceleration_time_left -= 1;
                accelerate_factor = 2.;
            }

            let head = snake.get_head();
            let new_head = Coordinate {
                x: head.x + snake.velocity.x * snake.speed * accelerate_factor,
                y: head.y + snake.velocity.y * snake.speed * accelerate_factor,
            };
            let new_head = Coordinate {
                x: new_head.x.rem_euclid(FIELD_SIZE),
                y: new_head.y.rem_euclid(FIELD_SIZE),
            };

            if snake.acceleration_time_left > 0 && snake.frame_count_offset % 6 == 0 {
                self.pellets.insert(
                    Uuid::new_v4(),
                    Pellet::new_with_color_and_size(
                        snake.bodies.pop_back().unwrap(),
                        snake.color.clone(),
                        3,
                    ),
                );
            }
            snake.bodies.pop_back();
            snake.bodies.push_front(new_head);

            let mut eaten_pellets: Vec<Uuid> = Vec::new();

            for (id, pellet) in self.pellets.iter_mut() {
                // Draw pellets towards the snake
                if pellet.position.distance2(&new_head) < ((snake.size * 2).pow(2) as f32) {
                    let nx = pellet.position.x + (new_head.x - pellet.position.x) / 5.;
                    let ny = pellet.position.y + (new_head.y - pellet.position.y) / 5.;
                    pellet.position = Coordinate { x: nx, y: ny };
                    touched_pellets.insert(*id);
                }

                // Eat pellets
                if pellet.position.distance2(&new_head) < (snake.size.pow(2) as f32) {
                    snake.bodies.push_back(snake.get_tail().to_owned());
                    eaten_pellets.push(*id);
                }
            }

            for id in eaten_pellets.iter() {
                self.pellets.remove(id);
            }

            snake.size = (15 + snake.bodies.len() / 50).min(40);
        }

        // Detect collision
        let mut dead_snakes: HashSet<Uuid> = HashSet::new();

        let snake_ids: Vec<Uuid> = self.snakes.keys().copied().collect();
        for (index, id1) in snake_ids.iter().enumerate() {
            for id2 in snake_ids.iter().skip(index + 1) {
                let snake1 = self.snakes.get(id1).unwrap();
                let snake2 = self.snakes.get(id2).unwrap();
                let head1 = snake1.get_head();
                let head2 = snake2.get_head();

                // the head to head collision, rules:
                // 1. the acceleration snake wins
                // 2. the bigger snake wins
                // 3. random
                if head1.distance2(head2) <= ((snake1.size + snake2.size).pow(2) as f32) {
                    if snake1.acceleration_time_left > 0 && snake2.acceleration_time_left > 0
                        || snake1.acceleration_time_left == 0 && snake2.acceleration_time_left == 0
                    {
                        match snake1.size.cmp(&snake2.size) {
                            Ordering::Greater => {
                                dead_snakes.insert(*id2);
                            }
                            Ordering::Less => {
                                dead_snakes.insert(*id1);
                            }
                            Ordering::Equal => {
                                if rand::thread_rng().gen_range(0..10) < 5 {
                                    dead_snakes.insert(*id1);
                                } else {
                                    dead_snakes.insert(*id2);
                                }
                            }
                        }
                    } else if snake2.acceleration_time_left > 0 {
                        dead_snakes.insert(*id1);
                    } else {
                        dead_snakes.insert(*id2);
                    }
                    continue;
                }

                if snake2.bodies.iter().any(|body| {
                    head1.distance2(body) <= ((snake1.size + snake2.size).pow(2) as f32)
                }) {
                    dead_snakes.insert(*id1);
                }
                if snake1.bodies.iter().any(|body| {
                    head2.distance2(body) <= ((snake1.size + snake2.size).pow(2) as f32)
                }) {
                    dead_snakes.insert(*id2);
                }
            }
        }

        for id in dead_snakes.iter() {
            self.remove_snake(id)
        }

        // Refill pellets
        self.fill_pellet();

        // Update time to live
        for (id, pellet) in self.pellets.iter_mut() {
            if !touched_pellets.contains(id) {
                pellet.update();
            }
            pellet.frame_count_offset += 1;
        }
        for (_, snake) in self.snakes.iter_mut() {
            snake.frame_count_offset += 1;
        }
        self.frame_count += 1;
    }

    pub fn change_velocity(&mut self, id: &Uuid, velocity: Coordinate) {
        if !velocity.x.is_finite() || !velocity.y.is_finite() {
            return;
        }

        if let Some(snake) = self.snakes.get_mut(id) {
            let weight = 0.2;
            let new_velocity = Coordinate {
                x: (1. - weight) * snake.velocity.x + weight * velocity.x,
                y: (1. - weight) * snake.velocity.y + weight * velocity.y,
            };
            let norm = (new_velocity.x.powi(2) + new_velocity.y.powi(2)).sqrt();
            if norm <= f32::EPSILON || !norm.is_finite() {
                return;
            }
            let new_velocity = Coordinate {
                x: new_velocity.x / norm,
                y: new_velocity.y / norm,
            };
            snake.velocity = new_velocity;
        }
    }

    pub fn map(&self, cx: f32, cy: f32) -> Map {
        let cell_size = FIELD_SIZE / MAP_SIZE as f32;

        // TODO: `arr` is the same for all users on every frame. Consider caching the value.
        let mut arr = vec![vec![0; MAP_SIZE]; MAP_SIZE];
        for (_, snake) in self.snakes.iter() {
            for body in snake.bodies.iter() {
                let x = (body.x / cell_size).floor() as usize;
                let y = (body.y / cell_size).floor() as usize;
                arr[x.clamp(0, MAP_SIZE - 1)][y.clamp(0, MAP_SIZE - 1)] += 1;
            }
        }
        for (_, pellet) in self.pellets.iter() {
            let x = (pellet.position.x / cell_size).floor() as usize;
            let y = (pellet.position.y / cell_size).floor() as usize;
            arr[x.clamp(0, MAP_SIZE - 1)][y.clamp(0, MAP_SIZE - 1)] += 1;
        }

        Map {
            map: arr,
            self_coordinate: Self::map_coordinate(cx, cy),
        }
    }

    pub fn map_coordinate(cx: f32, cy: f32) -> (usize, usize) {
        let cell_size = FIELD_SIZE / MAP_SIZE as f32;
        let x = (cx.rem_euclid(FIELD_SIZE) / cell_size).floor() as usize;
        let y = (cy.rem_euclid(FIELD_SIZE) / cell_size).floor() as usize;

        (x.min(MAP_SIZE - 1), y.min(MAP_SIZE - 1))
    }

    pub fn view(&self, id: &Uuid, cx: f32, cy: f32, width: f32, height: f32) -> View {
        //! Get the view of the game.
        //! The view is centered at (cx, cy) with width and height.

        let mut snakes: Vec<Snake> = Vec::new();
        let mut pellets: Vec<Pellet> = Vec::new();

        let x0 = cx - width / 2.0;
        let y0 = cy - height / 2.0;

        // 1. Get snakes in the rectangle
        for (_, snake) in self.snakes.iter() {
            let snake = snake.clone();
            let mut bodies: VecDeque<Coordinate> = VecDeque::new();
            for body in snake.bodies.iter() {
                if body.is_in_rectangle(x0, y0, width, height) {
                    bodies.push_back(Coordinate {
                        x: (body.x - x0).rem_euclid(FIELD_SIZE),
                        y: (body.y - y0).rem_euclid(FIELD_SIZE),
                    });
                }
            }
            let is_visible_head = snake.bodies[0].is_in_rectangle(x0, y0, width, height);
            if !bodies.is_empty() {
                snakes.push(Snake {
                    bodies,
                    is_visible_head,
                    ..snake
                });
            }
        }

        // 2. Get pellets in the rectangle
        for (_, pellet) in self.pellets.iter() {
            if pellet.position.is_in_rectangle(x0, y0, width, height) {
                let pellet = pellet.clone();
                pellets.push(Pellet {
                    position: Coordinate {
                        x: (pellet.position.x - x0).rem_euclid(FIELD_SIZE),
                        y: (pellet.position.y - y0).rem_euclid(FIELD_SIZE),
                    },
                    ..pellet
                });
            }
        }

        View {
            is_alive: self.snakes.contains_key(id),
            snakes,
            pellets,
            background_offset: Coordinate {
                x: (-x0).rem_euclid(100.0),
                y: (-y0).rem_euclid(100.0),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ignores_invalid_velocity() {
        let id = Uuid::new_v4();
        let mut engine = GameEngine::new();
        engine.add_snake(id);

        engine.change_velocity(&id, Coordinate { x: 0.0, y: 0.0 });
        engine.change_velocity(
            &id,
            Coordinate {
                x: f32::NAN,
                y: 1.0,
            },
        );

        assert_eq!(
            engine.get_snake(&id).unwrap().velocity,
            Coordinate { x: 0.0, y: 0.0 }
        );
    }

    #[test]
    fn equal_head_collision_removes_exactly_one_snake() {
        let mut engine = GameEngine::new();
        let first = Uuid::new_v4();
        let second = Uuid::new_v4();
        let position = Coordinate { x: 100.0, y: 100.0 };
        engine.snakes.insert(first, Snake::new(position, 0.0));
        engine.snakes.insert(second, Snake::new(position, 0.0));

        engine.forward();

        assert_eq!(engine.snakes.len(), 1);
    }
}
