mod types;
use types::{Coordinate, Map, Message, Pellet, Snake};
#[macro_use]
mod browser;
use browser::{
    canvas, create_mouse_position_getter, get_center_coordinate, get_context, get_height,
    get_width, window,
};
use std::cell::Cell;
use std::rc::Rc;
use wasm_bindgen::{
    prelude::{wasm_bindgen, Closure, JsValue},
    JsCast,
};
use web_sys::{js_sys::Function, CanvasRenderingContext2d, HtmlCanvasElement, WebSocket};

static GLOBAL_MARGIN: f64 = 50.;

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
}

#[wasm_bindgen]
impl RenderEngine {
    #[wasm_bindgen(constructor)]
    pub fn new(canvas: HtmlCanvasElement, socket: WebSocket, callback: Function) -> Self {
        Self {
            canvas,
            socket,
            callback,
        }
    }

    pub fn init(&mut self) {
        console_error_panic_hook::set_once();

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
            on_resize.forget();
        }

        // 3. Add a message handler to the websocket so that render is called when a message is received.
        {
            let is_alive = Cell::new(true);
            let frame_after_death = Cell::new(0);
            let socket = self.socket.clone();
            let context = get_context(&self.canvas);
            let callback = self.callback.clone();
            let get_mouse_position = create_mouse_position_getter();
            let on_message = Closure::wrap(Box::new(move |e: web_sys::MessageEvent| {
                // 3.1. Parse the message into a Message struct and render the Message.
                let message: Message =
                    serde_json::from_str(&e.data().as_string().unwrap()).unwrap();

                // 3.2. if the snake is dead, gradually darken the screen and call the callback function when the screen is completely dark.
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
                context
                    .set_global_alpha(1. - (frame_after_death.get() as f64 - 50.).max(0.) / 100.);
                render(&context, &message);

                // 3.3. Send the mouse position to the server. (To be more precise, send normalized vector from center to mouse position)
                if is_alive.get() {
                    let dir = vector(&get_center_coordinate(), &get_mouse_position());
                    socket
                        .send_with_str(format!("v {} {}", dir.x, dir.y).as_str())
                        .ok();
                }
            }) as Box<dyn FnMut(web_sys::MessageEvent)>);
            self.socket
                .set_onmessage(Some(on_message.as_ref().unchecked_ref()));
            on_message.forget();
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
            window()
                .unwrap()
                .set_interval_with_callback_and_timeout_and_arguments_0(
                    interval_callback.as_ref().unchecked_ref(),
                    100,
                )
                .unwrap();
            interval_callback.forget();
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
            on_mousedown.forget();
            on_mouseup.forget();
        }

        // 5. Finally, send a start message to the server, and start the game.
        self.socket.send_with_str("s").ok();
        self.socket
            .send_with_str(format!("w {} {}", self.canvas.width(), self.canvas.height()).as_str())
            .ok();
    }
}

fn render(context: &CanvasRenderingContext2d, message: &Message) {
    context.clear_rect(
        0.0,
        0.0,
        (get_width() + 100) as f64,
        (get_height() + 100) as f64,
    );
    render_background(context, &message.background_dots);
    render_pellets(context, &message.pellets);
    render_snakes(context, &message.snakes);
    render_map(context, &message.map);
}

