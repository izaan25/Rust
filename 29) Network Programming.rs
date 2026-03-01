// 29_network_programming.rs
// Comprehensive examples of network programming in Rust

// Note: This file demonstrates network concepts but requires proper network setup
// and available ports to run actual network operations

use std::collections::HashMap;
use std::error::Error;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{Mutex, Semaphore, RwLock};
use tokio::time::sleep;

// =========================================
// MODELS AND TYPES
// =========================================

#[derive(Debug, Clone)]
pub struct User {
    pub id: u32,
    pub name: String,
    pub email: String,
}

#[derive(Debug, Clone)]
pub struct Message {
    pub id: u32,
    pub content: String,
    pub timestamp: Instant,
    pub sender: String,
}

#[derive(Debug, Clone)]
pub struct NetworkStats {
    pub connections: u32,
    pub messages_sent: u32,
    pub messages_received: u32,
    pub bytes_transferred: u64,
}

// =========================================
// SIMULATED NETWORK CONNECTIONS
// =========================================

// Simulated TCP connection
pub struct SimulatedTcpConnection {
    pub id: u32,
    pub remote_addr: String,
    pub local_addr: String,
    pub buffer: Arc<Mutex<Vec<u8>>>,
    pub stats: Arc<Mutex<NetworkStats>>,
}

impl SimulatedTcpConnection {
    pub fn new(id: u32, remote_addr: String, local_addr: String) -> Self {
        SimulatedTcpConnection {
            id,
            remote_addr,
            local_addr,
            buffer: Arc::new(Mutex::new(Vec::new())),
            stats: Arc::new(Mutex::new(NetworkStats {
                connections: 1,
                messages_sent: 0,
                messages_received: 0,
                bytes_transferred: 0,
            })),
        }
    }
    
    pub async fn send(&self, data: &[u8]) -> Result<(), Box<dyn Error>> {
        let mut stats = self.stats.lock().await;
        stats.messages_sent += 1;
        stats.bytes_transferred += data.len() as u64;
        
        println!("TCP[{}] Sent {} bytes to {}", self.id, data.len(), self.remote_addr);
        Ok(())
    }
    
    pub async fn receive(&self) -> Result<Vec<u8>, Box<dyn Error>> {
        let mut buffer = self.buffer.lock().await;
        if !buffer.is_empty() {
            let data = buffer.clone();
            buffer.clear();
            
            let mut stats = self.stats.lock().await;
            stats.messages_received += 1;
            stats.bytes_transferred += data.len() as u64;
            
            println!("TCP[{}] Received {} bytes from {}", self.id, data.len(), self.remote_addr);
            Ok(data)
        } else {
            Err("No data available".into())
        }
    }
    
    pub async fn simulate_incoming_data(&self, data: Vec<u8>) {
        let mut buffer = self.buffer.lock().await;
        buffer.extend(data);
    }
}

// Simulated UDP socket
pub struct SimulatedUdpSocket {
    pub local_addr: String,
    pub buffer: Arc<Mutex<Vec<(String, Vec<u8>)>>>,
    pub stats: Arc<Mutex<NetworkStats>>,
}

impl SimulatedUdpSocket {
    pub fn new(local_addr: String) -> Self {
        SimulatedUdpSocket {
            local_addr,
            buffer: Arc::new(Mutex::new(Vec::new())),
            stats: Arc::new(Mutex::new(NetworkStats {
                connections: 0,
                messages_sent: 0,
                messages_received: 0,
                bytes_transferred: 0,
            })),
        }
    }
    
    pub async fn send_to(&self, data: &[u8], addr: &str) -> Result<(), Box<dyn Error>> {
        let mut stats = self.stats.lock().await;
        stats.messages_sent += 1;
        stats.bytes_transferred += data.len() as u64;
        
        println!("UDP Sent {} bytes to {}", data.len(), addr);
        Ok(())
    }
    
    pub async fn receive_from(&self) -> Result<(String, Vec<u8>), Box<dyn Error>> {
        let mut buffer = self.buffer.lock().await;
        if !buffer.is_empty() {
            let (addr, data) = buffer.remove(0);
            
            let mut stats = self.stats.lock().await;
            stats.messages_received += 1;
            stats.bytes_transferred += data.len() as u64;
            
            println!("UDP Received {} bytes from {}", data.len(), addr);
            Ok((addr, data))
        } else {
            Err("No data available".into())
        }
    }
    
    pub async fn simulate_incoming_data(&self, addr: String, data: Vec<u8>) {
        let mut buffer = self.buffer.lock().await;
        buffer.push((addr, data));
    }
}

// =========================================
// TCP SERVER AND CLIENT
// =========================================

pub struct TcpServer {
    connections: Arc<RwLock<HashMap<u32, SimulatedTcpConnection>>>,
    next_connection_id: Arc<Mutex<u32>>,
}

