use rand::RngExt;
use std::collections::{HashMap, HashSet, VecDeque};
// TODO: Consider using BuildHasher trait
use std::cmp::Ordering;
use uuid::Uuid;

use super::coordinate::Coordinate;
use super::map::Map;
use super::pellet::Pellet;
use super::snake::Snake;
use super::view::View;

pub(crate) const FIELD_SIZE: f32 = 10000.0;
const MIN_PELLET_COUNT: usize = 5_000;
const MAP_SIZE: usize = 100;
const PELLET_CELL_SIZE: f32 = 100.0;
const PELLET_GRID_SIZE: usize = (FIELD_SIZE / PELLET_CELL_SIZE) as usize;

pub struct GameEngine {
    pub(crate) frame_count: u32,
    pub(crate) snakes: HashMap<Uuid, Snake>,
    pub(crate) pellets: HashMap<Uuid, Pellet>,
    pub(crate) pellet_grid: Vec<Vec<Uuid>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DeathEvent {
    pub id: Uuid,
    pub score: usize,
}

#[derive(Default)]
pub struct FrameEvents {
    pub deaths: Vec<DeathEvent>,
    pub pellets_eaten: HashMap<Uuid, usize>,
}

impl Default for GameEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl GameEngine {
    pub fn new() -> GameEngine {
        GameEngine {
            frame_count: 0,
            snakes: HashMap::new(),
            pellets: HashMap::new(),
            pellet_grid: vec![Vec::new(); PELLET_GRID_SIZE * PELLET_GRID_SIZE],
        }
    }

    fn pellet_cell(position: &Coordinate) -> (usize, usize) {
        let x = (position.x.rem_euclid(FIELD_SIZE) / PELLET_CELL_SIZE).floor() as usize;
        let y = (position.y.rem_euclid(FIELD_SIZE) / PELLET_CELL_SIZE).floor() as usize;
        (x.min(PELLET_GRID_SIZE - 1), y.min(PELLET_GRID_SIZE - 1))
    }

    fn pellet_cell_index(x: usize, y: usize) -> usize {
        y * PELLET_GRID_SIZE + x
    }

    fn insert_pellet_into(
        pellets: &mut HashMap<Uuid, Pellet>,
        pellet_grid: &mut [Vec<Uuid>],
        id: Uuid,
        mut pellet: Pellet,
        created_at_frame: u32,
    ) {
        pellet.frame_count_offset = created_at_frame;
        let (x, y) = Self::pellet_cell(&pellet.center);
        pellet_grid[Self::pellet_cell_index(x, y)].push(id);
        pellets.insert(id, pellet);
    }

    fn remove_pellet_from(
        pellets: &mut HashMap<Uuid, Pellet>,
        pellet_grid: &mut [Vec<Uuid>],
        id: &Uuid,
    ) -> Option<Pellet> {
        let pellet = pellets.remove(id)?;
        let (x, y) = Self::pellet_cell(&pellet.center);
        pellet_grid[Self::pellet_cell_index(x, y)].retain(|candidate| candidate != id);
        Some(pellet)
    }

    pub(crate) fn nearby_pellet_ids(pellet_grid: &[Vec<Uuid>], position: &Coordinate) -> Vec<Uuid> {
        Self::nearby_pellet_ids_with_radius(pellet_grid, position, 1)
    }

    pub(crate) fn nearby_pellet_ids_with_radius(
        pellet_grid: &[Vec<Uuid>],
        position: &Coordinate,
        radius: isize,
    ) -> Vec<Uuid> {
        let (center_x, center_y) = Self::pellet_cell(position);
        let mut ids = Vec::new();

        for dx in -radius..=radius {
            for dy in -radius..=radius {
                let x = (center_x as isize + dx).rem_euclid(PELLET_GRID_SIZE as isize) as usize;
                let y = (center_y as isize + dy).rem_euclid(PELLET_GRID_SIZE as isize) as usize;
                ids.extend_from_slice(&pellet_grid[Self::pellet_cell_index(x, y)]);
            }
        }

        ids
    }

