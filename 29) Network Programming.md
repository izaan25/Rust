# Network Programming in Rust

## Overview

Rust provides excellent support for network programming with its focus on safety, performance, and concurrency. This guide covers TCP/UDP programming, HTTP servers, web frameworks, and advanced networking patterns.

---

## Network Ecosystem

### Core Networking Crates

| Crate | Purpose | Features |
|-------|---------|----------|
| `tokio` | Async runtime | TCP/UDP, async I/O |
| `hyper` | HTTP client/server | HTTP/1.1, HTTP/2 |
| `reqwest` | HTTP client | Async HTTP requests |
| `axum` | Web framework | Built on tokio/hyper |
| `warp` | Web framework | Filter-based routing |
| `tonic` | gRPC framework | High-performance RPC |
| `quinn` | QUIC protocol | Modern transport layer |
| `socket2` | Raw sockets | Low-level socket control |

### Choosing the Right Tools

- **tokio** - Foundation for async networking
- **hyper** - Low-level HTTP implementation
- **axum/warp** - High-level web frameworks
- **reqwest** - Simple HTTP client
- **tonic** - gRPC and protobuf support

---

## TCP Programming

### TCP Server

```rust
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    
    println!("Server listening on 127.0.0.1:8080");
    
    loop {
        let (socket, addr) = listener.accept().await?;
        println!("New connection from: {}", addr);
        
        tokio::spawn(async move {
            if let Err(e) = handle_connection(socket, addr).await {
                eprintln!("Error handling connection {}: {}", addr, e);
            }
        });
    }
}

async fn handle_connection(mut socket: TcpStream, addr: std::net::SocketAddr) -> Result<(), Box<dyn Error>> {
    let mut buffer = [0; 1024];
    
    loop {
        let bytes_read = socket.read(&mut buffer).await?;
        
        if bytes_read == 0 {
            break;
        }
        
        let message = String::from_utf8_lossy(&buffer[..bytes_read]);
        println!("Received from {}: {}", addr, message.trim());
        
        let response = format!("Echo: {}", message.trim());
        socket.write_all(response.as_bytes()).await?;
        
        if message.trim() == "quit" {
            break;
        }
    }
    
    println!("Connection closed: {}", addr);
    Ok(())
}
```

### TCP Client

```rust
use tokio::net::TcpStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut stream = TcpStream::connect("127.0.0.1:8080").await?;
    
    // Send messages
    let messages = vec!["Hello", "World", "Rust", "quit"];
    
    for message in messages {
        stream.write_all(message.as_bytes()).await?;
        stream.write_all(b"\n").await?;
        
        // Read response
        let mut buffer = [0; 1024];
        let bytes_read = stream.read(&mut buffer).await?;
        
        let response = String::from_utf8_lossy(&buffer[..bytes_read]);
        println!("Server response: {}", response.trim());
    }
    
    Ok(())
}
```

---

## UDP Programming

### UDP Server

```rust
use tokio::net::UdpSocket;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let socket = UdpSocket::bind("127.0.0.1:8080").await?;
    println!("UDP Server listening on 127.0.0.1:8080");
    
    let mut buf = [0; 1024];
    
    loop {
        let (len, addr) = socket.recv_from(&mut buf).await?;
        let message = String::from_utf8_lossy(&buf[..len]);
        println!("Received from {}: {}", addr, message.trim());
        
        let response = format!("UDP Echo: {}", message.trim());
        socket.send_to(response.as_bytes(), addr).await?;
    }
}
```

### UDP Client

```rust
use tokio::net::UdpSocket;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let socket = UdpSocket::bind("127.0.0.1:0").await?;
    
    let messages = vec!["Hello UDP", "World UDP", "Rust UDP"];
    
    for message in messages {
        socket.send_to(message.as_bytes(), "127.0.0.1:8080").await?;
        
        // Read response
        let mut buf = [0; 1024];
        let len = socket.recv(&mut buf).await?;
        
        let response = String::from_utf8_lossy(&buf[..len]);
        println!("Server response: {}", response.trim());
    }
    
    Ok(())
}
```

---

## HTTP Server with Axum

### Basic Axum Server

```toml
[dependencies]
tokio = { version = "1.0", features = ["full"] }
axum = "0.6"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tower = "0.4"
tower-http = { version = "0.3", features = ["cors", "trace"] }
tracing = "0.1"
tracing-subscriber = "0.3"
```

