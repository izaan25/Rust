// 27_webassembly_wasm.rs
// Comprehensive examples of WebAssembly (WASM) development in Rust

// Note: This file demonstrates WASM concepts but requires proper build configuration
// with wasm-bindgen and target compilation to wasm32-unknown-unknown

use wasm_bindgen::prelude::*;
use web_sys::{console, Document, Element, HtmlCanvasElement, Window, Performance, CanvasRenderingContext2d};
use js_sys::{ArrayBuffer, Uint8Array, Error, Promise, Reflect};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// =========================================
// BASIC WASM FUNCTIONS
// =========================================

// Export simple arithmetic functions
#[wasm_bindgen]
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

#[wasm_bindgen]
pub fn multiply(a: i32, b: i32) -> i32 {
    a * b
}

#[wasm_bindgen]
pub fn factorial(n: u32) -> u64 {
    match n {
        0 | 1 => 1,
        n => n as u64 * factorial(n - 1),
    }
}

// String processing functions
#[wasm_bindgen]
pub fn greet(name: &str) -> String {
    format!("Hello, {}! Welcome to Rust WebAssembly!", name)
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

#[wasm_bindgen]
pub fn reverse_string(s: &str) -> String {
    s.chars().rev().collect()
}

// =========================================
// WASM STRUCTS AND CLASSES
// =========================================

#[wasm_bindgen]
pub struct Calculator {
    value: f64,
    history: Vec<String>,
}

#[wasm_bindgen]
impl Calculator {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Calculator {
        Calculator {
            value: 0.0,
            history: Vec::new(),
        }
    }
    
    #[wasm_bindgen(getter)]
    pub fn value(&self) -> f64 {
        self.value
    }
    
    #[wasm_bindgen(getter)]
    pub fn history(&self) -> JsValue {
        JsValue::from_serde(&self.history)
    }
    
    #[wasm_bindgen(setter)]
    pub fn set_value(&mut self, value: f64) {
        self.value = value;
    }
    
    #[wasm_bindgen]
    pub fn add(&mut self, other: f64) {
        self.history.push(format!("{} + {} = {}", self.value, other, self.value + other));
        self.value += other;
    }
    
    #[wasm_bindgen]
    pub fn subtract(&mut self, other: f64) {
        self.history.push(format!("{} - {} = {}", self.value, other, self.value - other));
        self.value -= other;
    }
    
    #[wasm_bindgen]
    pub fn multiply(&mut self, other: f64) {
        self.history.push(format!("{} * {} = {}", self.value, other, self.value * other));
        self.value *= other;
    }
    
    #[wasm_bindgen]
    pub fn divide(&mut self, other: f64) -> Result<(), JsValue> {
        if other == 0.0 {
            Err(Error::new("Division by zero").into())
        } else {
            self.history.push(format!("{} / {} = {}", self.value, other, self.value / other));
            self.value /= other;
            Ok(())
        }
    }
    
    #[wasm_bindgen]
    pub fn clear(&mut self) {
        self.value = 0.0;
        self.history.clear();
    }
    
    #[wasm_bindgen]
    pub fn clear_history(&mut self) {
        self.history.clear();
    }
}

// =========================================
// DOM MANIPULATION
// =========================================

#[wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();
    console::log_1(&"Rust WASM module loaded successfully!".into());
}

#[wasm_bindgen]
pub fn create_element(tag: &str, id: Option<&str>, text: Option<&str>) -> Result<Element, JsValue> {
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    
    let element = document.create_element(tag)?;
    
    if let Some(element_id) = id {
        element.set_id(element_id);
    }
    
    if let Some(element_text) = text {
        element.set_text_content(Some(element_text));
    }
    
    Ok(element)
}

#[wasm_bindgen]
pub fn append_to_body(element: &Element) -> Result<(), JsValue> {
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let body = document.body().unwrap();
    
    body.append_child(element)?;
    Ok(())
}

#[wasm_bindgen]
pub fn create_button(text: &str, onclick: &str) -> Result<Element, JsValue> {
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    
    let button = document.create_element("button")?;
    button.set_text_content(Some(text));
    button.set_attribute("onclick", onclick)?;
    
    Ok(button)
}

#[wasm_bindgen]
pub fn create_input(input_type: &str, placeholder: Option<&str>) -> Result<Element, JsValue> {
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    
    let input = document.create_element("input")?;
    input.set_attribute("type", input_type)?;
    
    if let Some(ph) = placeholder {
        input.set_attribute("placeholder", ph)?;
    }
    
    Ok(input)
}

