use serde::{Deserialize, Serialize};
use wasm_bindgen::{
    prelude::{wasm_bindgen, Closure, JsValue},
    JsCast,
};

#[derive(Serialize, Deserialize)]
struct Coordinate {
    x: f64,
    y: f64,
}

#[derive(Serialize, Deserialize)]
struct Pellet {
    position: Coordinate,
    size: u8,
    color: String,
    frame_count_offset: u32,
}

#[derive(Serialize, Deserialize)]
struct Snake {
    bodies: Vec<Coordinate>,
    acceleration_time_left: u32,
    speed: f32,
    color: String,
    velocity: Coordinate,
    size: u32,
    frame_count_offset: u32,
}

#[derive(Serialize, Deserialize)]
struct Message {
    is_alive: bool,
    snakes: Vec<Snake>,
    pellets: Vec<Pellet>,
    map: Vec<Vec<u8>>,
    self_coordinate: [usize; 2],
}

#[wasm_bindgen]
extern "C" {}

#[wasm_bindgen]
pub struct RenderEngine {
    canvas: web_sys::HtmlCanvasElement,
    context: web_sys::CanvasRenderingContext2d,
    socket: web_sys::WebSocket,
    callback: web_sys::js_sys::Function,
}

#[wasm_bindgen]
impl RenderEngine {
    #[wasm_bindgen(constructor)]
    pub fn new(
        canvas: web_sys::HtmlCanvasElement,
        context: web_sys::CanvasRenderingContext2d,
        socket: web_sys::WebSocket,
        callback: web_sys::js_sys::Function,
    ) -> Self {
        Self {
            canvas,
            context,
            socket,
            callback,
        }
    }

    pub fn init(&mut self) {
        console_error_panic_hook::set_once();

        self.canvas.set_height(
            web_sys::window()
                .unwrap()
                .inner_height()
                .unwrap()
                .as_f64()
                .unwrap() as u32,
        );
        self.canvas.set_width(
            web_sys::window()
                .unwrap()
                .inner_width()
                .unwrap()
                .as_f64()
                .unwrap() as u32,
        );

        self.socket.send_with_str("s").ok();
        self.socket
            .send_with_str(format!("w {} {}", self.canvas.width(), self.canvas.height()).as_str())
            .ok();

        let context = self.context.clone();

        // See https://github.com/rustwasm/wasm-bindgen/issues/89
        let on_message = Closure::wrap(Box::new(move |e: web_sys::MessageEvent| {
            let data = e.data();
            let message: Message = match serde_json::from_str(&data.as_string().unwrap()) {
                Ok(message) => message,
                Err(e) => {
                    web_sys::console::log_1(&JsValue::from_str(e.to_string().as_str()));
                    return;
                }
            };
            web_sys::console::log_1(&JsValue::from_str(
                message.pellets.len().to_string().as_str(),
            ));
            for snake in message.snakes {
                context.set_fill_style(&JsValue::from_str(&snake.color));
                for body in snake.bodies {
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
        }) as Box<dyn FnMut(web_sys::MessageEvent)>);
        self.socket
            .set_onmessage(Some(on_message.as_ref().unchecked_ref()));
        on_message.forget();
    }
}
