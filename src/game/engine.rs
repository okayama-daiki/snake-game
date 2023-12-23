use rand::Rng;
use std::collections::{HashMap, VecDeque};
// TODO: Consider using BuildHasher trait
use num_traits::Float;
use uuid::Uuid;

use super::coordinate::Coordinate;
use super::pellet::Pellet;
use super::snake::Snake;
use super::view::View;

const MAX_PELLET_COUNT: usize = 5_000;
const PELLET_REACT_DISTANCE: f32 = 30.;
const SNAKE_FEED_DISTANCE: f32 = 20.;

pub struct GameEngine<T> {
    frame_count: u32,
    snakes: HashMap<Uuid, Snake<T>>,
    pellets: HashMap<Uuid, Pellet<T>>,
    min: Coordinate<T>,
    max: Coordinate<T>,
}

impl<T> GameEngine<T>
where
    T: Float,
    f32: Into<T>,
{
    pub fn new() -> GameEngine<T> {
        GameEngine {
            frame_count: 0,
            snakes: HashMap::new(),
            pellets: HashMap::new(),
            min: Coordinate {
                x: (-5000.0).into(),
                y: (-5000.0).into(),
            },
            max: Coordinate {
                x: 5000.0.into(),
                y: 5000.0.into(),
            },
        }
    }

    pub fn get_random_coordinate(&self) -> Coordinate<T> {
        let rx = rand::thread_rng().gen_range((0.0)..=1.0);
        let ry = rand::thread_rng().gen_range((0.0)..=1.0);
        let x = self.min.x + (self.max.x - self.min.x) * rx.into();
        let y = self.min.y + (self.max.y - self.min.y) * ry.into();
        Coordinate { x, y }
    }

    pub fn get_snake(&self, id: &Uuid) -> Option<&Snake<T>> {
        self.snakes.get(id)
    }

    pub fn add_snake(&mut self, initial_position: Coordinate<T>, id: Uuid) {
        let snake: Snake<T> = Snake::new(initial_position, 5.0.into());
        self.snakes.insert(id, snake);
    }

    pub fn remove_snake(&mut self, id: &Uuid) {
        if let Some(snake) = self.snakes.get(id) {
            for body in snake.bodies.iter() {
                if rand::thread_rng().gen_range(0..=10) < 5 {
                    let body = body.clone();
                    let dx = rand::thread_rng().gen_range((-10.)..=10.).into();
                    let dy = rand::thread_rng().gen_range((-10.)..=10.).into();
                    let pellet = Pellet::new_with_color_and_size(
                        Coordinate {
                            x: body.x + dx,
                            y: body.y + dy,
                        },
                        "120".to_string(),
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
            let position = self.get_random_coordinate();
            let pellet = Pellet::new(position.clone());
            let id = Uuid::new_v4();
            self.pellets.insert(id, pellet);
        }
    }

    pub fn forward(&mut self) {
        //! Forward one frame of the game.

        // Update snakes
        for (_, snake) in self.snakes.iter_mut() {
            let head = snake.get_head();
            let new_head = Coordinate {
                x: head.x + snake.velocity.x * snake.speed,
                y: head.y + snake.velocity.y * snake.speed,
            };

            snake.bodies.pop_back();
            snake.bodies.push_front(new_head.clone());

            let mut eaten_pellets: Vec<Uuid> = Vec::new();

            for (id, pellet) in self.pellets.iter_mut() {
                // Draw pellets towards the snake
                if pellet.position.distance2(&new_head) < PELLET_REACT_DISTANCE.powi(2).into() {
                    let nx = pellet.position.x + (new_head.x - pellet.position.x) / 7f32.into();
                    let ny = pellet.position.y + (new_head.y - pellet.position.y) / 7f32.into();
                    pellet.position = Coordinate { x: nx, y: ny };
                }

                // Eat pellets
                if pellet.position.distance2(&new_head) < SNAKE_FEED_DISTANCE.powi(2).into() {
                    snake.bodies.push_front(snake.get_head().clone());
                    eaten_pellets.push(*id);
                }
            }

            for id in eaten_pellets.iter() {
                self.pellets.remove(id);
            }
        }

        // Detect collision
        let mut dead_snakes: Vec<Uuid> = Vec::new();

        for (id, snake) in self.snakes.iter() {
            let head = snake.get_head();
            if head.x < self.min.x
                || head.x > self.max.x
                || head.y < self.min.y
                || head.y > self.max.y
            {
                dead_snakes.push(*id);
            }
            for (id2, snake2) in self.snakes.iter() {
                if id == id2 {
                    continue;
                }
                for body in snake2.bodies.iter() {
                    if head.distance2(body) < SNAKE_FEED_DISTANCE.powi(2).into() {
                        dead_snakes.push(*id);
                    }
                }
            }
        }

        for id in dead_snakes.iter() {
            self.remove_snake(id)
        }

        // Refill pellets
        self.fill_pellet();

        // Update time to live
        for (_, pellet) in self.pellets.iter_mut() {
            pellet.frame_count_offset += 1;
        }
        for (_, snake) in self.snakes.iter_mut() {
            snake.frame_count_offset += 1;
        }
        self.frame_count += 1;
    }

    pub fn change_velocity(&mut self, id: &Uuid, velocity: Coordinate<T>) {
        if let Some(snake) = self.snakes.get_mut(id) {
            let weight = 0.2.into();
            let new_velocity = Coordinate {
                x: (T::one() - weight) * snake.velocity.x + weight * velocity.x,
                y: (T::one() - weight) * snake.velocity.y + weight * velocity.y,
            };
            let norm = (new_velocity.x.powi(2) + new_velocity.y.powi(2)).sqrt();
            let new_velocity = Coordinate {
                x: new_velocity.x / norm,
                y: new_velocity.y / norm,
            };
            snake.velocity = new_velocity;
        }
    }

    pub fn view(&self, id: &Uuid, x: T, y: T, width: T, height: T) -> View<T> {
        let mut snakes: Vec<Snake<T>> = Vec::new();
        let mut pellets: Vec<Pellet<T>> = Vec::new();

        let x0 = x - width / 2.0.into();
        let y0 = y - height / 2.0.into();

        for (_, snake) in self.snakes.iter() {
            let snake = snake.clone();
            let mut bodies: VecDeque<Coordinate<T>> = VecDeque::new();
            for body in snake.bodies.iter() {
                if body.is_in_rectangle(x0, y0, width, height) {
                    let body = body.clone();
                    bodies.push_back(Coordinate {
                        x: body.x - x0,
                        y: body.y - y0,
                    });
                }
            }
            snakes.push(Snake { bodies, ..snake });
        }

        for (_, pellet) in self.pellets.iter() {
            if pellet.position.is_in_rectangle(x0, y0, width, height) {
                let pellet = pellet.clone();
                pellets.push(Pellet {
                    position: Coordinate {
                        x: pellet.position.x - x0,
                        y: pellet.position.y - y0,
                    },
                    ..pellet
                });
            }
        }

        View {
            is_alive: self.snakes.contains_key(id),
            snakes,
            pellets,
        }
    }
}