// =========================================
// EVENT HANDLING
// =========================================

#[wasm_bindgen]
pub struct ClickCounter {
    count: u32,
    element: Option<Element>,
}

#[wasm_bindgen]
impl ClickCounter {
    #[wasm_bindgen(constructor)]
    pub fn new() -> ClickCounter {
        ClickCounter {
            count: 0,
            element: None,
        }
    }
    
    #[wasm_bindgen(getter)]
    pub fn count(&self) -> u32 {
        self.count
    }
    
    #[wasm_bindgen]
    pub fn attach_to_element(&mut self, element_id: &str) -> Result<(), JsValue> {
        let window = web_sys::window().unwrap();
        let document = window.document().unwrap();
        
        let element = document.get_element_by_id(element_id)
            .ok_or_else(|| JsValue::from_str("Element not found"))?;
        
        // Store element reference
        self.element = Some(element.clone());
        
        // Set up event listener
        let count = self.count.clone();
        let closure = Closure::wrap(Box::new(move |event: web_sys::MouseEvent| {
            console::log_1(&format!("Mouse clicked at: ({}, {})", 
                event.client_x(), event.client_y()).into());
        }) as Box<dyn Fn(web_sys::MouseEvent)>);
        
        element.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())?;
        closure.forget(); // Keep closure alive
        
        Ok(())
    }
    
    #[wasm_bindgen]
    pub fn increment(&mut self) {
        self.count += 1;
        
        if let Some(ref element) = self.element {
            element.set_text_content(Some(&format!("Count: {}", self.count)));
        }
    }
    
    #[wasm_bindgen]
    pub fn reset(&mut self) {
        self.count = 0;
        
        if let Some(ref element) = self.element {
            element.set_text_content(Some("Count: 0"));
        }
    }
}

// =========================================
// CANVAS GRAPHICS
// =========================================

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
    pub fn draw_line(&self, x1: f64, y1: f64, x2: f64, y2: f64, color: &str, width: f64) {
        self.ctx.begin_path();
        self.ctx.move_to(x1, y1);
        self.ctx.line_to(x2, y2);
        self.ctx.set_stroke_style(color);
        self.ctx.set_line_width(width);
        self.ctx.stroke();
    }
    
    #[wasm_bindgen]
    pub fn draw_text(&self, text: &str, x: f64, y: f64, font: &str, color: &str) {
        self.ctx.set_font(font);
        self.ctx.set_fill_style(color);
        self.ctx.fill_text(text, x, y);
    }
    
    #[wasm_bindgen]
    pub fn draw_gradient_rect(&self, x: f64, y: f64, width: f64, height: f64, color1: &str, color2: &str) {
        let gradient = self.ctx.create_linear_gradient(x, y, x + width, y + height);
        gradient.add_color_stop(0.0, color1);
        gradient.add_color_stop(1.0, color2);
        self.ctx.set_fill_style(&gradient.to_string());
        self.ctx.fill_rect(x, y, width, height);
    }
    
    #[wasm_bindgen]
    pub fn set_global_alpha(&self, alpha: f64) {
        self.ctx.set_global_alpha(alpha);
    }
    
    #[wasm_bindgen]
    pub fn save(&self) {
        self.ctx.save();
    }
    
    #[wasm_bindgen]
    pub fn restore(&self) {
        self.ctx.restore();
    }
}

// =========================================
// ANIMATION LOOP
// =========================================

#[wasm_bindgen]
pub struct AnimationLoop {
    callback: Option<Closure<dyn Fn(f64)>>,
    animation_id: Option<i32>,
    is_running: bool,
}

#[wasm_bindgen]
impl AnimationLoop {
    #[wasm_bindgen(constructor)]
    pub fn new() -> AnimationLoop {
        AnimationLoop {
            callback: None,
            animation_id: None,
            is_running: false,
        }
    }
    
    #[wasm_bindgen]
    pub fn start(&mut self) -> Result<(), JsValue> {
        if self.is_running {
            return Ok(());
        }
        
        let window = web_sys::window().unwrap();
        
        let callback = Closure::wrap(Box::new(|timestamp: f64| {
            console::log_1(&format!("Animation frame at: {}", timestamp).into());
        }) as Box<dyn Fn(f64)>);
        
        let animation_id = window.request_animation_frame(callback.as_ref().unchecked_ref())?;
        
        self.callback = Some(callback);
        self.animation_id = Some(animation_id);
        self.is_running = true;
        
        Ok(())
    }
    
