use game::{coordinate::Coordinate, map::Map, pellet::Pellet, snake::Snake, view::View as Message};

#[macro_use]
mod browser;
use browser::{
    canvas, create_mouse_position_tracker, get_center_coordinate, get_context, get_height,
    get_width, now, window,
};
use std::rc::Rc;
use std::{
    cell::{Cell, RefCell},
    collections::VecDeque,
};
use wasm_bindgen::{
    prelude::{wasm_bindgen, Closure, JsValue},
    Clamped, JsCast,
};
use web_sys::{
    js_sys::{ArrayBuffer, Function, Uint8Array},
    BinaryType, CanvasRenderingContext2d, HtmlCanvasElement, ImageData, MessageEvent, MouseEvent,
    WebSocket,
};

static MINIMAP_SIZE: f64 = 100.;
static GLOBAL_MARGIN: f64 = 50.;
const PERFORMANCE_SAMPLE_COUNT: usize = 30;
const TARGET_FRAME_INTERVAL_MS: f64 = 1000.0 / 60.0;
const SERVER_FRAME_INTERVAL_MS: f64 = 1000.0 / 30.0;
const JITTER_BUFFER_FRAMES: f64 = 4.0;
const MAX_BUFFERED_SNAPSHOTS: usize = 16;
type AnimationFrameCallback = Rc<RefCell<Option<Closure<dyn FnMut(f64)>>>>;

struct Snapshot {
    message: Message,
    opacity: f64,
    sequence: u64,
}

#[derive(Default)]
struct RenderState {
    snapshots: VecDeque<Snapshot>,
    next_sequence: u64,
}

// ref: https://rustwasm.github.io/docs/book/game-of-life/debugging.html
// A macro to provide `println!(..)`-style syntax for `console.log` logging.
#[allow(unused_macros)]
macro_rules! log {
    ( $( $t:tt )* ) => {
        web_sys::console::log_1(&format!( $( $t )* ).into());
    }
}

#[wasm_bindgen]
extern "C" {}

#[wasm_bindgen]
pub struct RenderEngine {
    canvas: HtmlCanvasElement,
    socket: WebSocket,
    callback: Function,
    player_token: String,
    on_resize: Option<Closure<dyn FnMut()>>,
    on_message: Option<Closure<dyn FnMut(MessageEvent)>>,
    on_mouse_move: Option<Closure<dyn FnMut(MouseEvent)>>,
    on_mouse_down: Option<Closure<dyn FnMut()>>,
    on_mouse_up: Option<Closure<dyn FnMut()>>,
    interval_callback: Option<Closure<dyn FnMut()>>,
    interval_id: Option<i32>,
    animation_frame_callback: Option<AnimationFrameCallback>,
    animation_frame_id: Option<Rc<Cell<Option<i32>>>>,
}

#[wasm_bindgen]
impl RenderEngine {
    #[wasm_bindgen(constructor)]
    pub fn new(
        canvas: HtmlCanvasElement,
        socket: WebSocket,
        callback: Function,
        player_token: String,
    ) -> Self {
        Self {
            canvas,
            socket,
            callback,
            player_token,
            on_resize: None,
            on_message: None,
            on_mouse_move: None,
            on_mouse_down: None,
            on_mouse_up: None,
            interval_callback: None,
            interval_id: None,
            animation_frame_callback: None,
            animation_frame_id: None,
        }
    }