fn render_pellets(context: &CanvasRenderingContext2d, pellets: &Vec<Pellet>) {
    for pellet in pellets {
        context.set_fill_style(&JsValue::from_str(&pellet.hsl()));
        context.set_shadow_color(pellet.hsl().as_str());
        context.set_shadow_blur(pellet.size * 10.);
        context.begin_path();
        context
            .arc(
                pellet.position.x,
                pellet.position.y,
                pellet.radius(),
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
        context.set_fill_style(&JsValue::from_str(&snake.color));
        context.set_shadow_color(if snake.acceleration_time_left == 0 {
            "rgb(0, 100, 0)"
        } else {
            snake.color.as_str()
        });
        context.set_shadow_blur(if snake.acceleration_time_left == 0 {
            3.
        } else {
            (snake.acceleration_time_left as f64 / 7.).sin().abs() * 15.
        });
        for body in snake.bodies.iter().rev() {
            context.begin_path();
            context
                .arc(
                    body.x,
                    body.y,
                    snake.size as f64,
                    0.0,
                    std::f64::consts::PI * 2.0,
                )
                .unwrap();
            context.fill();
        }

        // Draw the face
        if snake.is_visible_head {
            let head = snake.bodies.first().unwrap();
            let theta = snake.velocity.y.atan2(snake.velocity.x);
            context.restore();
            context.set_fill_style(&JsValue::from_str("#fff"));
            context.begin_path();
            context
                .arc(
                    head.x + (snake.size as f64) * 0.6 * (theta - 35f64.to_radians()).cos(),
                    head.y + (snake.size as f64) * 0.6 * (theta - 35f64.to_radians()).sin(),
                    snake.size as f64 * 0.3,
                    0.,
                    std::f64::consts::PI * 2.,
                )
                .unwrap();
            context.fill();
            context.begin_path();
            context
                .arc(
                    head.x + (snake.size as f64) * 0.6 * (theta + 35f64.to_radians()).cos(),
                    head.y + (snake.size as f64) * 0.6 * (theta + 35f64.to_radians()).sin(),
                    snake.size as f64 * 0.3,
                    0.,
                    std::f64::consts::PI * 2.,
                )
                .unwrap();
            context.fill();
            context.set_fill_style(&JsValue::from_str("#000"));
            context.begin_path();
            context
                .arc(
                    head.x + (snake.size as f64) * 0.6 * (theta - 35f64.to_radians()).cos(),
                    head.y + (snake.size as f64) * 0.6 * (theta - 35f64.to_radians()).sin(),
                    snake.size as f64 * 0.15,
                    0.,
                    std::f64::consts::PI * 2.,
                )
                .unwrap();
            context.fill();
            context.begin_path();
            context
                .arc(
                    head.x + (snake.size as f64) * 0.6 * (theta + 35f64.to_radians()).cos(),
                    head.y + (snake.size as f64) * 0.6 * (theta + 35f64.to_radians()).sin(),
                    snake.size as f64 * 0.15,
                    0.,
                    std::f64::consts::PI * 2.,
                )
                .unwrap();
            context.fill();
        }
    }
}

fn render_map(context: &CanvasRenderingContext2d, map: &Map) {
    // NOTE: Based on the assumption that map is a 100*100 two-dimensional array
    const MAP_SIZE: u32 = 100;

    let sub_canvas = canvas().unwrap();
    sub_canvas.set_height(MAP_SIZE);
    sub_canvas.set_width(MAP_SIZE);
    let sub_context = get_context(&sub_canvas);

    // Draw the map
    for x in 0..MAP_SIZE as usize {
        for y in 0..MAP_SIZE as usize {
            sub_context.begin_path();
            sub_context.set_fill_style(&JsValue::from_str(
                format!("rgba(255, 255, 255, {})", map.map[x][y] as f32 / 10.).as_str(),
            ));
            sub_context.fill_rect(x as f64, y as f64, 1., 1.);
        }
    }

    // Draw the coordinate axis
    sub_context.set_stroke_style(&JsValue::from_str("#fff"));
    sub_context.set_line_width(0.5);
    sub_context.begin_path();
    sub_context.move_to(MAP_SIZE as f64 / 2., 0.);
    sub_context.line_to(MAP_SIZE as f64 / 2., MAP_SIZE as f64);
    sub_context.move_to(0., MAP_SIZE as f64 / 2.);
    sub_context.line_to(MAP_SIZE as f64, MAP_SIZE as f64 / 2.);
    sub_context.stroke();

    // Draw the self coordinate
    sub_context.set_fill_style(&JsValue::from_str("green"));
    sub_context.begin_path();
    sub_context
        .arc(
            map.self_coordinate[0] as f64,
            map.self_coordinate[1] as f64,
            3.,
            0.,
            std::f64::consts::PI * 2.,
        )
        .unwrap();
    sub_context.fill();

    // Paste the sub canvas to the main canvas
    let responsive_size = (get_width() as f64 / 20.).clamp(70., 100.);
    let margin = (get_width() as f64 / 10.).clamp(20., 50.);

    context.set_shadow_blur(0.);
    context
        .draw_image_with_html_canvas_element_and_dw_and_dh(
            &sub_canvas,
            get_width() as f64 - responsive_size - margin + GLOBAL_MARGIN,
            get_height() as f64 - responsive_size - margin + GLOBAL_MARGIN,
            responsive_size,
            responsive_size,
        )
        .unwrap();
}

fn render_background(context: &CanvasRenderingContext2d, background_dots: &Vec<Coordinate>) {
    context.set_fill_style(&JsValue::from_str("#222"));
    for dot in background_dots {
        context.begin_path();
        context
            .arc(dot.x, dot.y, 30., 0.0, std::f64::consts::PI * 2.0)
            .unwrap();
        context.fill();
    }
}

fn vector(a: &Coordinate, b: &Coordinate) -> Coordinate {
    //! Returns the normalized vector from a to b.
    let x = b.x - a.x;
    let y = b.y - a.y;
    let length = (x * x + y * y).sqrt();
    Coordinate {
        x: x / length,
        y: y / length,
    }
}
