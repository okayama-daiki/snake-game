use wasm_bindgen::prelude::{wasm_bindgen, JsValue};

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
    pub fn render(&self) {}
}