impl TcpServer {
    pub fn new() -> Self {
        TcpServer {
            connections: Arc::new(RwLock::new(HashMap::new())),
            next_connection_id: Arc::new(Mutex::new(1)),
        }
    }
    
    pub async fn bind(&self, addr: &str) -> Result<(), Box<dyn Error>> {
        println!("TCP Server listening on {}", addr);
        
        // Simulate accepting connections
        for i in 1..=3 {
            let client_addr = format!("127.0.0.1:{}", 8000 + i);
            self.simulate_accept_connection(client_addr).await?;
        }
        
        Ok(())
    }
    
    async fn simulate_accept_connection(&self, client_addr: String) -> Result<u32, Box<dyn Error>> {
        let mut next_id = self.next_connection_id.lock().await;
        let connection_id = *next_id;
        *next_id += 1;
        
        let connection = SimulatedTcpConnection::new(
            connection_id,
            client_addr.clone(),
            "127.0.0.1:8080".to_string(),
        );
        
        let mut connections = self.connections.write().await;
        connections.insert(connection_id, connection);
        
        println!("TCP Server accepted connection {} from {}", connection_id, client_addr);
        Ok(connection_id)
    }
    
    pub async fn handle_connection(&self, connection_id: u32) -> Result<(), Box<dyn Error>> {
        let connections = self.connections.read().await;
        
        if let Some(connection) = connections.get(&connection_id) {
            // Simulate receiving data
            connection.simulate_incoming_data(b"Hello from client".to_vec()).await;
            
            // Process the data
            if let Ok(data) = connection.receive().await {
                let response = format!("Echo: {}", String::from_utf8_lossy(&data));
                connection.send(response.as_bytes()).await?;
            }
        }
        
        Ok(())
    }
    
    pub async fn broadcast(&self, message: &[u8]) -> Result<(), Box<dyn Error>> {
        let connections = self.connections.read().await;
        
        for connection in connections.values() {
            connection.send(message).await?;
        }
        
        Ok(())
    }
    
    pub async fn get_stats(&self) -> NetworkStats {
        let connections = self.connections.read().await;
        let mut total_stats = NetworkStats {
            connections: connections.len() as u32,
            messages_sent: 0,
            messages_received: 0,
            bytes_transferred: 0,
        };
        
        for connection in connections.values() {
            let stats = connection.stats.lock().await;
            total_stats.messages_sent += stats.messages_sent;
            total_stats.messages_received += stats.messages_received;
            total_stats.bytes_transferred += stats.bytes_transferred;
        }
        
        total_stats
    }
}

pub struct TcpClient {
    connection: Option<SimulatedTcpConnection>,
}

impl TcpClient {
    pub fn new() -> Self {
        TcpClient { connection: None }
    }
    
    pub async fn connect(&mut self, server_addr: &str) -> Result<(), Box<dyn Error>> {
        let connection = SimulatedTcpConnection::new(
            1,
            server_addr.to_string(),
            "127.0.0.1:9000".to_string(),
        );
        
        self.connection = Some(connection);
        println!("TCP Client connected to {}", server_addr);
        Ok(())
    }
    
    pub async fn send(&self, data: &[u8]) -> Result<(), Box<dyn Error>> {
        if let Some(ref connection) = self.connection {
            connection.send(data).await?;
        }
        Ok(())
    }
    
    pub async fn receive(&self) -> Result<Vec<u8>, Box<dyn Error>> {
        if let Some(ref connection) = self.connection {
            connection.receive().await
        } else {
            Err("Not connected".into())
        }
    }
    
    pub async fn get_stats(&self) -> Option<NetworkStats> {
        if let Some(ref connection) = self.connection {
            Some(connection.stats.lock().await.clone())
        } else {
            None
        }
    }
}

// =========================================
// UDP SERVER AND CLIENT
// =========================================

pub struct UdpServer {
    socket: SimulatedUdpSocket,
}

impl UdpServer {
    pub fn new(addr: &str) -> Self {
        UdpServer {
            socket: SimulatedUdpSocket::new(addr.to_string()),
        }
    }
    
    pub async fn bind(&self) -> Result<(), Box<dyn Error>> {
        println!("UDP Server listening on {}", self.socket.local_addr);
        Ok(())
    }
    
    pub async fn receive(&self) -> Result<(String, Vec<u8>), Box<dyn Error>> {
        self.socket.receive_from().await
    }
    
    pub async fn send_to(&self, data: &[u8], addr: &str) -> Result<(), Box<dyn Error>> {
        self.socket.send_to(data, addr).await
    }
    
    pub async fn simulate_client_message(&self, client_addr: String, message: Vec<u8>) {
        self.socket.simulate_incoming_data(client_addr, message).await;
    }
    
