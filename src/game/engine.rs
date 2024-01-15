use rand::Rng;
use std::collections::{HashMap, HashSet, VecDeque};
// TODO: Consider using BuildHasher trait
use num_traits::Float;
use std::cmp::Ordering;
use uuid::Uuid;

use super::coordinate::Coordinate;
use super::map::Map;
use super::pellet::Pellet;
use super::snake::Snake;
use super::view::View;
use super::FIELD_SIZE;

const MAX_PELLET_COUNT: usize = 5_000;

pub struct GameEngine<T> {
    frame_count: u32,
    snakes: HashMap<Uuid, Snake<T>>,
    pellets: HashMap<Uuid, Pellet<T>>,
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
        }
    }

    pub fn get_random_coordinate(&self) -> Coordinate<T> {
        let rx = rand::thread_rng().gen_range((0.0)..=1.0);
        let ry = rand::thread_rng().gen_range((0.0)..=1.0);
        let x = FIELD_SIZE.into() * rx.into();
        let y = FIELD_SIZE.into() * ry.into();
        Coordinate { x, y }
    }

    pub fn get_snake(&self, id: &Uuid) -> Option<&Snake<T>> {
        self.snakes.get(id)
    }

    pub fn get_snake_mut(&mut self, id: &Uuid) -> Option<&mut Snake<T>> {
        self.snakes.get_mut(id)
    }

    pub fn add_snake(&mut self, id: Uuid) {
        let snake: Snake<T> = Snake::new(self.get_random_coordinate(), 5.0.into());
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
            let mut accelerate_factor = T::one();

            if snake.acceleration_time_left > 0 {
                snake.acceleration_time_left -= 1;
                accelerate_factor = T::from(2).unwrap();
            }

            let head = snake.get_head();
            let new_head = Coordinate {
                x: head.x + snake.velocity.x * snake.speed * accelerate_factor,
                y: head.y + snake.velocity.y * snake.speed * accelerate_factor,
            };
            let new_head = Coordinate {
                x: (new_head.x + FIELD_SIZE.into()) % FIELD_SIZE.into(),
                y: (new_head.y + FIELD_SIZE.into()) % FIELD_SIZE.into(),
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
            snake.bodies.push_front(new_head.clone());

            let mut eaten_pellets: Vec<Uuid> = Vec::new();

            for (id, pellet) in self.pellets.iter_mut() {
                // Draw pellets towards the snake
                if pellet.position.distance2(&new_head) < T::from((snake.size * 2).pow(2)).unwrap()
                {
                    let nx = pellet.position.x + (new_head.x - pellet.position.x) / 5f32.into();
                    let ny = pellet.position.y + (new_head.y - pellet.position.y) / 5f32.into();
                    pellet.position = Coordinate { x: nx, y: ny };
                } else {
                    pellet.update();
                }

                // Eat pellets
                if pellet.position.distance2(&new_head) < T::from(snake.size.pow(2)).unwrap() {
                    snake.bodies.push_back(snake.get_tail().clone());
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

        for (id1, snake1) in self.snakes.iter() {
            for (id2, snake2) in self.snakes.iter() {
                if id1 == id2 {
                    continue;
                }
                let head1 = snake1.get_head();
                let head2 = snake2.get_head();

                // the head to head collision, rules:
                // 1. the acceleration snake wins
                // 2. the bigger snake wins
                // 3. random
                if head1.distance2(head2) <= T::from((snake1.size + snake2.size).pow(2)).unwrap() {
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
                                if rand::thread_rng().gen_range(0..=10) < 5 {
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

                for body2 in snake2.bodies.iter() {
                    if head1.distance2(body2)
                        <= T::from((snake1.size + snake2.size).pow(2)).unwrap()
                    {
                        dead_snakes.insert(*id1);
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

    pub fn view(&self, id: &Uuid, cx: T, cy: T, width: T, height: T) -> View<T> {
        //! Get the view of the game.
        //! The view is centered at (cx, cy) with width and height.

        let mut snakes: Vec<Snake<T>> = Vec::new();
        let mut pellets: Vec<Pellet<T>> = Vec::new();

        let x0 = cx - width / 2.0.into();
        let y0 = cy - height / 2.0.into();

        // 1. Get snakes in the rectangle
        for (_, snake) in self.snakes.iter() {
            let snake = snake.clone();
            let mut bodies: VecDeque<Coordinate<T>> = VecDeque::new();
            for body in snake.bodies.iter() {
                if body.is_in_rectangle(x0, y0, width, height) {
                    let body = body.clone();
                    bodies.push_back(Coordinate {
                        x: (body.x - x0 + FIELD_SIZE.into()) % FIELD_SIZE.into(),
                        y: (body.y - y0 + FIELD_SIZE.into()) % FIELD_SIZE.into(),
                    });
                }
            }
            let is_visible_head = snake.bodies[0].is_in_rectangle(x0, y0, width, height);
            snakes.push(Snake {
                bodies,
                is_visible_head,
                ..snake
            });
        }

        // 2. Get pellets in the rectangle
        for (_, pellet) in self.pellets.iter() {
            if pellet.position.is_in_rectangle(x0, y0, width, height) {
                let pellet = pellet.clone();
                pellets.push(Pellet {
                    position: Coordinate {
                        x: (pellet.position.x - x0 + FIELD_SIZE.into()) % FIELD_SIZE.into(),
                        y: (pellet.position.y - y0 + FIELD_SIZE.into()) % FIELD_SIZE.into(),
                    },
                    ..pellet
                });
            }
        }

        // 3. Create the map
        const SIZE: usize = 100;
        let cell_size = FIELD_SIZE / SIZE as f32;

        // TODO: `arr` is the same for all users on every frame. Consider caching the value.
        let mut arr = vec![vec![0; SIZE]; SIZE];
        for (_, snake) in self.snakes.iter() {
            for body in snake.bodies.iter() {
                let x = (body.x.to_f32().unwrap() / cell_size).floor() as usize;
                let y = (body.y.to_f32().unwrap() / cell_size).floor() as usize;
                arr[x.clamp(0, SIZE - 1)][y.clamp(0, SIZE - 1)] += 1;
            }
        }
        for (_, pellet) in self.pellets.iter() {
            let x = (pellet.position.x.to_f32().unwrap() / cell_size).floor() as usize;
            let y = (pellet.position.y.to_f32().unwrap() / cell_size).floor() as usize;
            arr[x.clamp(0, SIZE - 1)][y.clamp(0, SIZE - 1)] += 1;
        }

        let self_coordinate = (
            (cx.to_f32().unwrap() / cell_size).floor() as usize,
            (cy.to_f32().unwrap() / cell_size).floor() as usize,
        );

        let map = Map {
            map: arr,
            self_coordinate,
        };

        // 4. Get background_dots in the rectangle
        let mut background_dots: Vec<Coordinate<T>> = Vec::new();

        for x in 0..100 {
            for y in 0..100 {
                let hex = Coordinate {
                    x: ((x * 100) as f32).into(),
                    y: ((y * 100) as f32).into(),
                };
                if hex.is_in_rectangle(x0, y0, width, height) {
                    background_dots.push(Coordinate {
                        x: (hex.x - x0 + FIELD_SIZE.into()) % FIELD_SIZE.into(),
                        y: (hex.y - y0 + FIELD_SIZE.into()) % FIELD_SIZE.into(),
                    });
                }
            }
        }

        View {
            is_alive: self.snakes.contains_key(id),
            snakes,
            pellets,
            background_dots,
            map,
        }
    }
}