    pub fn init(&mut self) {
        console_error_panic_hook::set_once();
        self.socket.set_binary_type(BinaryType::Arraybuffer);

        // 1. Set the canvas size to the window size.
        self.canvas.set_height(get_height());
        self.canvas.set_width(get_width());
        get_context(&self.canvas)
            .translate(-GLOBAL_MARGIN, -GLOBAL_MARGIN)
            .unwrap();

        // 2. Add a resize event handler to the window so that the canvas dynamically resizes and sends it to the server.
        {
            let socket = self.socket.clone();
            let canvas = self.canvas.clone();
            let on_resize = Closure::wrap(Box::new(move || {
                canvas.set_height(get_height());
                canvas.set_width(get_width());
                get_context(&canvas)
                    .translate(-GLOBAL_MARGIN, -GLOBAL_MARGIN)
                    .unwrap();
                socket
                    .send_with_str(format!("w {} {}", get_width(), get_height()).as_str())
                    .ok();
            }) as Box<dyn FnMut()>);
            window()
                .unwrap()
                .set_onresize(Some(on_resize.as_ref().unchecked_ref()));
            self.on_resize = Some(on_resize);
        }

        let render_state = Rc::new(RefCell::new(RenderState::default()));
        let context = get_context(&self.canvas);
        let minimap_canvas = canvas().unwrap();
        minimap_canvas.set_height(MINIMAP_SIZE as u32);
        minimap_canvas.set_width(MINIMAP_SIZE as u32);
        let minimap_context = get_context(&minimap_canvas);
        let mouse_tracker = create_mouse_position_tracker();
        window()
            .unwrap()
            .set_onmousemove(Some(mouse_tracker.handler.as_ref().unchecked_ref()));
        let mouse_position = mouse_tracker.position;
        self.on_mouse_move = Some(mouse_tracker.handler);

        // 3. Buffer WebSocket snapshots. Painting directly in this callback
        // made network jitter visible as dropped frames.
        {
            let is_alive = Cell::new(true);
            let frame_after_death = Cell::new(0);
            let socket = self.socket.clone();
            let callback = self.callback.clone();
            let mouse_position = mouse_position.clone();
            let render_state = render_state.clone();
            let minimap_context = minimap_context.clone();
            let on_message = Closure::wrap(Box::new(move |e: MessageEvent| {
                let array_buffer = e.data().dyn_into::<ArrayBuffer>().unwrap();
                let array = Uint8Array::new(&array_buffer);
                let vec = array.to_vec();

                if let Ok(message) = Message::from_bytes(&vec) {
                    // if the snake is dead, gradually darken the screen and call the callback function when the screen is completely dark.
                    if !message.is_alive && is_alive.get() {
                        is_alive.set(false);
                        frame_after_death.set(1);
                    }
                    if !is_alive.get() {
                        frame_after_death.set(frame_after_death.get() + 1);
                    }
                    if frame_after_death.get() == 150 {
                        callback.call0(&JsValue::NULL).unwrap();
                    }
                    let mut state = render_state.borrow_mut();
                    let sequence = state.next_sequence;
                    state.next_sequence += 1;
                    state.snapshots.push_back(Snapshot {
                        message,
                        opacity: 1. - (frame_after_death.get() as f64 - 50.).max(0.) / 100.,
                        sequence,
                    });
                    if state.snapshots.len() > MAX_BUFFERED_SNAPSHOTS {
                        state.snapshots.pop_front();
                    }
                }

                if let Ok(map) = Map::from_bytes(&vec) {
                    update_minimap(&minimap_context, &map);
                }

                // Send the normalized mouse direction to the server.
                if is_alive.get() {
                    let dir = vector(&get_center_coordinate(), &mouse_position.get());
                    socket
                        .send_with_str(format!("v {} {}", dir.x, dir.y).as_str())
                        .ok();
                }
            }) as Box<dyn FnMut(MessageEvent)>);
            self.socket
                .set_onmessage(Some(on_message.as_ref().unchecked_ref()));
            self.on_message = Some(on_message);
        }

        // 4. Paint at 60 Hz from a short jitter buffer. Render's proxy delivers
        // several 30 Hz snapshots in bursts, so arrival timestamps cannot be
        // used as the game timeline.
        {
            let animation_frame_callback: AnimationFrameCallback = Rc::new(RefCell::new(None));
            let animation_frame_id = Rc::new(Cell::new(None));
            let callback_for_frame = animation_frame_callback.clone();
            let id_for_frame = animation_frame_id.clone();
            let render_state = render_state.clone();
            let context = context.clone();
            let minimap_context = minimap_context.clone();
            let mouse_position = mouse_position.clone();
            let mut frame_count = 0;
            let mut frame_times: VecDeque<f64> = VecDeque::with_capacity(PERFORMANCE_SAMPLE_COUNT);
            let mut last_callback_time = now().unwrap();
            let mut last_render_time = last_callback_time;
            let mut accumulated_time = TARGET_FRAME_INTERVAL_MS;
            let mut playback_position: Option<f64> = None;

            *animation_frame_callback.borrow_mut() =
                Some(Closure::wrap(Box::new(move |timestamp: f64| {
                    accumulated_time += (timestamp - last_callback_time).clamp(0.0, 100.0);
                    last_callback_time = timestamp;
                    if accumulated_time + 0.5 >= TARGET_FRAME_INTERVAL_MS {
                        accumulated_time %= TARGET_FRAME_INTERVAL_MS;
                        let frame_duration = timestamp - last_render_time;
                        last_render_time = timestamp;
                        if frame_times.len() == PERFORMANCE_SAMPLE_COUNT {
                            frame_times.pop_front();
                        }
                        frame_times.push_back(frame_duration);

                        let state = render_state.borrow();
                        if playback_position.is_none()
                            && state.snapshots.len() > JITTER_BUFFER_FRAMES as usize
                        {
                            let newest = state.snapshots.back().unwrap().sequence as f64;
                            playback_position = Some(newest - JITTER_BUFFER_FRAMES);
                        }
                        if let Some(position) = playback_position.as_mut() {
                            let oldest = state.snapshots.front().unwrap().sequence as f64;
                            let newest = state.snapshots.back().unwrap().sequence as f64;
                            let buffered_frames = newest - *position;
                            *position += frame_duration / SERVER_FRAME_INTERVAL_MS
                                * playback_rate(buffered_frames);
                            *position = position.clamp(oldest, newest);

                            let (previous, current, amount) =
                                snapshot_pair(&state.snapshots, *position).unwrap();
                            context.set_global_alpha(
                                previous.opacity
                                    + (current.opacity - previous.opacity) * amount as f64,
                            );
                            let camera =
                                interpolated_camera(&previous.message, &current.message, amount);
                            render(
                                &context,
                                &minimap_context,
                                Some(&previous.message),
                                &current.message,
                                amount,
                                &camera,
                                &mouse_position.get(),
                            );
                        }
                        drop(state);

                        frame_count += 1;
                        if frame_count % PERFORMANCE_SAMPLE_COUNT == 0 && !frame_times.is_empty() {
                            log!(
                                "Display FPS: {}",
                                1000.
                                    / (frame_times.iter().sum::<f64>() / frame_times.len() as f64)
                            );
                        }
                    }

                    if let Some(callback) = callback_for_frame.borrow().as_ref() {
                        let id = window()
                            .unwrap()
                            .request_animation_frame(callback.as_ref().unchecked_ref())
                            .unwrap();
                        id_for_frame.set(Some(id));
                    }
                }) as Box<dyn FnMut(f64)>));

            let id = window()
                .unwrap()
                .request_animation_frame(
                    animation_frame_callback
                        .borrow()
                        .as_ref()
                        .unwrap()
                        .as_ref()
                        .unchecked_ref(),
                )
                .unwrap();
            animation_frame_id.set(Some(id));
            self.animation_frame_callback = Some(animation_frame_callback);
            self.animation_frame_id = Some(animation_frame_id);
        }

        // 5. Add a mousedown handler to the window so that the snake can accelerate when the window is clicked.
        {
            let socket = self.socket.clone();
            let is_mousedown = Rc::new(Cell::new(false));
            let is_mousedown_for_mousedown = is_mousedown.clone();
            let is_mousedown_for_mouseup = is_mousedown.clone();
            let interval_callback = Closure::wrap(Box::new(move || {
                if is_mousedown.get() {
                    socket.send_with_str("a").ok();
                }
            }) as Box<dyn FnMut()>);
            let interval_id = window()
                .unwrap()
                .set_interval_with_callback_and_timeout_and_arguments_0(
                    interval_callback.as_ref().unchecked_ref(),
                    100,
                )
                .unwrap();
            let on_mousedown = Closure::wrap(Box::new(move || {
                is_mousedown_for_mousedown.set(true);
            }) as Box<dyn FnMut()>);
            let on_mouseup = Closure::wrap(Box::new(move || {
                is_mousedown_for_mouseup.set(false);
            }) as Box<dyn FnMut()>);

            window()
                .unwrap()
                .set_onmousedown(Some(on_mousedown.as_ref().unchecked_ref()));
            window()
                .unwrap()
                .set_onmouseup(Some(on_mouseup.as_ref().unchecked_ref()));
            self.interval_callback = Some(interval_callback);
            self.interval_id = Some(interval_id);
            self.on_mouse_down = Some(on_mousedown);
            self.on_mouse_up = Some(on_mouseup);
        }

        // 6. Finally, send a start message to the server, and start the game.
        self.socket
            .send_with_str(format!("s {}", self.player_token).as_str())
            .ok();
        self.socket
            .send_with_str(format!("w {} {}", self.canvas.width(), self.canvas.height()).as_str())
            .ok();
    }