```rust
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{Html, Json},
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Debug, Serialize, Deserialize)]
struct User {
    id: u32,
    name: String,
    email: String,
}

#[derive(Debug, Deserialize)]
struct CreateUser {
    name: String,
    email: String,
}

#[derive(Debug, Deserialize)]
struct UserQuery {
    limit: Option<usize>,
    offset: Option<usize>,
}

// Application state
struct AppState {
    users: Arc<Mutex<HashMap<u32, User>>>,
    next_id: Arc<Mutex<u32>>,
}

impl AppState {
    fn new() -> Self {
        let mut users = HashMap::new();
        users.insert(1, User {
            id: 1,
            name: "Alice".to_string(),
            email: "alice@example.com".to_string(),
        });
        
        AppState {
            users: Arc::new(Mutex::new(users)),
            next_id: Arc::new(Mutex::new(2)),
        }
    }
}

// Handlers
async fn get_users(
    Query(query): Query<UserQuery>,
    State(state): State<AppState>,
) -> Json<Vec<User>> {
    let users = state.users.lock().unwrap();
    let user_list: Vec<User> = users.values().cloned().collect();
    
    let limit = query.limit.unwrap_or(user_list.len());
    let offset = query.offset.unwrap_or(0);
    
    Json(user_list.into_iter().skip(offset).take(limit).collect())
}

async fn get_user(
    Path(user_id): Path<u32>,
    State(state): State<AppState>,
) -> Result<Json<User>, StatusCode> {
    let users = state.users.lock().unwrap();
    
    match users.get(&user_id) {
        Some(user) => Ok(Json(user.clone())),
        None => Err(StatusCode::NOT_FOUND),
    }
}

async fn create_user(
    State(state): State<AppState>,
    Json(new_user): Json<CreateUser>,
) -> Result<(StatusCode, Json<User>), StatusCode> {
    let mut users = state.users.lock().unwrap();
    let mut next_id = state.next_id.lock().unwrap();
    
    let user = User {
        id: *next_id,
        name: new_user.name,
        email: new_user.email,
    };
    
    users.insert(*next_id, user.clone());
    *next_id += 1;
    
    Ok((StatusCode::CREATED, Json(user)))
}

async fn delete_user(
    Path(user_id): Path<u32>,
    State(state): State<AppState>,
) -> StatusCode {
    let mut users = state.users.lock().unwrap();
    
    match users.remove(&user_id) {
        Some(_) => StatusCode::NO_CONTENT,
        None => StatusCode::NOT_FOUND,
    }
}

async fn health_check() -> &'static str {
    "OK"
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();
    
    let state = AppState::new();
    
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/users", get(get_users).post(create_user))
        .route("/users/:id", get(get_user).delete(delete_user))
        .with_state(state)
        .layer(
            tower::ServiceBuilder::new()
                .layer(tower_http::cors::CorsLayer::permissive())
                .layer(tower_http::trace::TraceLayer::new_for_http())
        );
    
    let addr = "127.0.0.1:3000";
    tracing::info!("Server listening on {}", addr);
    
    axum::Server::bind(&addr.parse()?)
        .serve(app.into_make_service())
        .await?;
    
    Ok(())
}
```

---

## HTTP Client with Reqwest

### Basic HTTP Client

```toml
[dependencies]
tokio = { version = "1.0", features = ["full"] }
reqwest = { version = "0.11", features = ["json"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

```rust
use reqwest;
use serde::{Deserialize, Serialize};
use std::error::Error;

#[derive(Debug, Serialize, Deserialize)]
struct Post {
    userId: u32,
    id: u32,
    title: String,
    body: String,
}

#[derive(Debug, Serialize)]
struct NewPost {
    title: String,
    body: String,
    userId: u32,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let client = reqwest::Client::new();
    
    // GET request
    println!("=== GET Request ===");
    let response = client
        .get("https://jsonplaceholder.typicode.com/posts/1")
        .send()
        .await?;
    
    let post: Post = response.json().await?;
    println!("Post: {:?}", post);
    
    // POST request
    println!("\n=== POST Request ===");
    let new_post = NewPost {
        title: "Rust is awesome".to_string(),
        body: "Learning Rust programming".to_string(),
        userId: 1,
    };
    
    let response = client
        .post("https://jsonplaceholder.typicode.com/posts")
        .json(&new_post)
        .send()
        .await?;
    