    pub async fn get_stats(&self) -> NetworkStats {
        self.socket.stats.lock().await.clone()
    }
}

pub struct UdpClient {
    socket: SimulatedUdpSocket,
    server_addr: String,
}

impl UdpClient {
    pub fn new(server_addr: &str) -> Self {
        UdpClient {
            socket: SimulatedUdpSocket::new("127.0.0.1:9001".to_string()),
            server_addr: server_addr.to_string(),
        }
    }
    
    pub async fn send(&self, data: &[u8]) -> Result<(), Box<dyn Error>> {
        self.socket.send_to(data, &self.server_addr).await
    }
    
    pub async fn receive(&self) -> Result<Vec<u8>, Box<dyn Error>> {
        let (addr, data) = self.socket.receive_from().await?;
        if addr == self.server_addr {
            Ok(data)
        } else {
            Err("Received data from unexpected source".into())
        }
    }
    
    pub async fn simulate_server_response(&self, response: Vec<u8>) {
        self.socket.simulate_incoming_data(self.server_addr.clone(), response).await;
    }
    
    pub async fn get_stats(&self) -> NetworkStats {
        self.socket.stats.lock().await.clone()
    }
}

// =========================================
// HTTP SERVER SIMULATION
// =========================================

#[derive(Debug, Clone)]
pub struct HttpRequest {
    pub method: String,
    pub path: String,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct HttpResponse {
    pub status_code: u16,
    pub status_text: String,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
}

impl HttpResponse {
    pub fn ok(body: Vec<u8>) -> Self {
        HttpResponse {
            status_code: 200,
            status_text: "OK".to_string(),
            headers: HashMap::new(),
            body,
        }
    }
    
    pub fn not_found() -> Self {
        HttpResponse {
            status_code: 404,
            status_text: "Not Found".to_string(),
            headers: HashMap::new(),
            body: b"Not Found".to_vec(),
        }
    }
    
    pub fn internal_server_error() -> Self {
        HttpResponse {
            status_code: 500,
            status_text: "Internal Server Error".to_string(),
            headers: HashMap::new(),
            body: b"Internal Server Error".to_vec(),
        }
    }
}

pub type HttpHandler = Box<dyn Fn(HttpRequest) -> HttpResponse + Send + Sync>;

pub struct HttpServer {
    handlers: Arc<RwLock<HashMap<String, HttpHandler>>>,
    middleware: Arc<Mutex<Vec<Box<dyn Fn(HttpRequest) -> HttpRequest + Send + Sync>>>>,
}

impl HttpServer {
    pub fn new() -> Self {
        HttpServer {
            handlers: Arc::new(RwLock::new(HashMap::new())),
            middleware: Arc::new(Mutex::new(Vec::new())),
        }
    }
    
    pub fn add_handler<F>(&self, path: &str, handler: F)
    where
        F: Fn(HttpRequest) -> HttpResponse + Send + Sync + 'static,
    {
        let mut handlers = self.handlers.write().await;
        handlers.insert(path.to_string(), Box::new(handler));
    }
    
    pub fn add_middleware<F>(&self, middleware: F)
    where
        F: Fn(HttpRequest) -> HttpRequest + Send + Sync + 'static,
    {
        let mut middleware_list = self.middleware.lock().await;
        middleware_list.push(Box::new(middleware));
    }
    
    pub async fn handle_request(&self, request: HttpRequest) -> HttpResponse {
        // Apply middleware
        let mut processed_request = request;
        {
            let middleware = self.middleware.lock().await;
            for middleware_fn in middleware.iter() {
                processed_request = middleware_fn(processed_request);
            }
        }
        
        // Find handler
        let handlers = self.handlers.read().await;
        
        if let Some(handler) = handlers.get(&processed_request.path) {
            handler(processed_request)
        } else {
            HttpResponse::not_found()
        }
    }
    
    pub async fn simulate_request(&self, method: &str, path: &str, body: Vec<u8>) -> HttpResponse {
        let request = HttpRequest {
            method: method.to_string(),
            path: path.to_string(),
            headers: HashMap::new(),
            body,
        };
        
        self.handle_request(request).await
    }
}

// =========================================
// WEBSOCKET SIMULATION
// =========================================

#[derive(Debug, Clone)]
pub enum WebSocketMessage {
    Text(String),
    Binary(Vec<u8>),
    Ping(Vec<u8>),
    Pong(Vec<u8>),
    Close,
}

pub struct WebSocketConnection {
    pub id: u32,
    pub messages: Arc<Mutex<Vec<WebSocketMessage>>>,
    pub stats: Arc<Mutex<NetworkStats>>,
}

impl WebSocketConnection {
    pub fn new(id: u32) -> Self {
        WebSocketConnection {
            id,
            messages: Arc::new(Mutex::new(Vec::new())),
            stats: Arc::new(Mutex::new(NetworkStats {
                connections: 1,
                messages_sent: 0,
                messages_received: 0,
                bytes_transferred: 0,
            })),
        }
    }
    
