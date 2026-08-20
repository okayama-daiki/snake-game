use game::{coordinate::Coordinate, map::Map, pellet::Pellet, snake::Snake, view::View as Message};

#[macro_use]
mod browser;
use browser::{
    canvas, create_mouse_position_tracker, get_center_coordinate, get_context, get_height,
    get_width, now, window,
};
use std::rc::Rc;
use std::{cell::Cell, collections::VecDeque};
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
    on_resize: Option<Closure<dyn FnMut()>>,
    on_message: Option<Closure<dyn FnMut(MessageEvent)>>,
    on_mouse_move: Option<Closure<dyn FnMut(MouseEvent)>>,
    on_mouse_down: Option<Closure<dyn FnMut()>>,
    on_mouse_up: Option<Closure<dyn FnMut()>>,
    interval_callback: Option<Closure<dyn FnMut()>>,
    interval_id: Option<i32>,
}

#[wasm_bindgen]
impl RenderEngine {
    #[wasm_bindgen(constructor)]
    pub fn new(canvas: HtmlCanvasElement, socket: WebSocket, callback: Function) -> Self {
        Self {
            canvas,
            socket,
            callback,
            on_resize: None,
            on_message: None,
            on_mouse_move: None,
            on_mouse_down: None,
            on_mouse_up: None,
            interval_callback: None,
            interval_id: None,
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

        // 3. Add a message handler to the websocket so that render is called when a message is received.
        {
            let mut frame_count = 0;
            let mut process_time: VecDeque<f64> = VecDeque::with_capacity(PERFORMANCE_SAMPLE_COUNT);
            let mut fps_log: VecDeque<f64> = VecDeque::with_capacity(PERFORMANCE_SAMPLE_COUNT);
            let mut last_frame_time = now().unwrap();

            let is_alive = Cell::new(true);
            let frame_after_death = Cell::new(0);
            let socket = self.socket.clone();
            let context = get_context(&self.canvas);

            let minimap_canvas = canvas().unwrap();
            minimap_canvas.set_height(MINIMAP_SIZE as u32);
            minimap_canvas.set_width(MINIMAP_SIZE as u32);
            let minimap_context = get_context(&minimap_canvas);

            let callback = self.callback.clone();
            let mouse_tracker = create_mouse_position_tracker();
            window()
                .unwrap()
                .set_onmousemove(Some(mouse_tracker.handler.as_ref().unchecked_ref()));
            let mouse_position = mouse_tracker.position;
            self.on_mouse_move = Some(mouse_tracker.handler);
            let on_message = Closure::wrap(Box::new(move |e: MessageEvent| {
                // 3.0. Calculate the FPS
                let start = now().unwrap();
                let frame_duration = now().unwrap() - last_frame_time;
                last_frame_time = now().unwrap();
                if fps_log.len() == PERFORMANCE_SAMPLE_COUNT {
                    fps_log.pop_front();
                }
                fps_log.push_back(frame_duration);
                if frame_count > 0 && frame_count % PERFORMANCE_SAMPLE_COUNT == 0 {
                    log!(
                        "FPS: {}",
                        1000. / (fps_log.iter().sum::<f64>() / fps_log.len() as f64)
                    );
                    log!(
                        "FPS variance: {}",
                        fps_log
                            .iter()
                            .map(|x| (1000. / x - 60.).powi(2))
                            .sum::<f64>()
                            / fps_log.len() as f64
                    );
                    log!(
                        "Process time: {}",
                        process_time.iter().sum::<f64>() / process_time.len() as f64
                    );
                }

                // 3.1. Parse the message into a Message struct and render the Message.
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
                    context.set_global_alpha(
                        1. - (frame_after_death.get() as f64 - 50.).max(0.) / 100.,
                    );
                    render(&context, &minimap_context, &message);
                }

                if let Ok(map) = Map::from_bytes(&vec) {
                    update_minimap(&minimap_context, &map);
                }

                // 3.2. Send the mouse position to the server. (To be more precise, send normalized vector from center to mouse position)
                if is_alive.get() {
                    let dir = vector(&get_center_coordinate(), &mouse_position.get());
                    socket
                        .send_with_str(format!("v {} {}", dir.x, dir.y).as_str())
                        .ok();
                }
                frame_count += 1;
                if process_time.len() == PERFORMANCE_SAMPLE_COUNT {
                    process_time.pop_front();
                }
                process_time.push_back(now().unwrap() - start);
            }) as Box<dyn FnMut(MessageEvent)>);
            self.socket
                .set_onmessage(Some(on_message.as_ref().unchecked_ref()));
            self.on_message = Some(on_message);
        }

        // 4. Add a mousedown handler to the window so that the snake can accelerate when the window is clicked.
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

        // 5. Finally, send a start message to the server, and start the game.
        self.socket.send_with_str("s").ok();
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
        }