    let created_post: Post = response.json().await?;
    println!("Created post: {:?}", created_post);
    
    // PUT request
    println!("\n=== PUT Request ===");
    let updated_post = NewPost {
        title: "Updated title".to_string(),
        body: "Updated body".to_string(),
        userId: 1,
    };
    
    let response = client
        .put("https://jsonplaceholder.typicode.com/posts/1")
        .json(&updated_post)
        .send()
        .await?;
    
    let updated: Post = response.json().await?;
    println!("Updated post: {:?}", updated);
    
    // DELETE request
    println!("\n=== DELETE Request ===");
    let response = client
        .delete("https://jsonplaceholder.typicode.com/posts/1")
        .send()
        .await?;
    
    println!("Delete status: {}", response.status());
    
    Ok(())
}
```

### Advanced HTTP Client

```rust
use reqwest::{Client, Response};
use std::time::Duration;
use std::error::Error;

async fn create_client() -> Result<Client, Box<dyn Error>> {
    let client = Client::builder()
        .timeout(Duration::from_secs(10))
        .user_agent("Rust HTTP Client/1.0")
        .build()?;
    
    Ok(client)
}

async fn handle_response(response: Response) -> Result<(), Box<dyn Error>> {
    println!("Status: {}", response.status());
    println!("Headers: {:?}", response.headers());
    
    let body = response.text().await?;
    println!("Body: {}", body);
    
    Ok(())
}

async fn retry_request(
    client: &Client,
    url: &str,
    max_retries: u32,
) -> Result<Response, Box<dyn Error>> {
    let mut retries = 0;
    let mut delay = Duration::from_millis(100);
    
    loop {
        match client.get(url).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    return Ok(response);
                } else if retries >= max_retries {
                    return Err(format!("Request failed after {} retries", max_retries).into());
                }
            }
            Err(e) if retries >= max_retries => {
                return Err(Box::new(e));
            }
            Err(_) => {
                // Retry on network errors
            }
        }
        
        retries += 1;
        tokio::time::sleep(delay).await;
        delay *= 2; // Exponential backoff
    }
}
```

---

## WebSocket Programming

### WebSocket Server

```toml
[dependencies]
tokio = { version = "1.0", features = ["full"] }
tokio-tungstenite = "0.20"
futures-util = "0.3"
url = "2.0"
```

```rust
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::{accept_async, tungstenite::Message};
use futures_util::{SinkExt, StreamExt};
use std::error::Error;
use url::Url;

async fn handle_websocket(socket: TcpStream) -> Result<(), Box<dyn Error>> {
    let ws_stream = accept_async(socket).await?;
    let (mut ws_sender, mut ws_receiver) = ws_stream.split();
    
    println!("WebSocket connection established");
    
    while let Some(msg) = ws_receiver.next().await {
        match msg? {
            Message::Text(text) => {
                println!("Received: {}", text);
                
                let response = format!("Echo: {}", text);
                ws_sender.send(Message::Text(response)).await?;
            }
            Message::Binary(data) => {
                println!("Received binary data: {} bytes", data.len());
                ws_sender.send(Message::Binary(data)).await?;
            }
            Message::Close(_) => {
                println!("WebSocket connection closed");
                break;
            }
            Message::Ping(data) => {
                ws_sender.send(Message::Pong(data)).await?;
            }
            Message::Pong(_) => {
                // Handle pong
            }
        }
    }
    
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    println!("WebSocket server listening on ws://127.0.0.1:8080");
    
    loop {
        let (socket, addr) = listener.accept().await?;
        println!("New WebSocket connection from: {}", addr);
        
        tokio::spawn(async move {
            if let Err(e) = handle_websocket(socket).await {
                eprintln!("WebSocket error: {}", e);
            }
        });
    }
}
```

### WebSocket Client

```rust
use tokio_tungstenite::{connect_async, tungstenite::Message};
use futures_util::{SinkExt, StreamExt};
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let url = Url::parse("ws://127.0.0.1:8080")?;
    let (ws_stream, _) = connect_async(url).await?;
    
    let (mut ws_sender, mut ws_receiver) = ws_stream.split();
    
    // Send messages
    let messages = vec!["Hello WebSocket", "Rust is cool", "Goodbye"];
    
    for message in messages {
        ws_sender.send(Message::Text(message.to_string())).await?;
        
        // Read response
        if let Some(msg) = ws_receiver.next().await {
            match msg? {
                Message::Text(text) => println!("Echo: {}", text),
                _ => {}
            }
        }
    }
    
    // Close connection
    ws_sender.send(Message::Close(None)).await?;
    
    Ok(())
}
```

---

## gRPC with Tonic

### gRPC Service Definition

```protobuf
syntax = "proto3";

