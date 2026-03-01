# WebAssembly (WASM) in Rust

## Overview

Rust is one of the best languages for compiling to WebAssembly (WASM), offering near-native performance in the browser and other WASM environments. This guide covers WASM development, tooling, and integration patterns.

---

## WebAssembly Fundamentals

### What is WebAssembly?

WebAssembly is a binary instruction format that runs in web browsers and other WASM runtimes. It provides:

- **Near-native performance** - Close to native execution speed
- **Safety** - Sandboxed execution environment
- **Portability** - Runs on multiple platforms
- **Language agnostic** - Can be compiled from many languages

### Why Rust for WASM?

- **Zero-cost abstractions** - No runtime overhead
- **Memory safety** - Prevents common vulnerabilities
- **Small binary size** - Efficient WASM output
- **Tooling support** - Excellent WASM ecosystem
- **Type safety** - Compile-time guarantees

---

## WASM Build Configuration

### Cargo.toml Setup

```toml
[package]
name = "wasm-project"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
wasm-bindgen = "0.2"
js-sys = "0.3"
web-sys = "0.3"
wasm-bindgen-futures = "0.4"

[dependencies.web-sys]
version = "0.3"
features = [
  "console",
  "Document",
  "Element",
  "HtmlElement",
  "Window",
  "Performance",
]
```

### Build Scripts

```toml
[package.metadata.wasm-pack.profile.release]
wasm-opt = ['-O']

[package.metadata.wasm-pack.profile.dev]
wasm-opt = []
```

---

## Basic WASM Module

### Simple WASM Library

```rust
use wasm_bindgen::prelude::*;

// Export a function to JavaScript
#[wasm_bindgen]
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

// Export a struct
#[wasm_bindgen]
pub struct Calculator {
    value: f64,
}

#[wasm_bindgen]
impl Calculator {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Calculator {
        Calculator { value: 0.0 }
    }
    
    #[wasm_bindgen(getter)]
    pub fn value(&self) -> f64 {
        self.value
    }
    
    #[wasm_bindgen(setter)]
    pub fn set_value(&mut self, value: f64) {
        self.value = value;
    }
    
    #[wasm_bindgen]
    pub fn add(&mut self, other: f64) {
        self.value += other;
    }
    
    #[wasm_bindgen]
    pub fn multiply(&mut self, other: f64) {
        self.value *= other;
    }
}
```

### String Handling

```rust
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn greet(name: &str) -> String {
    format!("Hello, {}!", name)
}

#[wasm_bindgen]
pub fn process_text(text: &str) -> String {
    text.chars()
        .map(|c| match c {
            'a'..='z' => c.to_uppercase().to_string(),
            'A'..='Z' => c.to_lowercase().to_string(),
            _ => c.to_string(),
        })
        .collect()
}
```

---

## DOM Manipulation

### Working with Web APIs

```rust
use wasm_bindgen::prelude::*;
use web_sys::{console, Document, Element, Window};

#[wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();
    
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    
    // Create a new element
    let div = document.create_element("div").unwrap();
    div.set_text_content(Some("Hello from Rust!"));
    div.set_class_name("rust-generated");
    
    // Add to document body
    let body = document.body().unwrap();
    body.append_child(&div).unwrap();
    
    // Log to console
    console::log_1(&"WASM module loaded successfully!".into());
}

#[wasm_bindgen]
pub fn create_button(text: &str) -> Result<Element, JsValue> {
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    
    let button = document.create_element("button")?;
    button.set_text_content(Some(text));
    button.set_attribute("onclick", "alert('Button clicked!')")?;
    
    Ok(button)
}
```

### Event Handling