    #[wasm_bindgen]
    pub fn stop(&mut self) -> Result<(), JsValue> {
        if let Some(id) = self.animation_id {
            let window = web_sys::window().unwrap();
            window.cancel_animation_frame(id)?;
            self.animation_id = None;
        }
        
        self.callback = None;
        self.is_running = false;
        Ok(())
    }
    
    #[wasm_bindgen]
    pub fn is_running(&self) -> bool {
        self.is_running
    }
}

// =========================================
// PERFORMANCE MEASUREMENT
// =========================================

#[wasm_bindgen]
pub struct PerformanceMonitor {
    performance: Performance,
}

#[wasm_bindgen]
impl PerformanceMonitor {
    #[wasm_bindgen(constructor)]
    pub fn new() -> PerformanceMonitor {
        let window = web_sys::window().unwrap();
        PerformanceMonitor {
            performance: window.performance().unwrap(),
        }
    }
    
    #[wasm_bindgen]
    pub fn now(&self) -> f64 {
        self.performance.now()
    }
    
    #[wasm_bindgen]
    pub fn measure_time<F, R>(&self, name: &str, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        let start = self.now();
        let result = f();
        let end = self.now();
        
        console::log_1(&format!("{} took: {}ms", name, end - start).into());
        
        result
    }
    
    #[wasm_bindgen]
    pub fn benchmark_function(&self, iterations: u32) -> f64 {
        let start = self.now();
        
        let mut result = 0.0;
        for i in 0..iterations {
            result += (i as f64).sin();
        }
        
        let end = self.now();
        
        console::log_1(&format!("Benchmark with {} iterations took: {}ms", iterations, end - start).into());
        console::log_1(&format!("Result: {}", result).into());
        
        end - start
    }
}

// =========================================
// ASYNC OPERATIONS
// =========================================

use wasm_bindgen_futures::JsFuture;

#[wasm_bindgen]
pub async fn fetch_data(url: &str) -> Result<String, JsValue> {
    let mut opts = web_sys::RequestInit::new();
    opts.method("GET");
    opts.mode(web_sys::RequestMode::Cors);
    
    let request = web_sys::Request::new_with_str_and_init(url, &opts)?;
    
    let window = web_sys::window().unwrap();
    let resp_value = JsFuture::from(window.fetch_with_request(&request)).await?;
    
    let resp: web_sys::Response = resp_value.dyn_into()?;
    let text = JsFuture::from(resp.text()?).await?;
    
    Ok(text.as_string().unwrap_or_default())
}

#[wasm_bindgen]
pub async fn fetch_json(url: &str) -> Result<JsValue, JsValue> {
    let mut opts = web_sys::RequestInit::new();
    opts.method("GET");
    opts.mode(web_sys::RequestMode::Cors);
    
    let request = web_sys::Request::new_with_str_and_init(url, &opts)?;
    
    let window = web_sys::window().unwrap();
    let resp_value = JsFuture::from(window.fetch_with_request(&request)).await?;
    
    let resp: web_sys::Response = resp_value.dyn_into()?;
    let json = JsFuture::from(resp.json()?).await?;
    
    Ok(json)
}

#[wasm_bindgen]
pub async fn process_multiple_urls(urls: Vec<String>) -> Result<Vec<String>, JsValue> {
    let mut results = Vec::new();
    
    for url in urls {
        let data = fetch_data(&url).await?;
        results.push(data);
    }
    
    Ok(results)
}

// =========================================
// DATA STRUCTURES
// =========================================

#[wasm_bindgen]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

#[wasm_bindgen]
impl Point {
    #[wasm_bindgen(constructor)]
    pub fn new(x: f64, y: f64) -> Point {
        Point { x, y }
    }
    
    #[wasm_bindgen]
    pub fn distance_to(&self, other: &Point) -> f64 {
        ((self.x - other.x).powi(2) + (self.y - other.y).powi(2)).sqrt()
    }
    
    #[wasm_bindgen]
    pub fn translate(&mut self, dx: f64, dy: f64) {
        self.x += dx;
        self.y += dy;
    }
    
    #[wasm_bindgen]
    pub fn scale(&mut self, factor: f64) {
        self.x *= factor;
        self.y *= factor;
    }
    