package user;

service UserService {
    rpc GetUser(GetUserRequest) returns (GetUserResponse);
    rpc ListUsers(ListUsersRequest) returns (ListUsersResponse);
    rpc CreateUser(CreateUserRequest) returns (CreateUserResponse);
}

message GetUserRequest {
    uint32 id = 1;
}

message GetUserResponse {
    User user = 1;
}

message ListUsersRequest {
    uint32 limit = 1;
    uint32 offset = 2;
}

message ListUsersResponse {
    repeated User users = 1;
}

message CreateUserRequest {
    string name = 1;
    string email = 2;
}

message CreateUserResponse {
    User user = 1;
}

message User {
    uint32 id = 1;
    string name = 2;
    string email = 3;
}
```

### gRPC Server

```toml
[dependencies]
tokio = { version = "1.0", features = ["full"] }
tonic = "0.8"
prost = "0.11"
```

```rust
use tonic::{transport::Server, Request, Response, Status};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub mod user {
    tonic::include_proto!("user");
}

use user::{
    user_service_server::{UserService, UserServiceServer},
    *,
};

#[derive(Debug, Default)]
struct UserServiceImpl {
    users: Arc<Mutex<HashMap<u32, User>>>,
    next_id: Arc<Mutex<u32>>,
}

#[tonic::async_trait]
impl UserService for UserServiceImpl {
    async fn get_user(
        &self,
        request: Request<GetUserRequest>,
    ) -> Result<Response<GetUserResponse>, Status> {
        let req = request.into_inner();
        let users = self.users.lock().unwrap();
        
        match users.get(&req.id) {
            Some(user) => {
                let response = GetUserResponse { user: Some(user.clone()) };
                Ok(Response::new(response))
            }
            None => Err(Status::not_found("User not found")),
        }
    }
    
    async fn list_users(
        &self,
        request: Request<ListUsersRequest>,
    ) -> Result<Response<ListUsersResponse>, Status> {
        let req = request.into_inner();
        let users = self.users.lock().unwrap();
        
        let user_list: Vec<User> = users.values()
            .skip(req.offset as usize)
            .take(req.limit as usize)
            .cloned()
            .collect();
        
        let response = ListUsersResponse { users: user_list };
        Ok(Response::new(response))
    }
    
    async fn create_user(
        &self,
        request: Request<CreateUserRequest>,
    ) -> Result<Response<CreateUserResponse>, Status> {
        let req = request.into_inner();
        let mut users = self.users.lock().unwrap();
        let mut next_id = self.next_id.lock().unwrap();
        
        let user = User {
            id: *next_id,
            name: req.name,
            email: req.email,
        };
        
        users.insert(*next_id, user.clone());
        *next_id += 1;
        
        let response = CreateUserResponse { user: Some(user) };
        Ok(Response::new(response))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "127.0.0.1:50051".parse()?;
    
    let user_service = UserServiceImpl::default();
    
    println!("gRPC server listening on {}", addr);
    
    Server::builder()
        .add_service(UserServiceServer::new(user_service))
        .serve(addr)
        .await?;
    
    Ok(())
}
```

---

## Advanced Networking Patterns

### Connection Pool

```rust
use std::sync::Arc;
use tokio::sync::{Mutex, Semaphore};
use std::collections::VecDeque;

struct ConnectionPool<T> {
    connections: Arc<Mutex<VecDeque<T>>>,
    semaphore: Arc<Semaphore>,
    max_size: usize,
}

impl<T> ConnectionPool<T> {
    fn new(max_size: usize) -> Self {
        ConnectionPool {
            connections: Arc::new(Mutex::new(VecDeque::new())),
            semaphore: Arc::new(Semaphore::new(max_size)),
            max_size,
        }
    }
    