```rust
use wasm_bindgen::prelude::*;
use web_sys::{console, Event, MouseEvent};

#[wasm_bindgen]
pub struct ClickCounter {
    count: u32,
    element: web_sys::Element,
}

#[wasm_bindgen]
impl ClickCounter {
    #[wasm_bindgen(constructor)]
    pub fn new(element_id: &str) -> Result<ClickCounter, JsValue> {
        let window = web_sys::window().unwrap();
        let document = window.document().unwrap();
        
        let element = document.get_element_by_id(element_id)
            .ok_or_else(|| JsValue::from_str("Element not found"))?;
        
        let counter = ClickCounter { count: 0, element };
        
        // Set up event listener
        let closure = Closure::wrap(Box::new(move |event: MouseEvent| {
            console::log_1(&format!("Mouse clicked at: ({}, {})", 
                event.client_x(), event.client_y()).into());
        }) as Box<dyn Fn(MouseEvent)>);
        
        element.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())?;
        closure.forget(); // Keep closure alive
        
        Ok(counter)
    }
    
    #[wasm_bindgen]
    pub fn increment(&mut self) {
        self.count += 1;
        self.element.set_text_content(Some(&format!("Count: {}", self.count)));
    }
    
    #[wasm_bindgen]
    pub fn reset(&mut self) {
        self.count = 0;
        self.element.set_text_content(Some("Count: 0"));
    }
}
```

---

## Performance Optimization

### WASM-Specific Optimizations

```rust
use wasm_bindgen::prelude::*;
use js_sys::Uint8Array;
use web_sys::Performance;

#[wasm_bindgen]
pub fn optimized_array_sum(data: &[u8]) -> u32 {
    // Use iterators for better optimization
    data.iter().map(|&x| x as u32).sum()
}

#[wasm_bindgen]
pub fn process_large_array(data: &[u8]) -> Vec<u8> {
    // Pre-allocate result vector
    let mut result = Vec::with_capacity(data.len());
    
    // Process in chunks for better cache locality
    const CHUNK_SIZE: usize = 1024;
    
    for chunk in data.chunks(CHUNK_SIZE) {
        for &byte in chunk {
            // Some processing logic
            result.push(byte.wrapping_add(1));
        }
    }
    
    result
}

#[wasm_bindgen]
pub fn benchmark_function(iterations: u32) -> f64 {
    let window = web_sys::window().unwrap();
    let performance = window.performance().unwrap();
    
    let start = performance.now();
    
    // Perform computation
    let mut result = 0.0;
    for i in 0..iterations {
        result += (i as f64).sin();
    }
    
    let end = performance.now();
    
    console::log_1(&format!("Computation took: {}ms", end - start).into());
    
    result
}
```

### Memory Management

```rust
use wasm_bindgen::prelude::*;
use js_sys::ArrayBuffer;

#[wasm_bindgen]
pub struct WasmBuffer {
    buffer: Vec<u8>,
    js_buffer: ArrayBuffer,
}

#[wasm_bindgen]
impl WasmBuffer {
    #[wasm_bindgen(constructor)]
    pub fn new(size: usize) -> WasmBuffer {
        let buffer = vec![0u8; size];
        let js_buffer = unsafe {
            ArrayBuffer::view(&buffer).into()
        };
        
        WasmBuffer { buffer, js_buffer }
    }
    
    #[wasm_bindgen(getter)]
    pub fn buffer(&self) -> ArrayBuffer {
        self.js_buffer.clone()
    }
    
    #[wasm_bindgen]
    pub fn fill(&mut self, value: u8) {
        for byte in &mut self.buffer {
            *byte = value;
        }
    }
    
    #[wasm_bindgen]
    pub fn get(&self, index: usize) -> u8 {
        self.buffer[index]
    }
    
    #[wasm_bindgen]
    pub fn set(&mut self, index: usize, value: u8) {
        self.buffer[index] = value;
    }
}
```

---

## Async Operations

### Promises and Futures