    pub fn destroy(&mut self) {
        self.socket.set_onmessage(None);
        if let Ok(window) = window() {
            window.set_onresize(None);
            window.set_onmousemove(None);
            window.set_onmousedown(None);
            window.set_onmouseup(None);
            if let Some(interval_id) = self.interval_id.take() {
                window.clear_interval_with_handle(interval_id);
            }
            if let Some(animation_frame_id) = self.animation_frame_id.take() {
                if let Some(id) = animation_frame_id.take() {
                    window.cancel_animation_frame(id).ok();
                }
            }
        }

        self.on_resize = None;
        self.on_message = None;
        self.on_mouse_move = None;
        self.on_mouse_down = None;
        self.on_mouse_up = None;
        self.interval_callback = None;
        if let Some(animation_frame_callback) = self.animation_frame_callback.take() {
            animation_frame_callback.borrow_mut().take();
        }
    }
}

fn render(
    context: &CanvasRenderingContext2d,
    minimap_context: &CanvasRenderingContext2d,
    previous: Option<&Message>,
    current: &Message,
    amount: f32,
    camera: &(Coordinate, Coordinate),
    mouse_position: &Coordinate,
) {
    context.clear_rect(
        0.0,
        0.0,
        (get_width() + 100) as f64,
        (get_height() + 100) as f64,
    );
    render_background(context, &camera.0);
    context.save();
    context
        .translate(camera.1.x as f64, camera.1.y as f64)
        .unwrap();
    render_pellets(context, &current.pellets);
    context.restore();
    render_snakes(
        context,
        previous.map(|message| message.snakes.as_slice()),
        &current.snakes,
        amount,
        mouse_position,
    );
    render_minimap(context, minimap_context);
}