    async fn acquire<F, Fut>(&self, create_connection: F) -> Result<PooledConnection<T>, Box<dyn std::error::Error>>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<T, Box<dyn std::error::Error>>>,
    {
        let _permit = self.semaphore.acquire().await?;
        
        let mut connections = self.connections.lock().await;
        
        if let Some(conn) = connections.pop_front() {
            Ok(PooledConnection {
                connection: Some(conn),
                pool: self.connections.clone(),
                _permit: self.semaphore.clone(),
            })
        } else {
            drop(connections);
            let conn = create_connection().await?;
            Ok(PooledConnection {
                connection: Some(conn),
                pool: self.connections.clone(),
                _permit: self.semaphore.clone(),
            })
        }
    }
}

struct PooledConnection<T> {
    connection: Option<T>,
    pool: Arc<Mutex<VecDeque<T>>>,
    _permit: Arc<Semaphore>,
}

impl<T> PooledConnection<T> {
    fn get_mut(&mut self) -> &mut T {
        self.connection.as_mut().unwrap()
    }
}

impl<T> Drop for PooledConnection<T> {
    fn drop(&mut self) {
        if let Some(conn) = self.connection.take() {
            let pool = self.pool.clone();
            tokio::spawn(async move {
                let mut connections = pool.lock().await;
                connections.push_back(conn);
            });
        }
    }
}
```

### Rate Limiting

```rust
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

struct RateLimiter {
    max_requests: u32,
    window: Duration,
    requests: Arc<Mutex<Vec<Instant>>>,
}

impl RateLimiter {
    fn new(max_requests: u32, window: Duration) -> Self {
        RateLimiter {
            max_requests,
            window,
            requests: Arc::new(Mutex::new(Vec::new())),
        }
    }
    
    async fn check_rate_limit(&self) -> bool {
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
}

async fn rate_limited_handler(limiter: Arc<RateLimiter>) -> Result<String, &'static str> {
    if limiter.check_rate_limit().await {
        Ok("Request processed".to_string())
    } else {
        Err("Rate limit exceeded")
    }
}
```

### Load Balancer

```rust
use std::sync::Arc;
use std::collections::VecDeque;
use tokio::sync::Mutex;

enum LoadBalancingStrategy {
    RoundRobin,
    Random,
    LeastConnections,
}

struct LoadBalancer {
    servers: Arc<Mutex<VecDeque<String>>>,
    strategy: LoadBalancingStrategy,
    current_index: Arc<Mutex<usize>>,
}

impl LoadBalancer {
    fn new(servers: Vec<String>, strategy: LoadBalancingStrategy) -> Self {
        LoadBalancer {
            servers: Arc::new(Mutex::new(VecDeque::from(servers))),
            strategy,
            current_index: Arc::new(Mutex::new(0)),
        }
    }
    
    async fn get_server(&self) -> Option<String> {
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
                // Simplified - would need connection tracking
                servers[0].clone()
            }
        };
        
        Some(server)
    }
    
    async fn add_server(&self, server: String) {
        let mut servers = self.servers.lock().await;
        servers.push_back(server);
    }
    
    async fn remove_server(&self, server: &str) -> bool {
        let mut servers = self.servers.lock().await;
        if let Some(pos) = servers.iter().position(|s| s == server) {
            servers.remove(pos);
            true
        } else {
            false
        }
    }
}
```

---

## Key Takeaways

- **tokio** provides the foundation for async networking in Rust
- **TCP** is connection-oriented and reliable
- **UDP** is connectionless and fast
- **HTTP/HTTPS** are built on TCP with web frameworks
- **WebSockets** enable real-time bidirectional communication
- **gRPC** provides high-performance RPC with protobuf
- **Connection pooling** improves performance and resource utilization
- **Rate limiting** prevents abuse and ensures fair usage
- **Load balancing** distributes traffic across multiple servers

---

## Network Best Practices

| Practice | Description | Implementation |
|----------|-------------|----------------|
| **Error Handling** | Handle network errors gracefully | Use Result types and proper error propagation |
| **Timeouts** | Prevent hanging connections | Set appropriate timeouts for all operations |
| **Connection Pooling** | Reuse connections efficiently | Implement connection pools for database/HTTP clients |
| **Rate Limiting** | Prevent abuse and DoS attacks | Use token bucket or sliding window algorithms |
| **Security** | Encrypt and authenticate connections | Use TLS/SSL and proper authentication |
| **Monitoring** | Track network performance | Add metrics and logging for network operations |
| **Graceful Shutdown** | Handle shutdown properly | Use signal handling and connection draining |