```rust
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::{Request, RequestInit, RequestMode, Response};

#[wasm_bindgen]
pub async fn fetch_data(url: &str) -> Result<String, JsValue> {
    let mut opts = RequestInit::new();
    opts.method("GET");
    opts.mode(RequestMode::Cors);
    
    let request = Request::new_with_str_and_init(url, &opts)?;
    
    let window = web_sys::window().unwrap();
    let resp_value = JsFuture::from(window.fetch_with_request(&request)).await?;
    
    let resp: Response = resp_value.dyn_into()?;
    let text = JsFuture::from(resp.text()?).await?;
    
    Ok(text.as_string().unwrap_or_default())
}

#[wasm_bindgen]
pub async fn process_async_data() -> Result<Vec<String>, JsValue> {
    let urls = vec![
        "https://api.example.com/data1",
        "https://api.example.com/data2",
        "https://api.example.com/data3",
    ];
    
    let mut results = Vec::new();
    
    for url in urls {
        let data = fetch_data(url).await?;
        results.push(data);
    }
    
    Ok(results)
}
```

### Web Workers

```rust
use wasm_bindgen::prelude::*;
use web_sys::{DedicatedWorkerGlobalScope, MessageEvent, Worker};

#[wasm_bindgen]
pub fn create_worker(script_url: &str) -> Result<Worker, JsValue> {
    Worker::new(script_url)
}

#[wasm_bindgen]
pub fn worker_main() {
    // This function would be called from the worker script
    let scope = DedicatedWorkerGlobalScope::from(JsValue::from(-1)); // Worker global
    
    let closure = Closure::wrap(Box::new(move |event: MessageEvent| {
        let data = event.data();
        console::log_1(&format!("Worker received: {:?}", data).into());
        
        // Process data and send back
        let result = format!("Processed: {:?}", data);
        scope.post_message(&result.into());
    }) as Box<dyn Fn(MessageEvent)>);
    
    scope.set_onmessage(Some(closure.as_ref().unchecked_ref()));
    closure.forget();
}
```

---

## Graphics and Canvas

### Canvas Manipulation

```rust
use wasm_bindgen::prelude::*;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};

#[wasm_bindgen]
pub struct CanvasRenderer {
    ctx: CanvasRenderingContext2d,
    width: u32,
    height: u32,
}

#[wasm_bindgen]
impl CanvasRenderer {
    #[wasm_bindgen(constructor)]
    pub fn new(canvas_id: &str) -> Result<CanvasRenderer, JsValue> {
        let window = web_sys::window().unwrap();
        let document = window.document().unwrap();
        
        let canvas = document
            .get_element_by_id(canvas_id)
            .ok_or_else(|| JsValue::from_str("Canvas not found"))?
            .dyn_into::<HtmlCanvasElement>()
            .map_err(|_| JsValue::from_str("Element is not a canvas"))?;
        
        let ctx = canvas
            .get_context("2d")?
            .dyn_into::<CanvasRenderingContext2d>()
            .map_err(|_| JsValue::from_str("Failed to get 2d context"))?;
        
        let width = canvas.width();
        let height = canvas.height();
        
        Ok(CanvasRenderer { ctx, width, height })
    }
    
    #[wasm_bindgen]
    pub fn clear(&self) {
        self.ctx.clear_rect(0.0, 0.0, self.width as f64, self.height as f64);
    }
    
    #[wasm_bindgen]
    pub fn draw_rect(&self, x: f64, y: f64, width: f64, height: f64, color: &str) {
        self.ctx.set_fill_style(color);
        self.ctx.fill_rect(x, y, width, height);
    }
    
    #[wasm_bindgen]
    pub fn draw_circle(&self, x: f64, y: f64, radius: f64, color: &str) {
        self.ctx.begin_path();
        self.ctx.arc(x, y, radius, 0.0, 2.0 * std::f64::consts::PI);
        self.ctx.set_fill_style(color);
        self.ctx.fill();
    }
    
    #[wasm_bindgen]
    pub fn draw_text(&self, text: &str, x: f64, y: f64, font: &str, color: &str) {
        self.ctx.set_font(font);
        self.ctx.set_fill_style(color);
        self.ctx.fill_text(text, x, y);
    }
    
    #[wasm_bindgen]
    pub fn draw_line(&self, x1: f64, y1: f64, x2: f64, y2: f64, color: &str, width: f64) {
        self.ctx.begin_path();
        self.ctx.move_to(x1, y1);
        self.ctx.line_to(x2, y2);
        self.ctx.set_stroke_style(color);
        self.ctx.set_line_width(width);
        self.ctx.stroke();
    }
}
```