fn snapshot_pair(
    snapshots: &VecDeque<Snapshot>,
    position: f64,
) -> Option<(&Snapshot, &Snapshot, f32)> {
    let previous = snapshots
        .iter()
        .rev()
        .find(|snapshot| snapshot.sequence as f64 <= position)
        .or_else(|| snapshots.front())?;
    let current = snapshots
        .iter()
        .find(|snapshot| snapshot.sequence as f64 >= position)
        .or_else(|| snapshots.back())?;
    let span = current.sequence.saturating_sub(previous.sequence) as f64;
    let amount = if span > 0.0 {
        ((position - previous.sequence as f64) / span) as f32
    } else {
        0.0
    };

    Some((previous, current, amount.clamp(0.0, 1.0)))
}

fn playback_rate(buffered_frames: f64) -> f64 {
    if buffered_frames < 1.5 {
        0.8
    } else if buffered_frames > JITTER_BUFFER_FRAMES + 2.0 {
        1.1
    } else {
        1.0
    }
}

fn interpolated_camera(
    previous: &Message,
    current: &Message,
    amount: f32,
) -> (Coordinate, Coordinate) {
    let background_offset = Coordinate {
        x: lerp_wrapped(
            previous.background_offset.x,
            current.background_offset.x,
            amount,
            100.0,
        ),
        y: lerp_wrapped(
            previous.background_offset.y,
            current.background_offset.y,
            amount,
            100.0,
        ),
    };
    let camera_shift = Coordinate {
        x: wrapped_delta(background_offset.x, current.background_offset.x, 100.0),
        y: wrapped_delta(background_offset.y, current.background_offset.y, 100.0),
    };

    (background_offset, camera_shift)
}

fn lerp_wrapped(start: f32, end: f32, amount: f32, period: f32) -> f32 {
    (start + wrapped_delta(end, start, period) * amount).rem_euclid(period)
}