    pub async fn send(&self, message: WebSocketMessage) -> Result<(), Box<dyn Error>> {
        let mut stats = self.stats.lock().await;
        stats.messages_sent += 1;
        
        let bytes = match &message {
            WebSocketMessage::Text(text) => text.len(),
            WebSocketMessage::Binary(data) => data.len(),
            WebSocketMessage::Ping(data) => data.len(),
            WebSocketMessage::Pong(data) => data.len(),
            WebSocketMessage::Close => 0,
        };
        stats.bytes_transferred += bytes as u64;
        
        println!("WebSocket[{}] Sent: {:?}", self.id, message);
        Ok(())
    }
    
    pub async fn receive(&self) -> Option<WebSocketMessage> {
        let mut messages = self.messages.lock().await;
        if !messages.is_empty() {
            let message = messages.remove(0);
            
            let mut stats = self.stats.lock().await;
            stats.messages_received += 1;
            
            let bytes = match &message {
                WebSocketMessage::Text(text) => text.len(),
                WebSocketMessage::Binary(data) => data.len(),
                WebSocketMessage::Ping(data) => data.len(),
                WebSocketMessage::Pong(data) => data.len(),
                WebSocketMessage::Close => 0,
            };
            stats.bytes_transferred += bytes as u64;
            
            println!("WebSocket[{}] Received: {:?}", self.id, message);
            Some(message)
        } else {
            None
        }
    }
    
    pub async fn simulate_incoming_message(&self, message: WebSocketMessage) {
        let mut messages = self.messages.lock().await;
        messages.push(message);
    }
}

pub struct WebSocketServer {
    connections: Arc<RwLock<HashMap<u32, WebSocketConnection>>>,
    next_connection_id: Arc<Mutex<u32>>,
}

impl WebSocketServer {
    pub fn new() -> Self {
        WebSocketServer {
            connections: Arc::new(RwLock::new(HashMap::new())),
            next_connection_id: Arc::new(Mutex::new(1)),
        }
    }
    
    pub async fn accept_connection(&self) -> u32 {
        let mut next_id = self.next_connection_id.lock().await;
        let connection_id = *next_id;
        *next_id += 1;
        
        let connection = WebSocketConnection::new(connection_id);
        let mut connections = self.connections.write().await;
        connections.insert(connection_id, connection);
        
        println!("WebSocket Server accepted connection {}", connection_id);
        connection_id
    }
    
    pub async fn send_to_connection(&self, connection_id: u32, message: WebSocketMessage) -> Result<(), Box<dyn Error>> {
        let connections = self.connections.read().await;
        
        if let Some(connection) = connections.get(&connection_id) {
            connection.send(message).await?;
        } else {
            return Err("Connection not found".into());
        }
        
        Ok(())
    }
    
    pub async fn broadcast(&self, message: WebSocketMessage) -> Result<(), Box<dyn Error>> {
        let connections = self.connections.read().await;
        
        for connection in connections.values() {
            connection.send(message.clone()).await?;
        }
        
        Ok(())
    }
    
    pub async fn handle_connection(&self, connection_id: u32) -> Result<(), Box<dyn Error>> {
        let connections = self.connections.read().await;
        
        if let Some(connection) = connections.get(&connection_id) {
            // Simulate receiving messages
            connection.simulate_incoming_message(WebSocketMessage::Text("Hello WebSocket".to_string())).await;
            
            if let Some(message) = connection.receive().await {
                match message {
                    WebSocketMessage::Text(text) => {
                        let response = WebSocketMessage::Text(format!("Echo: {}", text));
                        connection.send(response).await?;
                    }
                    WebSocketMessage::Ping(data) => {
                        let pong = WebSocketMessage::Pong(data);
                        connection.send(pong).await?;
                    }
                    WebSocketMessage::Close => {
                        println!("WebSocket[{}] Connection closed", connection_id);
                    }
                    _ => {}
                }
            }
        }
        
        Ok(())
    }
}

// =========================================
// CONNECTION POOL
// =========================================

pub struct ConnectionPool<T> {
    connections: Arc<Mutex<Vec<T>>>,
    semaphore: Arc<Semaphore>,
    max_size: usize,
}

impl<T> ConnectionPool<T> {
    pub fn new(max_size: usize) -> Self {
        ConnectionPool {
            connections: Arc::new(Mutex::new(Vec::new())),
            semaphore: Arc::new(Semaphore::new(max_size)),
            max_size,
        }
    }
    