    #[wasm_bindgen]
    pub fn rotate(&mut self, angle: f64) {
        let cos_a = angle.cos();
        let sin_a = angle.sin();
        
        let new_x = self.x * cos_a - self.y * sin_a;
        let new_y = self.x * sin_a + self.y * cos_a;
        
        self.x = new_x;
        self.y = new_y;
    }
}

#[wasm_bindgen]
pub struct GeometryProcessor {
    points: Vec<Point>,
}

#[wasm_bindgen]
impl GeometryProcessor {
    #[wasm_bindgen(constructor)]
    pub fn new() -> GeometryProcessor {
        GeometryProcessor { points: Vec::new() }
    }
    
    #[wasm_bindgen]
    pub fn add_point(&mut self, point: Point) {
        self.points.push(point);
    }
    
    #[wasm_bindgen]
    pub fn get_points(&self) -> JsValue {
        JsValue::from_serde(&self.points)
    }
    
    #[wasm_bindgen]
    pub fn calculate_centroid(&self) -> Point {
        if self.points.is_empty() {
            return Point::new(0.0, 0.0);
        }
        
        let sum_x: f64 = self.points.iter().map(|p| p.x).sum();
        let sum_y: f64 = self.points.iter().map(|p| p.y).sum();
        let count = self.points.len() as f64;
        
        Point::new(sum_x / count, sum_y / count)
    }
    
    #[wasm_bindgen]
    pub fn find_nearest_point(&self, target: &Point) -> Option<Point> {
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
    
    #[wasm_bindgen]
    pub fn calculate_convex_hull(&self) -> Vec<Point> {
        // Simplified convex hull implementation (Graham scan)
        if self.points.len() < 3 {
            return self.points.clone();
        }
        
        // This is a placeholder - real implementation would be more complex
        self.points.clone()
    }
    
    #[wasm_bindgen]
    pub fn clear(&mut self) {
        self.points.clear();
    }
}

// =========================================
// MEMORY MANAGEMENT
// =========================================

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
    
    #[wasm_bindgen]
    pub fn len(&self) -> usize {
        self.buffer.len()
    }
    
    #[wasm_bindgen]
    pub fn as_uint8_array(&self) -> Uint8Array {
        Uint8Array::view(&self.buffer)
    }
    
    #[wasm_bindgen]
    pub fn copy_from(&mut self, source: &[u8]) {
        let copy_len = std::cmp::min(source.len(), self.buffer.len());
        self.buffer[..copy_len].copy_from_slice(&source[..copy_len]);
    }
}

// =========================================
// ERROR HANDLING
// =========================================

#[wasm_bindgen]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WasmError {
    InvalidInput(String),
    NetworkError(String),
    ProcessingError(String),
    ValidationError(String),
}