### Animation Loop

```rust
use wasm_bindgen::prelude::*;
use web_sys::{window, Performance};

#[wasm_bindgen]
pub struct AnimationLoop {
    callback: Closure<dyn Fn(f64)>,
    animation_id: Option<i32>,
}

#[wasm_bindgen]
impl AnimationLoop {
    #[wasm_bindgen(constructor)]
    pub fn new() -> AnimationLoop {
        let callback = Closure::wrap(Box::new(|timestamp: f64| {
            // Animation logic here
            console::log_1(&format!("Animation frame at: {}", timestamp).into());
        }) as Box<dyn Fn(f64)>);
        
        AnimationLoop {
            callback,
            animation_id: None,
        }
    }
    
    #[wasm_bindgen]
    pub fn start(&mut self) -> Result<(), JsValue> {
        let window = window().unwrap();
        let performance = window.performance().unwrap();
        
        let animation_id = window.request_animation_frame(
            self.callback.as_ref().unchecked_ref()
        )?;
        
        self.animation_id = Some(animation_id);
        Ok(())
    }
    
    #[wasm_bindgen]
    pub fn stop(&mut self) -> Result<(), JsValue> {
        if let Some(id) = self.animation_id {
            let window = window().unwrap();
            window.cancel_animation_frame(id)?;
            self.animation_id = None;
        }
        Ok(())
    }
}
```

---

## WebAssembly Modules

### Complex Data Structures

```rust
use wasm_bindgen::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[wasm_bindgen]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WasmPoint {
    pub x: f64,
    pub y: f64,
}

#[wasm_bindgen]
impl WasmPoint {
    #[wasm_bindgen(constructor)]
    pub fn new(x: f64, y: f64) -> WasmPoint {
        WasmPoint { x, y }
    }
    
    #[wasm_bindgen]
    pub fn distance_to(&self, other: &WasmPoint) -> f64 {
        ((self.x - other.x).powi(2) + (self.y - other.y).powi(2)).sqrt()
    }
    
    #[wasm_bindgen]
    pub fn translate(&mut self, dx: f64, dy: f64) {
        self.x += dx;
        self.y += dy;
    }
}

#[wasm_bindgen]
pub struct GeometryProcessor {
    points: Vec<WasmPoint>,
}

#[wasm_bindgen]
impl GeometryProcessor {
    #[wasm_bindgen(constructor)]
    pub fn new() -> GeometryProcessor {
        GeometryProcessor { points: Vec::new() }
    }
    
    #[wasm_bindgen]
    pub fn add_point(&mut self, point: WasmPoint) {
        self.points.push(point);
    }
    
    #[wasm_bindgen]
    pub fn get_points(&self) -> JsValue {
        JsValue::from_serde(&self.points)
    }
    
    #[wasm_bindgen]
    pub fn calculate_centroid(&self) -> WasmPoint {
        if self.points.is_empty() {
            return WasmPoint::new(0.0, 0.0);
        }
        
        let sum_x: f64 = self.points.iter().map(|p| p.x).sum();
        let sum_y: f64 = self.points.iter().map(|p| p.y).sum();
        let count = self.points.len() as f64;
        
        WasmPoint::new(sum_x / count, sum_y / count)
    }
    
    #[wasm_bindgen]
    pub fn find_nearest_point(&self, target: &WasmPoint) -> Option<WasmPoint> {
        if self.points.is_empty() {
            return None;
        }
        
        let mut nearest = self.points[0].clone();
        let mut min_distance = nearest.distance_to(target);
        
        for point in &self.points[1..] {
            let distance = point.distance_to(target);
            if distance < min_distance {
                nearest = point.clone();
                min_distance = distance;
            }
        }
        
        Some(nearest)
    }
}
```