    pub async fn acquire(&self) -> Option<PooledConnection<T>> {
        let _permit = self.semaphore.acquire().await.ok()?;
        
        let mut connections = self.connections.lock().await;
        
        if let Some(conn) = connections.pop() {
            Some(PooledConnection {
                connection: Some(conn),
                pool: self.connections.clone(),
                _permit: self.semaphore.clone(),
            })
        } else {
            None
        }
    }
    
    pub async fn release(&self, connection: T) {
        let mut connections = self.connections.lock().await;
        if connections.len() < self.max_size {
            connections.push(connection);
        }
    }
}

pub struct PooledConnection<T> {
    connection: Option<T>,
    pool: Arc<Mutex<Vec<T>>>,
    _permit: Arc<Semaphore>,
}

impl<T> PooledConnection<T> {
    pub fn get(&self) -> Option<&T> {
        self.connection.as_ref()
    }
    
    pub fn get_mut(&mut self) -> Option<&mut T> {
        self.connection.as_mut()
    }
}

impl<T> Drop for PooledConnection<T> {
    fn drop(&mut self) {
        if let Some(conn) = self.connection.take() {
            let pool = self.pool.clone();
            tokio::spawn(async move {
                let mut connections = pool.lock().await;
                connections.push(conn);
            });
        }
    }
}

// =========================================
// RATE LIMITER
// =========================================

pub struct RateLimiter {
    max_requests: u32,
    window: Duration,
    requests: Arc<Mutex<Vec<Instant>>>,
}

impl RateLimiter {
    pub fn new(max_requests: u32, window: Duration) -> Self {
        RateLimiter {
            max_requests,
            window,
            requests: Arc::new(Mutex::new(Vec::new())),
        }
    }
    
    pub async fn check_rate_limit(&self) -> bool {
        let now = Instant::now();
        let mut requests = self.requests.lock().await;
        
        // Remove old requests
        requests.retain(|&time| now.duration_since(time) < self.window);
        
        if requests.len() < self.max_requests as usize {
            requests.push(now);
            true
        } else {
            false
        }
    }
    
    pub async fn wait_if_needed(&self) -> Result<(), Box<dyn Error>> {
        while !self.check_rate_limit().await {
            sleep(Duration::from_millis(100)).await;
        }
        Ok(())
    }
}

// =========================================
// LOAD BALANCER
// =========================================

#[derive(Debug, Clone)]
pub enum LoadBalancingStrategy {
    RoundRobin,
    Random,
    LeastConnections,
}

pub struct LoadBalancer {
    servers: Arc<Mutex<Vec<String>>>,
    strategy: LoadBalancingStrategy,
    current_index: Arc<Mutex<usize>>,
    connections: Arc<Mutex<HashMap<String, u32>>>,
}

impl LoadBalancer {
    pub fn new(servers: Vec<String>, strategy: LoadBalancingStrategy) -> Self {
        let connections = servers.iter().map(|s| (s.clone(), 0)).collect();
        
        LoadBalancer {
            servers: Arc::new(Mutex::new(servers)),
            strategy,
            current_index: Arc::new(Mutex::new(0)),
            connections: Arc::new(Mutex::new(connections)),
        }
    }
    
    pub async fn get_server(&self) -> Option<String> {
        let mut servers = self.servers.lock().await;
        
        if servers.is_empty() {
            return None;
        }
        
        let server = match self.strategy {
            LoadBalancingStrategy::RoundRobin => {
                let mut index = self.current_index.lock().await;
                let server = servers[*index].clone();
                *index = (*index + 1) % servers.len();
                server
            }
            LoadBalancingStrategy::Random => {
                use rand::Rng;
                let mut rng = rand::thread_rng();
                let index = rng.gen_range(0..servers.len());
                servers[index].clone()
            }
            LoadBalancingStrategy::LeastConnections => {
                let connections = self.connections.lock().await;
                let mut min_connections = u32::MAX;
                let mut best_server = None;
                
                for server in servers.iter() {
                    let conn_count = connections.get(server).unwrap_or(&0);
                    if conn_count < &min_connections {
                        min_connections = *conn_count;
                        best_server = Some(server.clone());
                    }
                }
                
                best_server
            }
        };
        
        // Increment connection count
        if let Some(ref server) = server {
            let mut connections = self.connections.lock().await;
            *connections.entry(server.clone()).or_insert(0) += 1;
        }
        
        server
    }
    
    pub async fn release_server(&self, server: &str) {
        let mut connections = self.connections.lock().await;
        if let Some(count) = connections.get_mut(server) {
            if *count > 0 {
                *count -= 1;
            }
        }
    }
    
    pub async fn add_server(&self, server: String) {
        let mut servers = self.servers.lock().await;
        servers.push(server);
    }
    