    fn pellet_ids_in_rectangle(&self, x0: f32, y0: f32, width: f32, height: f32) -> Vec<Uuid> {
        let x_cells = axis_cells(x0, width);
        let y_cells = axis_cells(y0, height);
        let mut ids = Vec::new();

        for x in x_cells {
            for &y in &y_cells {
                ids.extend_from_slice(&self.pellet_grid[Self::pellet_cell_index(x, y)]);
            }
        }

        ids
    }

    pub fn get_random_coordinate(&self) -> Coordinate {
        let rx = rand::rng().random_range(0.0..1.0);
        let ry = rand::rng().random_range(0.0..1.0);
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

    pub fn score(&self, id: &Uuid) -> Option<usize> {
        self.snakes.get(id).map(|snake| snake.bodies.len())
    }

    pub fn add_snake(&mut self, id: Uuid) {
        let snake: Snake = Snake::new(self.get_random_coordinate(), 5.0);
        self.snakes.insert(id, snake);
    }

    pub fn add_snake_at(&mut self, id: Uuid, position: Coordinate) {
        self.snakes.insert(id, Snake::new(position, 5.0));
    }

    pub fn remove_snake(&mut self, id: &Uuid) {
        let dropped_pellets = self
            .snakes
            .get(id)
            .map(|snake| {
                snake
                    .bodies
                    .iter()
                    .filter_map(|body| {
                        if rand::rng().random_range(0..10) < 5 {
                            let dx = rand::rng().random_range(-10.0..10.0);
                            let dy = rand::rng().random_range(-10.0..10.0);
                            Some(Pellet::new_with_color_and_size(
                                Coordinate {
                                    x: body.x + dx,
                                    y: body.y + dy,
                                },
                                snake.color.clone(),
                                3,
                            ))
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        for pellet in dropped_pellets {
            Self::insert_pellet_into(
                &mut self.pellets,
                &mut self.pellet_grid,
                Uuid::new_v4(),
                pellet,
                self.frame_count,
            );
        }
        self.snakes.remove(id);
    }

    fn fill_pellet(&mut self) {
        while self.pellets.len() < MIN_PELLET_COUNT {
            let new_pellet = Pellet::new(self.get_random_coordinate());
            let id = Uuid::new_v4();
            Self::insert_pellet_into(
                &mut self.pellets,
                &mut self.pellet_grid,
                id,
                new_pellet,
                self.frame_count,
            );
        }
    }

    pub fn forward(&mut self) -> FrameEvents {
        //! Forward one frame of the game.

        let mut events = FrameEvents::default();
        let mut touched_pellets: HashSet<Uuid> = HashSet::new();

        // Update snakes
        for (snake_id, snake) in self.snakes.iter_mut() {
            let mut accelerate_factor = 1.;
            snake.turn_towards_target();

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
                let id = Uuid::new_v4();
                let pellet = Pellet::new_with_color_and_size(
                    snake.bodies.pop_back().unwrap(),
                    snake.color.clone(),
                    3,
                );
                Self::insert_pellet_into(
                    &mut self.pellets,
                    &mut self.pellet_grid,
                    id,
                    pellet,
                    self.frame_count,
                );
            }
            snake.bodies.pop_back();
            snake.bodies.push_front(new_head);

            let mut eaten_pellets: Vec<Uuid> = Vec::new();

            let nearby_pellets = Self::nearby_pellet_ids(&self.pellet_grid, &new_head);
            for id in nearby_pellets {
                let Some(pellet) = self.pellets.get_mut(&id) else {
                    continue;
                };
                // Draw pellets towards the snake
                if pellet.position.distance2(&new_head) < ((snake.size * 2).pow(2) as f32) {
                    let nx = pellet.position.x + (new_head.x - pellet.position.x) / 5.;
                    let ny = pellet.position.y + (new_head.y - pellet.position.y) / 5.;
                    pellet.position = Coordinate { x: nx, y: ny };
                    touched_pellets.insert(id);
                }

                // Eat pellets
                if pellet.position.distance2(&new_head) < (snake.size.pow(2) as f32) {
                    snake.bodies.push_back(snake.get_tail().to_owned());
                    eaten_pellets.push(id);
                }
            }

            for id in eaten_pellets.iter() {
                Self::remove_pellet_from(&mut self.pellets, &mut self.pellet_grid, id);
            }
            if !eaten_pellets.is_empty() {
                events.pellets_eaten.insert(*snake_id, eaten_pellets.len());
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
                                if rand::rng().random_range(0..10) < 5 {
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
            if let Some(score) = self.score(id) {
                events.deaths.push(DeathEvent { id: *id, score });
            }
            self.remove_snake(id)
        }

        // Refill pellets
        self.fill_pellet();

        // Update time to live
        // The orbit is a visual effect. Updating trigonometry for all 5,000
        // pellets every server frame caused the low-CPU production server to
        // miss most of its 30 Hz deadlines. Only attracted pellets mutate on
        // the server; visible orbiting is calculated per client below.
        let mut moved_pellets = Vec::with_capacity(touched_pellets.len());
        for id in touched_pellets {
            if let Some(pellet) = self.pellets.get_mut(&id) {
                let previous_cell = Self::pellet_cell(&pellet.center);
                let next_cell = Self::pellet_cell(&pellet.position);
                pellet.center = pellet.position;
                if previous_cell != next_cell {
                    moved_pellets.push((id, previous_cell, next_cell));
                }
            }
        }
        for (id, (previous_x, previous_y), (next_x, next_y)) in moved_pellets {
            self.pellet_grid[Self::pellet_cell_index(previous_x, previous_y)]
                .retain(|candidate| candidate != &id);
            self.pellet_grid[Self::pellet_cell_index(next_x, next_y)].push(id);
        }
        for (_, snake) in self.snakes.iter_mut() {
            snake.frame_count_offset += 1;
        }
        self.frame_count += 1;
        events
    }

    pub fn change_velocity(&mut self, id: &Uuid, velocity: Coordinate) {
        if !velocity.x.is_finite() || !velocity.y.is_finite() {
            return;
        }

        if let Some(snake) = self.snakes.get_mut(id) {
            let norm = (velocity.x.powi(2) + velocity.y.powi(2)).sqrt();
            if norm <= f32::EPSILON || !norm.is_finite() {
                return;
            }
            snake.target_velocity = Coordinate {
                x: velocity.x / norm,
                y: velocity.y / norm,
            };
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
        for pellet in self.pellets.values() {
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
        for id in self.pellet_ids_in_rectangle(x0, y0, width, height) {
            let Some(pellet) = self.pellets.get(&id) else {
                continue;
            };
            if pellet.position.is_in_rectangle(x0, y0, width, height) {
                let mut pellet = pellet.clone();
                pellet.frame_count_offset =
                    self.frame_count.wrapping_sub(pellet.frame_count_offset);
                pellet.update();
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

fn axis_cells(start: f32, length: f32) -> Vec<usize> {
    if length >= FIELD_SIZE {
        return (0..PELLET_GRID_SIZE).collect();
    }

    let first_visible_cell = (start.rem_euclid(FIELD_SIZE) / PELLET_CELL_SIZE).floor() as usize;
    let start_cell = (first_visible_cell + PELLET_GRID_SIZE - 1) % PELLET_GRID_SIZE;
    let cell_count = (length.max(0.0) / PELLET_CELL_SIZE).ceil() as usize + 3;
    (0..cell_count.min(PELLET_GRID_SIZE))
        .map(|offset| (start_cell + offset) % PELLET_GRID_SIZE)
        .collect()
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

    #[test]
    fn default_engine_initializes_the_pellet_grid() {
        let engine = GameEngine::default();

        assert_eq!(
            engine.pellet_grid.len(),
            PELLET_GRID_SIZE * PELLET_GRID_SIZE
        );
    }

    #[test]
    fn spatial_grid_finds_pellets_across_the_field_boundary() {
        let mut engine = GameEngine::new();
        let pellet_id = Uuid::new_v4();
        let pellet = Pellet::new(Coordinate {
            x: FIELD_SIZE - 10.0,
            y: 50.0,
        });
        GameEngine::insert_pellet_into(
            &mut engine.pellets,
            &mut engine.pellet_grid,
            pellet_id,
            pellet,
            engine.frame_count,
        );

        let nearby =
            GameEngine::nearby_pellet_ids(&engine.pellet_grid, &Coordinate { x: 5.0, y: 50.0 });

        assert!(nearby.contains(&pellet_id));
    }
}