fn wrapped_delta(value: f32, origin: f32, period: f32) -> f32 {
    (value - origin + period / 2.0).rem_euclid(period) - period / 2.0
}

fn render_pellets(context: &CanvasRenderingContext2d, pellets: &Vec<Pellet>) {
    context.set_shadow_blur(0.0);
    for pellet in pellets {
        let hsl = pellet_rendering_helper::to_hsl(pellet);
        context.set_fill_style_str(hsl.as_str());
        context.begin_path();
        context
            .arc(
                pellet.position.x as f64,
                pellet.position.y as f64,
                pellet_rendering_helper::to_radius(pellet),
                0.0,
                std::f64::consts::PI * 2.0,
            )
            .unwrap();
        context.fill();
    }
}

fn render_snakes(
    context: &CanvasRenderingContext2d,
    previous_snakes: Option<&[Snake]>,
    snakes: &[Snake],
    amount: f32,
    mouse_position: &Coordinate,
) {
    context.set_shadow_blur(0.0);
    let screen_center = get_center_coordinate();
    let self_head_position = Coordinate {
        x: screen_center.x + GLOBAL_MARGIN as f32,
        y: screen_center.y + GLOBAL_MARGIN as f32,
    };
    let cursor_direction = vector(&screen_center, mouse_position);
    for (snake_index, snake) in snakes.iter().enumerate() {
        // Draw the body
        let previous_snake = matching_previous_snake(previous_snakes, snake_index, snake);
        let snake_size = interpolated_snake_size(previous_snake, snake, amount);
        let hsl = snake_rendering_helper::to_hsl(snake);
        let bodies: Vec<_> = (0..snake.bodies.len())
            .map(|body_index| interpolated_body(previous_snake, snake, body_index, amount))
            .collect();
        let head = snake.is_visible_head.then(|| bodies[0]);
        let is_self = head.is_some_and(|head| {
            (head.x - self_head_position.x).abs() < 1.0
                && (head.y - self_head_position.y).abs() < 1.0
        });

        for body in bodies.iter().rev() {
            context.set_fill_style_str("rgba(0, 0, 0, 0.3)");
            context.set_shadow_color("rgba(0, 0, 0, 0.3)");
            context.set_shadow_blur(10.0);
            context.begin_path();
            context
                .arc(
                    body.x as f64,
                    body.y as f64,
                    snake_size,
                    0.0,
                    std::f64::consts::PI * 2.0,
                )
                .unwrap();
            context.fill();

            context.set_fill_style_str(hsl.as_str());
            context.set_shadow_color(hsl.as_str());
            context.set_shadow_blur(snake_glow_blur(snake));
            context.begin_path();
            context
                .arc(
                    body.x as f64,
                    body.y as f64,
                    snake_size,
                    0.0,
                    std::f64::consts::PI * 2.0,
                )
                .unwrap();
            context.fill();
        }
        context.set_shadow_blur(0.0);

        // Draw the face
        if let Some(head) = head {
            let theta = interpolated_heading(previous_snake, snake, amount);
            let eye_distance = snake_size * 0.6;
            let left_eye = Coordinate {
                x: head.x + eye_distance as f32 * (theta - 35f64.to_radians()).cos() as f32,
                y: head.y + eye_distance as f32 * (theta - 35f64.to_radians()).sin() as f32,
            };
            let right_eye = Coordinate {
                x: head.x + eye_distance as f32 * (theta + 35f64.to_radians()).cos() as f32,
                y: head.y + eye_distance as f32 * (theta + 35f64.to_radians()).sin() as f32,
            };
            let gaze_direction =
                eye_gaze_direction(is_self, mouse_position, cursor_direction, theta);
            let pupil_offset = snake_size as f32 * 0.12;

            context.set_fill_style_str("#fff");
            context.begin_path();
            for eye in [left_eye, right_eye] {
                let radius = snake_size * 0.3;
                context.move_to(eye.x as f64 + radius, eye.y as f64);
                context
                    .arc(
                        eye.x as f64,
                        eye.y as f64,
                        radius,
                        0.,
                        std::f64::consts::PI * 2.,
                    )
                    .unwrap();
            }
            context.fill();

            context.set_fill_style_str("#000");
            context.begin_path();
            for eye in [left_eye, right_eye] {
                let pupil = Coordinate {
                    x: eye.x + gaze_direction.x * pupil_offset,
                    y: eye.y + gaze_direction.y * pupil_offset,
                };
                let radius = snake_size * 0.15;
                context.move_to(pupil.x as f64 + radius, pupil.y as f64);
                context
                    .arc(
                        pupil.x as f64,
                        pupil.y as f64,
                        radius,
                        0.,
                        std::f64::consts::PI * 2.,
                    )
                    .unwrap();
            }
            context.fill();
        }
    }
}