    pub async fn remove_server(&self, server: &str) -> bool {
        let mut servers = self.servers.lock().await;
        if let Some(pos) = servers.iter().position(|s| s == server) {
            servers.remove(pos);
            
            let mut connections = self.connections.lock().await;
            connections.remove(server);
            
            true
        } else {
            false
        }
    }
}

// =========================================
// DEMONSTRATION FUNCTIONS
// =========================================

pub async fn demonstrate_tcp_server_client() -> Result<(), Box<dyn Error>> {
    println!("=== TCP SERVER/CLIENT DEMONSTRATION ===");
    
    // Create server
    let server = TcpServer::new();
    server.bind("127.0.0.1:8080").await?;
    
    // Handle connections
    let server_clone = server.clone();
    tokio::spawn(async move {
        for i in 1..=3 {
            if let Err(e) = server_clone.handle_connection(i).await {
                eprintln!("Error handling connection {}: {}", i, e);
            }
        }
    });
    
    // Create clients
    let mut client1 = TcpClient::new();
    let mut client2 = TcpClient::new();
    
    client1.connect("127.0.0.1:8080").await?;
    client2.connect("127.0.0.1:8080").await?;
    
    // Send messages
    client1.send(b"Hello from client 1").await?;
    client2.send(b"Hello from client 2").await?;
    
    // Receive responses
    if let Ok(response) = client1.receive().await {
        println!("Client 1 received: {}", String::from_utf8_lossy(&response));
    }
    
    if let Ok(response) = client2.receive().await {
        println!("Client 2 received: {}", String::from_utf8_lossy(&response));
    }
    
    // Broadcast message
    server.broadcast(b"Broadcast message to all clients").await?;
    
    // Show stats
    let stats = server.get_stats().await;
    println!("Server stats: {:?}", stats);
    
    Ok(())
}

pub async fn demonstrate_udp_server_client() -> Result<(), Box<dyn Error>> {
    println!("\n=== UDP SERVER/CLIENT DEMONSTRATION ===");
    
    // Create server
    let server = UdpServer::new("127.0.0.1:8080");
    server.bind().await?;
    
    // Create clients
    let client1 = UdpClient::new("127.0.0.1:8080");
    let client2 = UdpClient::new("127.0.0.1:8080");
    
    // Send messages
    client1.send(b"Hello from UDP client 1").await?;
    client2.send(b"Hello from UDP client 2").await?;
    
    // Simulate server receiving messages
    server.simulate_client_message("127.0.0.1:9001".to_string(), b"Hello from UDP client 1".to_vec()).await;
    server.simulate_client_message("127.0.0.1:9002".to_string(), b"Hello from UDP client 2".to_vec()).await;
    
    // Server processes messages
    while let Ok((addr, data)) = server.receive().await {
        let response = format!("UDP Echo: {}", String::from_utf8_lossy(&data));
        server.send_to(response.as_bytes(), &addr).await?;
        
        // Simulate client receiving response
        if addr == "127.0.0.1:9001" {
            client1.simulate_server_response(response.as_bytes()).await;
        } else if addr == "127.0.0.1:9002" {
            client2.simulate_server_response(response.as_bytes()).await;
        }
    }
    
    // Show stats
    let stats = server.get_stats().await;
    println!("UDP Server stats: {:?}", stats);
    
    Ok(())
}

pub async fn demonstrate_http_server() -> Result<(), Box<dyn Error>> {
    println!("\n=== HTTP SERVER DEMONSTRATION ===");
    
    let server = HttpServer::new();
    
    // Add middleware
    server.add_middleware(|mut req| {
        println!("Middleware processing: {} {}", req.method, req.path);
        req.headers.insert("X-Processed".to_string(), "true".to_string());
        req
    });
    
    // Add handlers
    server.add_handler("/", |_req| {
        HttpResponse::ok(b"Welcome to the HTTP Server!".to_vec())
    });
    
    server.add_handler("/api/users", |_req| {
        let users = vec![
            User { id: 1, name: "Alice".to_string(), email: "alice@example.com".to_string() },
            User { id: 2, name: "Bob".to_string(), email: "bob@example.com".to_string() },
        ];
        
        let json = serde_json::to_string(&users).unwrap();
        HttpResponse::ok(json.into_bytes())
    });
    
    server.add_handler("/api/status", |_req| {
        HttpResponse::ok(b"Server is running".to_vec())
    });
    
    // Simulate requests
    let response1 = server.simulate_request("GET", "/", b"".to_vec());
    println!("GET / -> {}", response1.status_code);
    
    let response2 = server.simulate_request("GET", "/api/users", b"".to_vec());
    println!("GET /api/users -> {}", response2.status_code);
    
    let response3 = server.simulate_request("GET", "/api/status", b"".to_vec());
    println!("GET /api/status -> {}", response3.status_code);
    
    let response4 = server.simulate_request("GET", "/nonexistent", b"".to_vec());
    println!("GET /nonexistent -> {}", response4.status_code);
    
    Ok(())
}

pub async fn demonstrate_websocket() -> Result<(), Box<dyn Error>> {
    println!("\n=== WEBSOCKET DEMONSTRATION ===");
    
    let server = WebSocketServer::new();
    
    // Accept connections
    let conn1 = server.accept_connection().await;
    let conn2 = server.accept_connection().await;
    
    // Handle connections
    let server_clone = server.clone();
    tokio::spawn(async move {
        server_clone.handle_connection(conn1).await.ok();
    });
    
    let server_clone = server.clone();
    tokio::spawn(async move {
        server_clone.handle_connection(conn2).await.ok();
    });
    
    // Send messages
    server.send_to_connection(conn1, WebSocketMessage::Text("Welcome to WebSocket!".to_string())).await?;
    server.send_to_connection(conn2, WebSocketMessage::Text("Welcome to WebSocket!".to_string())).await?;
    
    // Broadcast message
    server.broadcast(WebSocketMessage::Text("Broadcast to all connections".to_string())).await?;
    
    // Send ping
    server.send_to_connection(conn1, WebSocketMessage::Ping(b"ping".to_vec())).await?;
    
    Ok(())
}

pub async fn demonstrate_connection_pool() -> Result<(), Box<dyn Error>> {
    println!("\n=== CONNECTION POOL DEMONSTRATION ===");
    
    let pool: ConnectionPool<String> = ConnectionPool::new(3);
    
    // Add some connections to the pool
    pool.release("Connection 1".to_string()).await;
    pool.release("Connection 2".to_string()).await;
    pool.release("Connection 3".to_string()).await;
    
    // Acquire connections
    let conn1 = pool.acquire().await;
    let conn2 = pool.acquire().await;
    let conn3 = pool.acquire().await;
    let conn4 = pool.acquire().await; // Should be None
    
    println!("Acquired connections: {}, {}, {}, {}", 
             conn1.is_some(), conn2.is_some(), conn3.is_some(), conn4.is_some());
    
    // Use connections
    if let Some(conn) = conn1 {
        println!("Using connection: {:?}", conn.get());
    }
    
    // Connection is returned to pool when dropped
    drop(conn1);
    
    // Should be able to acquire again
    let conn5 = pool.acquire().await;
    println!("After drop, acquired connection: {}", conn5.is_some());
    
    Ok(())
}

pub async fn demonstrate_rate_limiting() -> Result<(), Box<dyn Error>> {
    println!("\n=== RATE LIMITING DEMONSTRATION ===");
    
    let rate_limiter = RateLimiter::new(3, Duration::from_secs(2));
    
    // Test rate limiting
    for i in 1..=5 {
        if rate_limiter.check_rate_limit().await {
            println!("Request {} allowed", i);
        } else {
            println!("Request {} rate limited", i);
        }
    }
    
    // Wait and try again
    sleep(Duration::from_secs(2)).await;
    
    if rate_limiter.check_rate_limit().await {
        println!("Request after wait allowed");
    }
    
    // Test wait_if_needed
    println!("Testing wait_if_needed...");
    let start = Instant::now();
    rate_limiter.wait_if_needed().await?;
    let elapsed = start.elapsed();
    println!("Wait took: {:?}", elapsed);
    
    Ok(())
}

pub async fn demonstrate_load_balancer() -> Result<(), Box<dyn Error>> {
    println!("\n=== LOAD BALANCER DEMONSTRATION ===");
    
    let servers = vec![
        "server1.example.com".to_string(),
        "server2.example.com".to_string(),
        "server3.example.com".to_string(),
    ];
    
    // Test Round Robin
    println!("Round Robin:");
    let lb_rr = LoadBalancer::new(servers.clone(), LoadBalancingStrategy::RoundRobin);
    for i in 0..6 {
        if let Some(server) = lb_rr.get_server().await {
            println!("Request {} -> {}", i + 1, server);
            lb_rr.release_server(&server).await;
        }
    }
    
    // Test Random
    println!("\nRandom:");
    let lb_random = LoadBalancer::new(servers.clone(), LoadBalancingStrategy::Random);
    for i in 0..6 {
        if let Some(server) = lb_random.get_server().await {
            println!("Request {} -> {}", i + 1, server);
            lb_random.release_server(&server).await;
        }
    }
    
    // Test Least Connections
    println!("\nLeast Connections:");
    let lb_lc = LoadBalancer::new(servers.clone(), LoadBalancingStrategy::LeastConnections);
    for i in 0..6 {
        if let Some(server) = lb_lc.get_server().await {
            println!("Request {} -> {}", i + 1, server);
            // Simulate some connections staying active
            if i % 2 == 0 {
                lb_lc.release_server(&server).await;
            }
        }
    }
    
    // Test server management
    println!("\nServer Management:");
    let lb_mgmt = LoadBalancer::new(servers.clone(), LoadBalancingStrategy::RoundRobin);
    
    lb_mgmt.add_server("server4.example.com".to_string()).await;
    println!("Added server4.example.com");
    
    lb_mgmt.remove_server("server2.example.com").await;
    println!("Removed server2.example.com");
    
    if let Some(server) = lb_mgmt.get_server().await {
        println!("Got server: {}", server);
    }
    
    Ok(())
}

// =========================================
// MAIN DEMONSTRATION
// =========================================

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("=== NETWORK PROGRAMMING DEMONSTRATIONS ===");
    
