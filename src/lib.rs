mod types;
use types::{Coordinate, Message, Pellet, Snake};
#[macro_use]
mod browser;

#[allow(unused_imports)]
use anyhow::{anyhow, Result};
#[allow(unused_imports)]
use browser::{
    create_mouse_position_getter, get_center_coordinate, get_context, get_height, get_width, window,
};

#[allow(unused_imports)]
use wasm_bindgen::{
    prelude::{wasm_bindgen, Closure, JsValue},
    JsCast, UnwrapThrowExt,
};
#[allow(unused_imports)]
use web_sys::{
    console, js_sys::Function, CanvasRenderingContext2d, HtmlCanvasElement, MessageEvent,
    MouseEvent, WebSocket,
};

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

        let mut context = get_context(&self.canvas);

        let get_mouse_position = create_mouse_position_getter();

        // 3. Add a message handler to the websocket, so that render is called when a message is received.
        let socket = self.socket.clone();
        let on_message = Closure::wrap(Box::new(move |e: web_sys::MessageEvent| {
            let message: Message = serde_json::from_str(&e.data().as_string().unwrap()).unwrap();
            render(&mut context, message);

            let direction = vector(&get_center_coordinate(), &get_mouse_position());
            socket
                .send_with_str(format!("v {} {}", direction.x, direction.y).as_str())
                .ok();
        }) as Box<dyn FnMut(web_sys::MessageEvent)>);
        self.socket
            .set_onmessage(Some(on_message.as_ref().unchecked_ref()));
        on_message.forget();

        // 4. Finally, send a start message to the server, and start the game.
        self.socket
            .send_with_str("s")
            .map_err(|e| {
                console::log_1(&e);
                e
            })
            .ok();
        self.socket
            .send_with_str(format!("w {} {}", self.canvas.width(), self.canvas.height()).as_str())
            .ok();
    }
}

fn render(context: &mut CanvasRenderingContext2d, message: Message) {
    context.clear_rect(0.0, 0.0, get_width() as f64, get_height() as f64);
    render_pellets(context, &message.pellets);
    render_snakes(context, &message.snakes);
}

fn render_snakes(context: &mut CanvasRenderingContext2d, snakes: &Vec<Snake>) {
    for snake in snakes {
        context.set_fill_style(&JsValue::from_str(&snake.color));
        context.set_shadow_color("rgb(0, 100, 0)");
        context.set_shadow_blur(3.);
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
    }
}

fn render_pellets(context: &mut CanvasRenderingContext2d, pellets: &Vec<Pellet>) {
    console::log_1(&JsValue::from_str("Rendering pellets"));
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