fn matching_previous_snake<'a>(
    previous_snakes: Option<&'a [Snake]>,
    snake_index: usize,
    current: &Snake,
) -> Option<&'a Snake> {
    previous_snakes
        .and_then(|previous| previous.get(snake_index))
        .filter(|previous| previous.color == current.color)
}

fn interpolated_snake_size(previous: Option<&Snake>, current: &Snake, amount: f32) -> f64 {
    let current_size = current.size as f64;
    let previous_size = previous
        .map(|snake| snake.size as f64)
        .unwrap_or(current_size);

    previous_size + (current_size - previous_size) * amount as f64
}

fn snake_glow_blur(snake: &Snake) -> f64 {
    if snake.acceleration_time_left == 0 {
        3.0
    } else {
        (snake.acceleration_time_left as f64 / 7.0).sin().abs() * 15.0
    }
}

fn interpolated_body(
    previous: Option<&Snake>,
    current: &Snake,
    body_index: usize,
    amount: f32,
) -> Coordinate {
    let current_body = current.bodies[body_index];
    let Some(previous_body) = previous
        .and_then(|snake| snake.bodies.get(body_index))
        .copied()
    else {
        return current_body;
    };

    // A large jump means that this body crossed a wrapped viewport edge or
    // the vector ordering changed. Interpolating that transition would sweep
    // a circle across the whole canvas.
    if (current_body.x - previous_body.x).abs() > 250.0
        || (current_body.y - previous_body.y).abs() > 250.0
    {
        return current_body;
    }

    Coordinate {
        x: previous_body.x + (current_body.x - previous_body.x) * amount,
        y: previous_body.y + (current_body.y - previous_body.y) * amount,
    }
}

fn interpolated_heading(previous: Option<&Snake>, current: &Snake, amount: f32) -> f64 {
    let previous_velocity = previous
        .map(|snake| snake.velocity)
        .unwrap_or(current.velocity);
    let x = previous_velocity.x + (current.velocity.x - previous_velocity.x) * amount;
    let y = previous_velocity.y + (current.velocity.y - previous_velocity.y) * amount;

    if x * x + y * y <= f32::EPSILON {
        current.velocity.y.atan2(current.velocity.x) as f64
    } else {
        y.atan2(x) as f64
    }
}

fn eye_gaze_direction(
    is_self: bool,
    mouse_position: &Coordinate,
    cursor_direction: Coordinate,
    heading: f64,
) -> Coordinate {
    if is_self && (mouse_position.x != 0.0 || mouse_position.y != 0.0) {
        cursor_direction
    } else {
        Coordinate {
            x: heading.cos() as f32,
            y: heading.sin() as f32,
        }
    }
}