    demonstrate_tcp_server_client().await?;
    demonstrate_udp_server_client().await?;
    demonstrate_http_server().await?;
    demonstrate_websocket().await?;
    demonstrate_connection_pool().await?;
    demonstrate_rate_limiting().await?;
    demonstrate_load_balancer().await?;
    
    println!("\n=== NETWORK PROGRAMMING DEMONSTRATIONS COMPLETE ===");
    println!("Note: This uses simulated network components. Real implementations would use:");
    println!("- tokio for async runtime");
    println!("- hyper for HTTP client/server");
    println!("- axum/warp for web frameworks");
    println!("- tokio-tungstenite for WebSockets");
    println!("- reqwest for HTTP clients");
    
    Ok(())
}

// =========================================
// UNIT TESTS
// =========================================

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_tcp_connection() {
        let connection = SimulatedTcpConnection::new(1, "127.0.0.1:8080".to_string(), "127.0.0.1:9000".to_string());
        
        connection.send(b"test message").await.unwrap();
        connection.simulate_incoming_data(b"test response".to_vec()).await;
        
        let response = connection.receive().await.unwrap();
        assert_eq!(response, b"test response");
    }
    
    #[tokio::test]
    async fn test_udp_socket() {
        let socket = SimulatedUdpSocket::new("127.0.0.1:8080".to_string());
        
        socket.send_to(b"test message", "127.0.0.1:9000").await.unwrap();
        socket.simulate_incoming_data("127.0.0.1:9000".to_string(), b"test response".to_vec()).await;
        
        let (addr, response) = socket.receive_from().await.unwrap();
        assert_eq!(addr, "127.0.0.1:9000");
        assert_eq!(response, b"test response");
    }
    
    #[tokio::test]
    async fn test_http_server() {
        let server = HttpServer::new();
        
        server.add_handler("/test", |_req| {
            HttpResponse::ok(b"Test response".to_vec())
        });
        
        let response = server.simulate_request("GET", "/test", b"".to_vec());
        assert_eq!(response.status_code, 200);
        assert_eq!(response.body, b"Test response");
    }
    
    #[tokio::test]
    async fn test_websocket_connection() {
        let connection = WebSocketConnection::new(1);
        
        connection.send(WebSocketMessage::Text("test".to_string())).await.unwrap();
        connection.simulate_incoming_message(WebSocketMessage::Text("response".to_string())).await;
        
        let message = connection.receive().await.unwrap();
        match message {
            WebSocketMessage::Text(text) => assert_eq!(text, "response"),
            _ => panic!("Expected text message"),
        }
    }
    
    #[tokio::test]
    async fn test_connection_pool() {
        let pool: ConnectionPool<String> = ConnectionPool::new(2);
        
        pool.release("conn1".to_string()).await;
        pool.release("conn2".to_string()).await;
        
        let conn1 = pool.acquire().await.unwrap();
        let conn2 = pool.acquire().await.unwrap();
        let conn3 = pool.acquire().await; // Should be None
        
        assert!(conn1.get().is_some());
        assert!(conn2.get().is_some());
        assert!(conn3.is_none());
    }
    
    #[tokio::test]
    async fn test_rate_limiter() {
        let rate_limiter = RateLimiter::new(2, Duration::from_millis(100));
        
        assert!(rate_limiter.check_rate_limit().await);
        assert!(rate_limiter.check_rate_limit().await);
        assert!(!rate_limiter.check_rate_limit().await); // Should be rate limited
    }
    
    #[tokio::test]
    async fn test_load_balancer() {
        let servers = vec!["server1".to_string(), "server2".to_string()];
        let lb = LoadBalancer::new(servers, LoadBalancingStrategy::RoundRobin);
        
        let server1 = lb.get_server().await.unwrap();
        let server2 = lb.get_server().await.unwrap();
        let server3 = lb.get_server().await.unwrap();
        
        assert_eq!(server1, "server1");
        assert_eq!(server2, "server2");
        assert_eq!(server3, "server1"); // Should wrap around
    }
}