impl WasmError {
    pub fn to_js_error(&self) -> Error {
        let message = match self {
            WasmError::InvalidInput(msg) => format!("Invalid input: {}", msg),
            WasmError::NetworkError(msg) => format!("Network error: {}", msg),
            WasmError::ProcessingError(msg) => format!("Processing error: {}", msg),
            WasmError::ValidationError(msg) => format!("Validation error: {}", msg),
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
pub fn validate_email(email: &str) -> Result<(), WasmError> {
    if email.is_empty() {
        return Err(WasmError::ValidationError("Email cannot be empty".to_string()));
    }
    
    if !email.contains('@') {
        return Err(WasmError::ValidationError("Invalid email format".to_string()));
    }
    
    Ok(())
}

#[wasm_bindgen]
pub fn process_data_safe(data: &[u8]) -> Result<Vec<u8>, WasmError> {
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

// =========================================
// UTILITY FUNCTIONS
// =========================================

#[wasm_bindgen]
pub fn generate_uuid() -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    
    let mut hasher = DefaultHasher::new();
    std::time::SystemTime::now().hash(&mut hasher);
    let hash = hasher.finish();
    
    format!("{:x}", hash)[..8].to_string()
}

#[wasm_bindgen]
pub fn format_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    
    if bytes == 0 {
        return "0 B".to_string();
    }
    
    let mut size = bytes as f64;
    let mut unit_index = 0;
    
    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }
    
    format!("{:.2} {}", size, UNITS[unit_index])
}

#[wasm_bindgen]
pub fn debounce<F>(callback: Closure<dyn Fn()>, delay_ms: u32) -> Result<(), JsValue> {
    let window = web_sys::window().unwrap();
    
    let timeout_id = window.set_timeout_with_callback_and_timeout_and_arguments_0(
        callback.as_ref().unchecked_ref(),
        delay_ms,
    )?;
    
    callback.forget();
    console::log_1(&format!("Debounce scheduled with timeout: {}", timeout_id).into());
    
    Ok(())
}

// =========================================
// MAIN DEMONSTRATION
// =========================================

#[wasm_bindgen]
pub fn run_demonstration() -> Result<(), JsValue> {
    console::log_1(&"=== WASM DEMONSTRATION ===".into());
    
    // Basic operations
    console::log_1(&format!("add(5, 3) = {}", add(5, 3)).into());
    console::log_1(&format!("factorial(5) = {}", factorial(5)).into());
    
    // String operations
    console::log_1(&greet("WebAssembly").into());
    console::log_1(&process_text("Hello World").into());
    
    // Calculator demo
    let mut calc = Calculator::new();
    calc.add(10.0);
    calc.multiply(2.0);
    console::log_1(&format!("Calculator result: {}", calc.value()).into());
    
    // Geometry demo
    let mut processor = GeometryProcessor::new();
    processor.add_point(Point::new(0.0, 0.0));
    processor.add_point(Point::new(1.0, 1.0));
    processor.add_point(Point::new(2.0, 0.0));
    
    let centroid = processor.calculate_centroid();
    console::log_1(&format!("Centroid: ({}, {})", centroid.x, centroid.y).into());
    
    // Performance demo
    let monitor = PerformanceMonitor::new();
    monitor.benchmark_function(10000);
    
    // Buffer demo
    let mut buffer = WasmBuffer::new(256);
    buffer.fill(42);
    console::log_1(&format!("Buffer first byte: {}", buffer.get(0)).into());
    
    console::log_1(&"=== DEMONSTRATION COMPLETE ===".into());
    
    Ok(())
}

// =========================================
// UNIT TESTS (for simulation only)
// =========================================

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_basic_operations() {
        assert_eq!(add(5, 3), 8);
        assert_eq!(multiply(4, 7), 28);
        assert_eq!(factorial(5), 120);
    }
    
    #[test]
    fn test_string_operations() {
        assert_eq!(greet("World"), "Hello, World! Welcome to Rust WebAssembly!");
        assert_eq!(process_text("Hello"), "hELLO");
        assert_eq!(reverse_string("hello"), "olleh");
    }
    
    #[test]
    fn test_calculator() {
        let mut calc = Calculator::new();
        calc.add(5.0);
        calc.multiply(2.0);
        assert_eq!(calc.value(), 10.0);
        
        calc.subtract(3.0).unwrap();
        assert_eq!(calc.value(), 7.0);
    }
    
    #[test]
    fn test_point_operations() {
        let p1 = Point::new(0.0, 0.0);
        let p2 = Point::new(3.0, 4.0);
        
        assert!((p1.distance_to(&p2) - 5.0).abs() < f64::EPSILON);
        
        let mut p = Point::new(1.0, 1.0);
        p.translate(2.0, 3.0);
        assert_eq!(p.x, 3.0);
        assert_eq!(p.y, 4.0);
    }
    
    #[test]
    fn test_geometry_processor() {
        let mut processor = GeometryProcessor::new();
        processor.add_point(Point::new(0.0, 0.0));
        processor.add_point(Point::new(2.0, 0.0));
        processor.add_point(Point::new(1.0, 2.0));
        
        let centroid = processor.calculate_centroid();
        assert!((centroid.x - 1.0).abs() < f64::EPSILON);
        assert!((centroid.y - 0.6666666666666666).abs() < f64::EPSILON);
    }
    
    #[test]
    fn test_wasm_buffer() {
        let mut buffer = WasmBuffer::new(10);
        assert_eq!(buffer.len(), 10);
        
        buffer.fill(42);
        assert_eq!(buffer.get(0), 42);
        assert_eq!(buffer.get(9), 42);
        
        buffer.set(5, 100);
        assert_eq!(buffer.get(5), 100);
    }
    
    #[test]
    fn test_error_handling() {
        assert_eq!(safe_divide(10.0, 2.0).unwrap(), 5.0);
        assert!(safe_divide(10.0, 0.0).is_err());
        
        assert!(validate_email("test@example.com").is_ok());
        assert!(validate_email("invalid-email").is_err());
    }
}