fn update_minimap(minimap_context: &CanvasRenderingContext2d, map: &Map) {
    // NOTE: Based on the assumption that map is a 100*100 two-dimensional array

    minimap_context.clear_rect(0.0, 0.0, MINIMAP_SIZE, MINIMAP_SIZE);

    // Draw all map cells with a single browser API call. Calling fillRect for
    // every cell caused a visible frame drop whenever the minimap updated.
    let size = MINIMAP_SIZE as usize;
    let mut pixels = vec![0; size * size * 4];
    for x in 0..MINIMAP_SIZE as usize {
        for y in 0..MINIMAP_SIZE as usize {
            let index = (y * size + x) * 4;
            pixels[index] = 255;
            pixels[index + 1] = 255;
            pixels[index + 2] = 255;
            pixels[index + 3] = (map.map[x][y].min(10) * 25) as u8;
        }
    }
    let image_data = ImageData::new_with_u8_clamped_array_and_sh(
        Clamped(pixels.as_slice()),
        size as u32,
        size as u32,
    )
    .unwrap();
    minimap_context
        .put_image_data(&image_data, 0.0, 0.0)
        .unwrap();

    // Draw the coordinate axis
    minimap_context.set_stroke_style_str("#fff");
    minimap_context.set_line_width(0.5);
    minimap_context.begin_path();
    minimap_context.move_to(MINIMAP_SIZE / 2., 0.);
    minimap_context.line_to(MINIMAP_SIZE / 2., MINIMAP_SIZE);
    minimap_context.move_to(0., MINIMAP_SIZE / 2.);
    minimap_context.line_to(MINIMAP_SIZE, MINIMAP_SIZE / 2.);
    minimap_context.stroke();

    // Draw the self coordinate
    minimap_context.set_fill_style_str("green");
    minimap_context.begin_path();
    minimap_context
        .arc(
            map.self_coordinate.0 as f64,
            map.self_coordinate.1 as f64,
            3.,
            0.,
            std::f64::consts::PI * 2.,
        )
        .unwrap();
    minimap_context.fill();
}

fn render_minimap(context: &CanvasRenderingContext2d, minimap_context: &CanvasRenderingContext2d) {
    // Paste the sub canvas to the main canvas
    let responsive_size = (get_width() as f64 / 20.).clamp(70., 100.);
    let margin = (get_width() as f64 / 10.).clamp(20., 50.);
    let minimap_canvas = minimap_context.canvas().unwrap();

    context.set_shadow_blur(0.);
    context
        .draw_image_with_html_canvas_element_and_dw_and_dh(
            &minimap_canvas,
            get_width() as f64 - responsive_size - margin + GLOBAL_MARGIN,
            get_height() as f64 - responsive_size - margin + GLOBAL_MARGIN,
            responsive_size,
            responsive_size,
        )
        .unwrap();
}

fn render_background(context: &CanvasRenderingContext2d, offset: &Coordinate) {
    context.set_fill_style_str("#222");
    let width = (get_width() + 100) as f32;
    let height = (get_height() + 100) as f32;
    let mut x = offset.x;
    context.begin_path();
    while x <= width {
        let mut y = offset.y;
        while y <= height {
            context.move_to(x as f64 + 30.0, y as f64);
            context
                .arc(x as f64, y as f64, 30., 0.0, std::f64::consts::PI * 2.0)
                .unwrap();
            y += 100.0;
        }
        x += 100.0;
    }
    context.fill();
}

fn vector(a: &Coordinate, b: &Coordinate) -> Coordinate {
    //! Returns the normalized vector from a to b.
    let x = b.x - a.x;
    let y = b.y - a.y;
    let length = (x * x + y * y).sqrt();
    if length <= f32::EPSILON || !length.is_finite() {
        return Coordinate { x: 0.0, y: 0.0 };
    }
    Coordinate {
        x: x / length,
        y: y / length,
    }
}

mod pellet_rendering_helper {
    use super::Pellet;

    pub fn to_hsl(pellet: &Pellet) -> String {
        format!(
            "hsl({}, 100%, {}%)",
            pellet.color,
            (30. * (pellet.frame_count_offset as f64 / 7.).sin()).abs() + 50.
        )
    }

    pub fn to_radius(pellet: &Pellet) -> f64 {
        (pellet.size as f64 * 2.).min(pellet.frame_count_offset as f64)
    }
}

mod snake_rendering_helper {
    use super::Snake;