---

## Error Handling

### WASM Error Types

```rust
use wasm_bindgen::prelude::*;
use js_sys::Error;

#[wasm_bindgen]
pub enum WasmError {
    InvalidInput(String),
    NetworkError(String),
    ProcessingError(String),
}

impl WasmError {
    pub fn to_js_error(&self) -> Error {
        let message = match self {
            WasmError::InvalidInput(msg) => format!("Invalid input: {}", msg),
            WasmError::NetworkError(msg) => format!("Network error: {}", msg),
            WasmError::ProcessingError(msg) => format!("Processing error: {}", msg),
        };
        Error::new(&message)
    }
}

#[wasm_bindgen]
pub fn safe_divide(a: f64, b: f64) -> Result<f64, WasmError> {
    if b == 0.0 {
        Err(WasmError::InvalidInput("Division by zero".to_string()))
    } else {
        Ok(a / b)
    }
}

#[wasm_bindgen]
pub fn process_data(data: &[u8]) -> Result<Vec<u8>, WasmError> {
    if data.is_empty() {
        return Err(WasmError::InvalidInput("Empty data".to_string()));
    }
    
    if data.len() > 1024 * 1024 {
        return Err(WasmError::ProcessingError("Data too large".to_string()));
    }
    
    // Process data
    let mut result = Vec::with_capacity(data.len());
    for &byte in data {
        result.push(byte.wrapping_add(1));
    }
    
    Ok(result)
}
```

---

## Build and Deployment

### Build Commands

```bash
# Build for development
wasm-pack build --dev

# Build for production
wasm-pack build --release

# Build with specific target
wasm-pack build --target web --release

# Build with custom features
wasm-pack build --features "serde, wasm-bindgen-serialize"
```

### Package.json Integration

```json
{
  "name": "wasm-project",
  "version": "0.1.0",
  "scripts": {
    "build": "wasm-pack build --target web --release",
    "dev": "wasm-pack build --target web --dev && webpack serve",
    "serve": "webpack serve"
  },
  "dependencies": {
    "wasm-project": "file:./pkg"
  },
  "devDependencies": {
    "webpack": "^5.0.0",
    "webpack-cli": "^4.0.0",
    "webpack-dev-server": "^4.0.0"
  }
}
```

### Webpack Configuration

```javascript
const path = require('path');

module.exports = {
  entry: './index.js',
  output: {
    path: path.resolve(__dirname, 'dist'),
    filename: 'bundle.js',
  },
  experiments: {
    asyncWebAssembly: true,
  },
  module: {
    rules: [
      {
        test: /\.wasm$/,
        type: 'webassembly/async',
      },
    ],
  },
};
```

---

## Key Takeaways

- **Rust + WASM** provides near-native performance in the browser
- **wasm-bindgen** is the primary tool for JavaScript interop
- **Memory management** requires careful consideration in WASM
- **Performance** can be optimized with proper patterns
- **Async operations** work seamlessly with JavaScript promises
- **DOM manipulation** is possible through web-sys bindings
- **Build tooling** is mature and well-integrated

---

## Common WASM Crates

| Crate | Purpose | Use Case |
|-------|---------|----------|
| `wasm-bindgen` | JavaScript interop | Web development |
| `web-sys` | Web API bindings | Browser APIs |
| `js-sys` | JavaScript bindings | JS interop |
| `wasm-bindgen-futures` | Async support | Promises/Futures |
| `console_error_panic_hook` | Panic handling | Debugging |
| `gloo` | Toolkit | Web utilities |
| `yew` | Framework | Web apps |