        self.on_resize = None;
        self.on_message = None;
        self.on_mouse_move = None;
        self.on_mouse_down = None;
        self.on_mouse_up = None;
        self.interval_callback = None;
    }
}

fn render(
    context: &CanvasRenderingContext2d,
    minimap_context: &CanvasRenderingContext2d,
    message: &Message,
) {
    context.clear_rect(
        0.0,
        0.0,
        (get_width() + 100) as f64,
        (get_height() + 100) as f64,
    );
    render_background(context, &message.background_offset);
    render_pellets(context, &message.pellets);
    render_snakes(context, &message.snakes);
    render_minimap(context, minimap_context);
}

fn render_pellets(context: &CanvasRenderingContext2d, pellets: &Vec<Pellet>) {
    for pellet in pellets {
        let hsl = pellet_rendering_helper::to_hsl(pellet);
        context.set_fill_style_str(hsl.as_str());
        context.set_shadow_color(hsl.as_str());
        context.set_shadow_blur((pellet.size as f64) * 10.);
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

fn render_snakes(context: &CanvasRenderingContext2d, snakes: &Vec<Snake>) {
    for snake in snakes {
        // Draw the body

        let hsl = snake_rendering_helper::to_hsl(snake);

        for body in snake.bodies.iter().rev() {
            context.set_fill_style_str("rgba(0, 0, 0, 0.3)");
            context.set_shadow_color("rgba(0, 0, 0, 0.3)");
            context.set_shadow_blur(10.);
            context.begin_path();
            context
                .arc(
                    body.x as f64,
                    body.y as f64,
                    snake.size as f64,
                    0.0,
                    std::f64::consts::PI * 2.0,
                )
                .unwrap();
            context.fill();
            context.set_fill_style_str(hsl.as_str());
            context.set_shadow_color(hsl.as_str());
            context.set_shadow_blur(if snake.acceleration_time_left == 0 {
                3.
            } else {
                (snake.acceleration_time_left as f64 / 7.).sin().abs() * 15.
            });
            context.begin_path();
            context
                .arc(
                    body.x as f64,
                    body.y as f64,
                    snake.size as f64,
                    0.0,
                    std::f64::consts::PI * 2.0,
                )
                .unwrap();
            context.fill();
        }

        // Draw the face
        if snake.is_visible_head {
            let head = snake.bodies.front().unwrap();
            let theta = snake.velocity.y.atan2(snake.velocity.x) as f64;
            context.restore();
            context.set_fill_style_str("#fff");
            context.begin_path();
            context
                .arc(
                    head.x as f64 + (snake.size as f64) * 0.6 * (theta - 35f64.to_radians()).cos(),
                    head.y as f64 + (snake.size as f64) * 0.6 * (theta - 35f64.to_radians()).sin(),
                    snake.size as f64 * 0.3,
                    0.,
                    std::f64::consts::PI * 2.,
                )
                .unwrap();
            context.fill();
            context.begin_path();
            context
                .arc(
                    head.x as f64 + (snake.size as f64) * 0.6 * (theta + 35f64.to_radians()).cos(),
                    head.y as f64 + (snake.size as f64) * 0.6 * (theta + 35f64.to_radians()).sin(),
                    snake.size as f64 * 0.3,
                    0.,
                    std::f64::consts::PI * 2.,
                )
                .unwrap();
            context.fill();
            context.set_fill_style_str("#000");
            context.begin_path();
            context
                .arc(
                    head.x as f64 + (snake.size as f64) * 0.6 * (theta - 35f64.to_radians()).cos(),
                    head.y as f64 + (snake.size as f64) * 0.6 * (theta - 35f64.to_radians()).sin(),
                    snake.size as f64 * 0.15,
                    0.,
                    std::f64::consts::PI * 2.,
                )
                .unwrap();
            context.fill();
            context.begin_path();
            context
                .arc(
                    head.x as f64 + (snake.size as f64) * 0.6 * (theta + 35f64.to_radians()).cos(),
                    head.y as f64 + (snake.size as f64) * 0.6 * (theta + 35f64.to_radians()).sin(),
                    snake.size as f64 * 0.15,
                    0.,
                    std::f64::consts::PI * 2.,
                )
                .unwrap();
            context.fill();
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
    while x <= width {
        let mut y = offset.y;
        while y <= height {
            context.begin_path();
            context
                .arc(x as f64, y as f64, 30., 0.0, std::f64::consts::PI * 2.0)
                .unwrap();
            context.fill();
            y += 100.0;
        }
        x += 100.0;
    }
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