    pub fn to_hsl(snake: &Snake) -> String {
        format!("hsl({}, 100%, 40%)", snake.color)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_message() -> Message {
        Message {
            is_alive: true,
            snakes: Vec::new(),
            pellets: Vec::new(),
            background_offset: Coordinate::default(),
        }
    }

    #[test]
    fn wrapped_interpolation_takes_the_short_path() {
        assert!((lerp_wrapped(2.0, 98.0, 0.5, 100.0) - 0.0).abs() < f32::EPSILON);
        assert!((lerp_wrapped(98.0, 2.0, 0.5, 100.0) - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn wrapped_delta_handles_camera_boundary() {
        assert!((wrapped_delta(2.0, 98.0, 100.0) - 4.0).abs() < f32::EPSILON);
        assert!((wrapped_delta(98.0, 2.0, 100.0) + 4.0).abs() < f32::EPSILON);
    }

    #[test]
    fn jitter_buffer_selects_adjacent_game_frames() {
        let snapshots = (0..4)
            .map(|sequence| Snapshot {
                message: test_message(),
                opacity: 1.0,
                sequence,
            })
            .collect();

        let (previous, current, amount) = snapshot_pair(&snapshots, 1.5).unwrap();

        assert_eq!(previous.sequence, 1);
        assert_eq!(current.sequence, 2);
        assert!((amount - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn playback_slows_before_underrun_and_catches_up_after_a_burst() {
        assert_eq!(playback_rate(1.0), 0.8);
        assert_eq!(playback_rate(JITTER_BUFFER_FRAMES), 1.0);
        assert_eq!(playback_rate(8.0), 1.1);
    }

    #[test]
    fn snake_bodies_are_interpolated_between_snapshots() {
        let previous = Snake::new(Coordinate { x: 10.0, y: 20.0 }, 5.0);
        let mut current = previous.clone();
        current.bodies[0] = Coordinate { x: 20.0, y: 30.0 };

        let body = interpolated_body(Some(&previous), &current, 0, 0.5);

        assert_eq!(body, Coordinate { x: 15.0, y: 25.0 });
    }

    #[test]
    fn snake_bodies_remain_interpolated_when_length_changes() {
        let previous = Snake::new(Coordinate { x: 10.0, y: 20.0 }, 5.0);
        let mut current = previous.clone();
        current.bodies[0] = Coordinate { x: 20.0, y: 30.0 };
        current.bodies.push_back(Coordinate { x: 10.0, y: 20.0 });
        let previous_snakes = [previous];
        let previous_snake = matching_previous_snake(Some(&previous_snakes), 0, &current);

        let body = interpolated_body(previous_snake, &current, 0, 0.5);

        assert_eq!(body, Coordinate { x: 15.0, y: 25.0 });
    }

    #[test]
    fn snake_size_is_interpolated_between_snapshots() {
        let mut previous = Snake::new(Coordinate::default(), 5.0);
        previous.size = 15;
        let mut current = previous.clone();
        current.size = 17;

        let size = interpolated_snake_size(Some(&previous), &current, 0.5);

        assert!((size - 16.0).abs() < f64::EPSILON);
    }

    #[test]
    fn snake_has_a_soft_glow_at_normal_speed() {
        let snake = Snake::new(Coordinate::default(), 5.0);

        assert!((snake_glow_blur(&snake) - 3.0).abs() < f64::EPSILON);
    }

    #[test]
    fn snake_glow_pulses_while_accelerating() {
        let mut snake = Snake::new(Coordinate::default(), 5.0);
        snake.acceleration_time_left = 11;

        let expected = (11.0_f64 / 7.0).sin().abs() * 15.0;

        assert!((snake_glow_blur(&snake) - expected).abs() < f64::EPSILON);
    }

    #[test]
    fn snake_heading_is_interpolated_between_snapshots() {
        let mut previous = Snake::new(Coordinate::default(), 5.0);
        previous.velocity = Coordinate { x: 1.0, y: 0.0 };
        let mut current = previous.clone();
        current.velocity = Coordinate { x: 0.0, y: 1.0 };

        let heading = interpolated_heading(Some(&previous), &current, 0.5);

        assert!((heading - std::f64::consts::FRAC_PI_4).abs() < 1e-6);
    }

    #[test]
    fn player_pupils_use_the_live_cursor_direction() {
        let mouse_position = Coordinate { x: 100.0, y: 20.0 };
        let cursor_direction = Coordinate { x: 0.0, y: -1.0 };

        let gaze = eye_gaze_direction(true, &mouse_position, cursor_direction, 0.0);

        assert_eq!(gaze, cursor_direction);
    }

    #[test]
    fn remote_pupils_follow_their_snake_heading() {
        let gaze = eye_gaze_direction(
            false,
            &Coordinate { x: 100.0, y: 20.0 },
            Coordinate { x: 0.0, y: -1.0 },
            0.0,
        );

        assert_eq!(gaze, Coordinate { x: 1.0, y: 0.0 });
    }
}
