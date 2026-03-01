// 28_database_programming.rs
// Comprehensive examples of database programming in Rust

// Note: This file demonstrates database concepts but requires proper database setup
// and connection strings to run actual database operations

use std::env;
use std::time::Duration;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use async_trait::async_trait;
use thiserror::Error;

// =========================================
// MODELS AND TYPES
// =========================================

#[derive(Debug, Clone)]
pub struct User {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct Post {
    pub id: Uuid,
    pub title: String,
    pub content: Option<String>,
    pub user_id: Uuid,
    pub published_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct NewUser {
    pub name: String,
    pub email: String,
}

#[derive(Debug, Clone)]
pub struct NewPost {
    pub title: String,
    pub content: Option<String>,
    pub user_id: Uuid,
}

#[derive(Debug, Clone)]
pub struct Account {
    pub id: i64,
    pub user_id: Uuid,
    pub balance: f64,
    pub created_at: DateTime<Utc>,
}

// =========================================
// ERROR HANDLING
// =========================================

#[derive(Error, Debug)]
pub enum DatabaseError {
    #[error("Record not found")]
    NotFound,
    
    #[error("Duplicate record: {0}")]
    Duplicate(String),
    
    #[error("Connection error: {0}")]
    ConnectionError(String),
    
    #[error("Validation error: {0}")]
    ValidationError(String),
    
    #[error("Migration error: {0}")]
    MigrationError(String),
    
    #[error("Transaction error: {0}")]
    TransactionError(String),
}

pub type DbResult<T> = Result<T, DatabaseError>;

// =========================================
// SIMULATED DATABASE CONNECTION
// =========================================

// Simulated database connection for demonstration
pub struct SimulatedDatabase {
    users: std::collections::HashMap<Uuid, User>,
    posts: std::collections::HashMap<Uuid, Post>,
    accounts: std::collections::HashMap<i64, Account>,
    next_account_id: i64,
}

impl SimulatedDatabase {
    pub fn new() -> Self {
        SimulatedDatabase {
            users: std::collections::HashMap::new(),
            posts: std::collections::HashMap::new(),
            accounts: std::collections::HashMap::new(),
            next_account_id: 1,
        }
    }
    
    pub fn create_user(&mut self, new_user: NewUser) -> DbResult<User> {
        // Check for duplicate email
        for user in self.users.values() {
            if user.email == new_user.email {
                return Err(DatabaseError::Duplicate("Email already exists".to_string()));
            }
        }
        
        let user = User {
            id: Uuid::new_v4(),
            name: new_user.name,
            email: new_user.email,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        
        self.users.insert(user.id, user.clone());
        Ok(user)
    }
    
    pub fn get_user_by_id(&self, user_id: Uuid) -> DbResult<User> {
        self.users.get(&user_id)
            .cloned()
            .ok_or(DatabaseError::NotFound)
    }
    
    pub fn get_user_by_email(&self, email: &str) -> DbResult<User> {
        self.users.values()
            .find(|user| user.email == email)
            .cloned()
            .ok_or(DatabaseError::NotFound)
    }
    
    pub fn update_user(&mut self, user: User) -> DbResult<User> {
        if !self.users.contains_key(&user.id) {
            return Err(DatabaseError::NotFound);
        }
        
        let mut updated_user = user;
        updated_user.updated_at = Utc::now();
        self.users.insert(updated_user.id, updated_user.clone());
        Ok(updated_user)
    }
    
    pub fn delete_user(&mut self, user_id: Uuid) -> DbResult<bool> {
        let existed = self.users.remove(&user_id).is_some();
        
        // Also delete user's posts
        self.posts.retain(|_, post| post.user_id != user_id);
        
        Ok(existed)
    }
    
    pub fn create_post(&mut self, new_post: NewPost) -> DbResult<Post> {
        // Check if user exists
        if !self.users.contains_key(&new_post.user_id) {
            return Err(DatabaseError::ValidationError("User not found".to_string()));
        }
        
        let post = Post {
            id: Uuid::new_v4(),
            title: new_post.title,
            content: new_post.content,
            user_id: new_post.user_id,
            published_at: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        
        self.posts.insert(post.id, post.clone());
        Ok(post)
    }
    
    pub fn get_posts_by_user(&self, user_id: Uuid) -> DbResult<Vec<Post>> {
        let posts: Vec<Post> = self.posts.values()
            .filter(|post| post.user_id == user_id)
            .cloned()
            .collect();
        
        Ok(posts)
    }
    
    pub fn get_published_posts(&self) -> DbResult<Vec<Post>> {
        let posts: Vec<Post> = self.posts.values()
            .filter(|post| post.published_at.is_some())
            .cloned()
            .collect();
        
        Ok(posts)
    }
    
    pub fn publish_post(&mut self, post_id: Uuid) -> DbResult<Post> {
        let post = self.posts.get_mut(&post_id)
            .ok_or(DatabaseError::NotFound)?;
        
        post.published_at = Some(Utc::now());
        post.updated_at = Utc::now();
        
        Ok(post.clone())
    }
    
    pub fn create_account(&mut self, user_id: Uuid, initial_balance: f64) -> DbResult<Account> {
        // Check if user exists
        if !self.users.contains_key(&user_id) {
            return Err(DatabaseError::ValidationError("User not found".to_string()));
        }
        
        let account = Account {
            id: self.next_account_id,
            user_id,
            balance: initial_balance,
            created_at: Utc::now(),
        };
        
        self.accounts.insert(account.id, account.clone());
        self.next_account_id += 1;
        
        Ok(account)
    }
    
    pub fn get_account(&self, account_id: i64) -> DbResult<Account> {
        self.accounts.get(&account_id)
            .cloned()
            .ok_or(DatabaseError::NotFound)
    }
    
    pub fn update_account_balance(&mut self, account_id: i64, new_balance: f64) -> DbResult<Account> {
        let account = self.accounts.get_mut(&account_id)
            .ok_or(DatabaseError::NotFound)?;
        
        account.balance = new_balance;
        Ok(account.clone())
    }
}

// =========================================
// REPOSITORY PATTERN
// =========================================

#[async_trait]
pub trait UserRepository {
    async fn create(&self, user: NewUser) -> DbResult<User>;
    async fn find_by_id(&self, id: Uuid) -> DbResult<Option<User>>;
    async fn find_by_email(&self, email: &str) -> DbResult<Option<User>>;
    async fn update(&self, user: &User) -> DbResult<User>;
    async fn delete(&self, id: Uuid) -> DbResult<bool>;
    async fn list(&self, limit: u32, offset: u32) -> DbResult<Vec<User>>;
}

#[async_trait]
pub trait PostRepository {
    async fn create(&self, post: NewPost) -> DbResult<Post>;
    async fn find_by_id(&self, id: Uuid) -> DbResult<Option<Post>>;
    async fn find_by_user(&self, user_id: Uuid) -> DbResult<Vec<Post>>;
    async fn find_published(&self) -> DbResult<Vec<Post>>;
    async fn publish(&self, post_id: Uuid) -> DbResult<Post>;
    async fn delete(&self, id: Uuid) -> DbResult<bool>;
}

#[async_trait]
pub trait AccountRepository {
    async fn create(&self, user_id: Uuid, initial_balance: f64) -> DbResult<Account>;
    async fn find_by_id(&self, id: i64) -> DbResult<Option<Account>>;
    async fn update_balance(&self, id: i64, balance: f64) -> DbResult<Account>;
    async fn find_by_user(&self, user_id: Uuid) -> DbResult<Vec<Account>>;
}

// Simulated repository implementations
pub struct SimulatedUserRepository {
    db: std::sync::Arc<tokio::sync::Mutex<SimulatedDatabase>>,
}

impl SimulatedUserRepository {
    pub fn new(db: std::sync::Arc<tokio::sync::Mutex<SimulatedDatabase>>) -> Self {
        SimulatedUserRepository { db }
    }
}

#[async_trait]
impl UserRepository for SimulatedUserRepository {
    async fn create(&self, user: NewUser) -> DbResult<User> {
        let mut db = self.db.lock().await;
        db.create_user(user)
    }
    
    async fn find_by_id(&self, id: Uuid) -> DbResult<Option<User>> {
        let db = self.db.lock().await;
        match db.get_user_by_id(id) {
            Ok(user) => Ok(Some(user)),
            Err(DatabaseError::NotFound) => Ok(None),
            Err(e) => Err(e),
        }
    }
    
    async fn find_by_email(&self, email: &str) -> DbResult<Option<User>> {
        let db = self.db.lock().await;
        match db.get_user_by_email(email) {
            Ok(user) => Ok(Some(user)),
            Err(DatabaseError::NotFound) => Ok(None),
            Err(e) => Err(e),
        }
    }
    
    async fn update(&self, user: &User) -> DbResult<User> {
        let mut db = self.db.lock().await;
        db.update_user(user.clone())
    }
    
    async fn delete(&self, id: Uuid) -> DbResult<bool> {
        let mut db = self.db.lock().await;
        db.delete_user(id)
    }
    
    async fn list(&self, limit: u32, offset: u32) -> DbResult<Vec<User>> {
        let db = self.db.lock().await;
        let users: Vec<User> = db.users.values()
            .skip(offset as usize)
            .take(limit as usize)
            .cloned()
            .collect();
        Ok(users)
    }
}

pub struct SimulatedPostRepository {
    db: std::sync::Arc<tokio::sync::Mutex<SimulatedDatabase>>,
}

impl SimulatedPostRepository {
    pub fn new(db: std::sync::Arc<tokio::sync::Mutex<SimulatedDatabase>>) -> Self {
        SimulatedPostRepository { db }
    }
}

#[async_trait]
impl PostRepository for SimulatedPostRepository {
    async fn create(&self, post: NewPost) -> DbResult<Post> {
        let mut db = self.db.lock().await;
        db.create_post(post)
    }
    
    async fn find_by_id(&self, id: Uuid) -> DbResult<Option<Post>> {
        let db = self.db.lock().await;
        match db.posts.get(&id) {
            Some(post) => Ok(Some(post.clone())),
            None => Ok(None),
        }
    }
    
    async fn find_by_user(&self, user_id: Uuid) -> DbResult<Vec<Post>> {
        let db = self.db.lock().await;
        db.get_posts_by_user(user_id)
    }
    
    async fn find_published(&self) -> DbResult<Vec<Post>> {
        let db = self.db.lock().await;
        db.get_published_posts()
    }
    
    async fn publish(&self, post_id: Uuid) -> DbResult<Post> {
        let mut db = self.db.lock().await;
        db.publish_post(post_id)
    }
    
    async fn delete(&self, id: Uuid) -> DbResult<bool> {
        let mut db = self.db.lock().await;
        let existed = db.posts.remove(&id).is_some();
        Ok(existed)
    }
}

pub struct SimulatedAccountRepository {
    db: std::sync::Arc<tokio::sync::Mutex<SimulatedDatabase>>,
}

impl SimulatedAccountRepository {
    pub fn new(db: std::sync::Arc<tokio::sync::Mutex<SimulatedDatabase>>) -> Self {
        SimulatedAccountRepository { db }
    }
}

#[async_trait]
impl AccountRepository for SimulatedAccountRepository {
    async fn create(&self, user_id: Uuid, initial_balance: f64) -> DbResult<Account> {
        let mut db = self.db.lock().await;
        db.create_account(user_id, initial_balance)
    }
    
    async fn find_by_id(&self, id: i64) -> DbResult<Option<Account>> {
        let db = self.db.lock().await;
        match db.get_account(id) {
            Ok(account) => Ok(Some(account)),
            Err(DatabaseError::NotFound) => Ok(None),
            Err(e) => Err(e),
        }
    }
    
    async fn update_balance(&self, id: i64, balance: f64) -> DbResult<Account> {
        let mut db = self.db.lock().await;
        db.update_account_balance(id, balance)
    }
    
    async fn find_by_user(&self, user_id: Uuid) -> DbResult<Vec<Account>> {
        let db = self.db.lock().await;
        let accounts: Vec<Account> = db.accounts.values()
            .filter(|account| account.user_id == user_id)
            .cloned()
            .collect();
        Ok(accounts)
    }
}

// =========================================
// UNIT OF WORK PATTERN
// =========================================

pub struct UnitOfWork {
    db: std::sync::Arc<tokio::sync::Mutex<SimulatedDatabase>>,
    user_repo: SimulatedUserRepository,
    post_repo: SimulatedPostRepository,
    account_repo: SimulatedAccountRepository,
    committed: bool,
}

impl UnitOfWork {
    pub fn new(db: std::sync::Arc<tokio::sync::Mutex<SimulatedDatabase>>) -> Self {
        let user_repo = SimulatedUserRepository::new(db.clone());
        let post_repo = SimulatedPostRepository::new(db.clone());
        let account_repo = SimulatedAccountRepository::new(db.clone());
        
        UnitOfWork {
            db,
            user_repo,
            post_repo,
            account_repo,
            committed: false,
        }
    }
    
    pub fn user_repository(&self) -> &dyn UserRepository {
        &self.user_repo
    }
    
    pub fn post_repository(&self) -> &dyn PostRepository {
        &self.post_repo
    }
    
    pub fn account_repository(&self) -> &dyn AccountRepository {
        &self.account_repo
    }
    
    pub async fn commit(mut self) -> DbResult<()> {
        // In a real database, this would commit the transaction
        self.committed = true;
        Ok(())
    }
    
    pub async fn rollback(self) -> DbResult<()> {
        // In a real database, this would rollback the transaction
        if !self.committed {
            println!("Transaction rolled back");
        }
        Ok(())
    }
}

// =========================================
// SERVICE LAYER
// =========================================

pub struct UserService {
    user_repo: Box<dyn UserRepository + Send + Sync>,
}

impl UserService {
    pub fn new(user_repo: Box<dyn UserRepository + Send + Sync>) -> Self {
        UserService { user_repo }
    }
    
    pub async fn create_user(&self, name: String, email: String) -> DbResult<User> {
        // Validate input
        if name.trim().is_empty() {
            return Err(DatabaseError::ValidationError("Name cannot be empty".to_string()));
        }
        
        if !email.contains('@') {
            return Err(DatabaseError::ValidationError("Invalid email format".to_string()));
        }
        
        let new_user = NewUser { name, email };
        self.user_repo.create(new_user).await
    }
    
    pub async fn get_user(&self, id: Uuid) -> DbResult<User> {
        self.user_repo.find_by_id(id).await?
            .ok_or(DatabaseError::NotFound)
    }
    
    pub async fn update_user(&self, user: User) -> DbResult<User> {
        self.user_repo.update(&user).await
    }
    
    pub async fn delete_user(&self, id: Uuid) -> DbResult<bool> {
        self.user_repo.delete(id).await
    }
}

pub struct PostService {
    post_repo: Box<dyn PostRepository + Send + Sync>,
    user_repo: Box<dyn UserRepository + Send + Sync>,
}

impl PostService {
    pub fn new(
        post_repo: Box<dyn PostRepository + Send + Sync>,
        user_repo: Box<dyn UserRepository + Send + Sync>,
    ) -> Self {
        PostService { post_repo, user_repo }
    }
    
    pub async fn create_post(&self, title: String, content: Option<String>, user_id: Uuid) -> DbResult<Post> {
        // Validate user exists
        self.user_repo.find_by_id(user_id).await?
            .ok_or(DatabaseError::ValidationError("User not found".to_string()))?;
        
        // Validate input
        if title.trim().is_empty() {
            return Err(DatabaseError::ValidationError("Title cannot be empty".to_string()));
        }
        
        let new_post = NewPost {
            title,
            content,
            user_id,
        };
        
        self.post_repo.create(new_post).await
    }
    
    pub async fn publish_post(&self, post_id: Uuid) -> DbResult<Post> {
        self.post_repo.publish(post_id).await
    }
    
    pub async fn get_user_posts(&self, user_id: Uuid) -> DbResult<Vec<Post>> {
        self.post_repo.find_by_user(user_id).await
    }
    
    pub async fn get_published_posts(&self) -> DbResult<Vec<Post>> {
        self.post_repo.find_published().await
    }
}

pub struct AccountService {
    account_repo: Box<dyn AccountRepository + Send + Sync>,
    user_repo: Box<dyn UserRepository + Send + Sync>,
}

impl AccountService {
    pub fn new(
        account_repo: Box<dyn AccountRepository + Send + Sync>,
        user_repo: Box<dyn UserRepository + Send + Sync>,
    ) -> Self {
        AccountService { account_repo, user_repo }
    }
    
    pub async fn create_account(&self, user_id: Uuid, initial_balance: f64) -> DbResult<Account> {
        // Validate user exists
        self.user_repo.find_by_id(user_id).await?
            .ok_or(DatabaseError::ValidationError("User not found".to_string()))?;
        
        // Validate balance
        if initial_balance < 0.0 {
            return Err(DatabaseError::ValidationError("Initial balance cannot be negative".to_string()));
        }
        
        self.account_repo.create(user_id, initial_balance).await
    }
    
    pub async fn transfer_funds(&self, from_account_id: i64, to_account_id: i64, amount: f64) -> DbResult<()> {
        if amount <= 0.0 {
            return Err(DatabaseError::ValidationError("Transfer amount must be positive".to_string()));
        }
        
        // Get accounts
        let from_account = self.account_repo.find_by_id(from_account_id).await?
            .ok_or(DatabaseError::ValidationError("Source account not found".to_string()))?;
        
        let to_account = self.account_repo.find_by_id(to_account_id).await?
            .ok_or(DatabaseError::ValidationError("Destination account not found".to_string()))?;
        
        // Check sufficient funds
        if from_account.balance < amount {
            return Err(DatabaseError::ValidationError("Insufficient funds".to_string()));
        }
        
        // Update balances
        let new_from_balance = from_account.balance - amount;
        let new_to_balance = to_account.balance + amount;
        
        self.account_repo.update_balance(from_account_id, new_from_balance).await?;
        self.account_repo.update_balance(to_account_id, new_to_balance).await?;
        
        Ok(())
    }
    
    pub async fn get_account_balance(&self, account_id: i64) -> DbResult<f64> {
        let account = self.account_repo.find_by_id(account_id).await?
            .ok_or(DatabaseError::NotFound)?;
        Ok(account.balance)
    }
}

// =========================================
// CONNECTION POOL SIMULATION
// =========================================

pub struct ConnectionPool {
    connections: std::sync::Arc<tokio::sync::Mutex<Vec<SimulatedDatabase>>>,
    max_size: usize,
}

impl ConnectionPool {
    pub fn new(max_size: usize) -> Self {
        let mut connections = Vec::with_capacity(max_size);
        
        for _ in 0..max_size {
            connections.push(SimulatedDatabase::new());
        }
        
        ConnectionPool {
            connections: std::sync::Arc::new(tokio::sync::Mutex::new(connections)),
            max_size,
        }
    }
    
    pub async fn acquire(&self) -> PooledConnection {
        let mut connections = self.connections.lock().await;
        
        match connections.pop() {
            Some(db) => PooledConnection {
                db: Some(db),
                pool: self.connections.clone(),
            },
            None => {
                // Create new connection if pool is empty
                PooledConnection {
                    db: Some(SimulatedDatabase::new()),
                    pool: self.connections.clone(),
                }
            }
        }
    }
}

pub struct PooledConnection {
    db: Option<SimulatedDatabase>,
    pool: std::sync::Arc<tokio::sync::Mutex<Vec<SimulatedDatabase>>>,
}

impl PooledConnection {
    pub fn as_ref(&self) -> &SimulatedDatabase {
        self.db.as_ref().unwrap()
    }
    
    pub fn as_mut(&mut self) -> &mut SimulatedDatabase {
        self.db.as_mut().unwrap()
    }
}

impl Drop for PooledConnection {
    fn drop(&mut self) {
        if let Some(db) = self.db.take() {
            let pool = self.pool.clone();
            tokio::spawn(async move {
                let mut connections = pool.lock().await;
                connections.push(db);
            });
        }
    }
}

// =========================================
// DEMONSTRATION FUNCTIONS
// =========================================

pub async fn demonstrate_basic_operations() -> DbResult<()> {
    println!("=== BASIC DATABASE OPERATIONS ===");
    
    let db = std::sync::Arc::new(tokio::sync::Mutex::new(SimulatedDatabase::new()));
    let user_repo = Box::new(SimulatedUserRepository::new(db.clone()));
    let post_repo = Box::new(SimulatedPostRepository::new(db.clone()));
    let account_repo = Box::new(SimulatedAccountRepository::new(db.clone()));
    
    // Create users
    let user1 = user_repo.create(NewUser {
        name: "Alice".to_string(),
        email: "alice@example.com".to_string(),
    }).await?;
    
    let user2 = user_repo.create(NewUser {
        name: "Bob".to_string(),
        email: "bob@example.com".to_string(),
    }).await?;
    
    println!("Created users: {} and {}", user1.name, user2.name);
    
    // Create posts
    let post1 = post_repo.create(NewPost {
        title: "First Post".to_string(),
        content: Some("This is my first post".to_string()),
        user_id: user1.id,
    }).await?;
    
    let post2 = post_repo.create(NewPost {
        title: "Second Post".to_string(),
        content: Some("This is my second post".to_string()),
        user_id: user1.id,
    }).await?;
    
    println!("Created posts: {} and {}", post1.title, post2.title);
    
    // Create accounts
    let account1 = account_repo.create(user1.id, 1000.0).await?;
    let account2 = account_repo.create(user2.id, 500.0).await?;
    
    println!("Created accounts with balances: {} and {}", account1.balance, account2.balance);
    
    // Publish a post
    let published_post = post_repo.publish(post1.id).await?;
    println!("Published post: {}", published_post.title);
    
    // Get published posts
    let published_posts = post_repo.find_published().await?;
    println!("Published posts count: {}", published_posts.len());
    
    Ok(())
}

pub async fn demonstrate_repository_pattern() -> DbResult<()> {
    println!("\n=== REPOSITORY PATTERN ===");
    
    let db = std::sync::Arc::new(tokio::sync::Mutex::new(SimulatedDatabase::new()));
    let user_repo = Box::new(SimulatedUserRepository::new(db.clone()));
    
    let user_service = UserService::new(user_repo);
    
    // Create user through service
    let user = user_service.create_user(
        "Charlie".to_string(),
        "charlie@example.com".to_string(),
    ).await?;
    
    println!("Created user through service: {}", user.name);
    
    // Get user
    let retrieved_user = user_service.get_user(user.id).await?;
    println!("Retrieved user: {}", retrieved_user.name);
    
    // Update user
    let mut updated_user = retrieved_user;
    updated_user.name = "Charlie Brown".to_string();
    let updated_user = user_service.update_user(updated_user).await?;
    println!("Updated user name: {}", updated_user.name);
    
    Ok(())
}

pub async fn demonstrate_unit_of_work() -> DbResult<()> {
    println!("\n=== UNIT OF WORK PATTERN ===");
    
    let db = std::sync::Arc::new(tokio::sync::Mutex::new(SimulatedDatabase::new()));
    let uow = UnitOfWork::new(db.clone());
    
    // Create user and posts in a single unit of work
    let user = uow.user_repository().create(NewUser {
        name: "David".to_string(),
        email: "david@example.com".to_string(),
    }).await?;
    
    let post1 = uow.post_repository().create(NewPost {
        title: "Post 1".to_string(),
        content: Some("Content 1".to_string()),
        user_id: user.id,
    }).await?;
    
    let post2 = uow.post_repository().create(NewPost {
        title: "Post 2".to_string(),
        content: Some("Content 2".to_string()),
        user_id: user.id,
    }).await?;
    
    println!("Created user with {} posts", 2);
    
    // Commit the transaction
    uow.commit().await?;
    println!("Transaction committed");
    
    // Verify data exists
    let user_repo = SimulatedUserRepository::new(db);
    let retrieved_user = user_repo.find_by_id(user.id).await?;
    assert!(retrieved_user.is_some());
    
    Ok(())
}

pub async fn demonstrate_connection_pool() -> DbResult<()> {
    println!("\n=== CONNECTION POOL ===");
    
    let pool = ConnectionPool::new(5);
    
    // Acquire connections
    let mut conn1 = pool.acquire().await;
    let mut conn2 = pool.acquire().await;
    let mut conn3 = pool.acquire().await;
    
    // Use connections
    let user1 = conn1.as_mut().create_user(NewUser {
        name: "Eve".to_string(),
        email: "eve@example.com".to_string(),
    })?;
    
    let user2 = conn2.as_mut().create_user(NewUser {
        name: "Frank".to_string(),
        email: "frank@example.com".to_string(),
    })?;
    
    let user3 = conn3.as_mut().create_user(NewUser {
        name: "Grace".to_string(),
        email: "grace@example.com".to_string(),
    })?;
    
    println!("Created users with pool: {}, {}, {}", user1.name, user2.name, user3.name);
    
    // Connections are returned to pool when dropped
    drop(conn1);
    drop(conn2);
    drop(conn3);
    
    println!("Connections returned to pool");
    
    Ok(())
}

pub async fn demonstrate_service_layer() -> DbResult<()> {
    println!("\n=== SERVICE LAYER ===");
    
    let db = std::sync::Arc::new(tokio::sync::Mutex::new(SimulatedDatabase::new()));
    let user_repo = Box::new(SimulatedUserRepository::new(db.clone()));
    let post_repo = Box::new(SimulatedPostRepository::new(db.clone()));
    let account_repo = Box::new(SimulatedAccountRepository::new(db.clone()));
    
    let user_service = UserService::new(user_repo);
    let post_service = PostService::new(post_repo, Box::new(SimulatedUserRepository::new(db.clone())));
    let account_service = AccountService::new(account_repo, Box::new(SimulatedUserRepository::new(db.clone())));
    
    // Create user and account
    let user = user_service.create_user(
        "Henry".to_string(),
        "henry@example.com".to_string(),
    ).await?;
    
    let account = account_service.create_account(user.id, 1000.0).await?;
    println!("Created user {} with account balance {}", user.name, account.balance);
    
    // Create posts
    let post = post_service.create_post(
        "Henry's First Post".to_string(),
        Some("Hello world!".to_string()),
        user.id,
    ).await?;
    
    println!("Created post: {}", post.title);
    
    // Publish post
    let published_post = post_service.publish_post(post.id).await?;
    println!("Published post: {}", published_post.title);
    
    // Transfer funds (need another account)
    let user2 = user_service.create_user(
        "Ivy".to_string(),
        "ivy@example.com".to_string(),
    ).await?;
    
    let account2 = account_service.create_account(user2.id, 500.0).await?;
    
    // Transfer from Henry to Ivy
    account_service.transfer_funds(account.id, account2.id, 200.0).await?;
    
    let henry_balance = account_service.get_account_balance(account.id).await?;
    let ivy_balance = account_service.get_account_balance(account2.id).await?;
    
    println!("After transfer - Henry: {}, Ivy: {}", henry_balance, ivy_balance);
    
    Ok(())
}

// =========================================
// MAIN DEMONSTRATION
// =========================================

#[tokio::main]
async fn main() -> DbResult<()> {
    println!("=== DATABASE PROGRAMMING DEMONSTRATIONS ===");
    
    demonstrate_basic_operations().await?;
    demonstrate_repository_pattern().await?;
    demonstrate_unit_of_work().await?;
    demonstrate_connection_pool().await?;
    demonstrate_service_layer().await?;
    
    println!("\n=== DATABASE PROGRAMMING DEMONSTRATIONS COMPLETE ===");
    println!("Note: This uses a simulated database. Real implementations would use:");
    println!("- SQLx for modern async database access");
    println!("- Diesel for ORM functionality");
    println!("- Connection pooling for performance");
    println!("- Transactions for data consistency");
    
    Ok(())
}

// =========================================
// UNIT TESTS
// =========================================

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_user_creation() {
        let db = std::sync::Arc::new(tokio::sync::Mutex::new(SimulatedDatabase::new()));
        let user_repo = SimulatedUserRepository::new(db);
        
        let user = user_repo.create(NewUser {
            name: "Test User".to_string(),
            email: "test@example.com".to_string(),
        }).await.unwrap();
        
        assert_eq!(user.name, "Test User");
        assert_eq!(user.email, "test@example.com");
    }
    
    #[tokio::test]
    async fn test_duplicate_email_error() {
        let db = std::sync::Arc::new(tokio::sync::Mutex::new(SimulatedDatabase::new()));
        let user_repo = SimulatedUserRepository::new(db);
        
        user_repo.create(NewUser {
            name: "User 1".to_string(),
            email: "duplicate@example.com".to_string(),
        }).await.unwrap();
        
        let result = user_repo.create(NewUser {
            name: "User 2".to_string(),
            email: "duplicate@example.com".to_string(),
        }).await;
        
        assert!(matches!(result, Err(DatabaseError::Duplicate(_))));
    }
    
    #[tokio::test]
    async fn test_post_creation() {
        let db = std::sync::Arc::new(tokio::sync::Mutex::new(SimulatedDatabase::new()));
        let user_repo = SimulatedUserRepository::new(db.clone());
        let post_repo = SimulatedPostRepository::new(db);
        
        let user = user_repo.create(NewUser {
            name: "Test User".to_string(),
            email: "test@example.com".to_string(),
        }).await.unwrap();
        
        let post = post_repo.create(NewPost {
            title: "Test Post".to_string(),
            content: Some("Test content".to_string()),
            user_id: user.id,
        }).await.unwrap();
        
        assert_eq!(post.title, "Test Post");
        assert_eq!(post.user_id, user.id);
    }
    
    #[tokio::test]
    async fn test_account_transfer() {
        let db = std::sync::Arc::new(tokio::sync::Mutex::new(SimulatedDatabase::new()));
        let user_repo = SimulatedUserRepository::new(db.clone());
        let account_repo = SimulatedAccountRepository::new(db);
        
        let user1 = user_repo.create(NewUser {
            name: "User 1".to_string(),
            email: "user1@example.com".to_string(),
        }).await.unwrap();
        
        let user2 = user_repo.create(NewUser {
            name: "User 2".to_string(),
            email: "user2@example.com".to_string(),
        }).await.unwrap();
        
        let account1 = account_repo.create(user1.id, 1000.0).await.unwrap();
        let account2 = account_repo.create(user2.id, 500.0).await.unwrap();
        
        // Transfer 200 from account1 to account2
        let new_balance1 = account_repo.update_balance(account1.id, 800.0).await.unwrap();
        let new_balance2 = account_repo.update_balance(account2.id, 700.0).await.unwrap();
        
        assert_eq!(new_balance1.balance, 800.0);
        assert_eq!(new_balance2.balance, 700.0);
    }
    
    #[tokio::test]
    async fn test_connection_pool() {
        let pool = ConnectionPool::new(3);
        
        let conn1 = pool.acquire().await;
        let conn2 = pool.acquire().await;
        let conn3 = pool.acquire().await;
        
        // All connections should be available
        assert!(conn1.db.is_some());
        assert!(conn2.db.is_some());
        assert!(conn3.db.is_some());
        
        // Return connections to pool
        drop(conn1);
        drop(conn2);
        drop(conn3);
        
        // Should be able to acquire again
        let conn4 = pool.acquire().await;
        assert!(conn4.db.is_some());
    }
    
    #[tokio::test]
    async fn test_service_validation() {
        let db = std::sync::Arc::new(tokio::sync::Mutex::new(SimulatedDatabase::new()));
        let user_repo = Box::new(SimulatedUserRepository::new(db));
        let user_service = UserService::new(user_repo);
        
        // Test invalid email
        let result = user_service.create_user(
            "Test".to_string(),
            "invalid-email".to_string(),
        ).await;
        
        assert!(matches!(result, Err(DatabaseError::ValidationError(_))));
        
        // Test empty name
        let result = user_service.create_user(
            "".to_string(),
            "test@example.com".to_string(),
        ).await;
        
        assert!(matches!(result, Err(DatabaseError::ValidationError(_))));
    }
    
    #[tokio::test]
    async fn test_fund_transfer_validation() {
        let db = std::sync::Arc::new(tokio::sync::Mutex::new(SimulatedDatabase::new()));
        let user_repo = Box::new(SimulatedUserRepository::new(db.clone()));
        let account_repo = Box::new(SimulatedAccountRepository::new(db));
        
        let user = user_repo.create(NewUser {
            name: "Test User".to_string(),
            email: "test@example.com".to_string(),
        }).await.unwrap();
        
        let account = account_repo.create(user.id, 100.0).await.unwrap();
        
        let account_service = AccountService::new(account_repo, user_repo);
        
        // Test insufficient funds
        let result = account_service.transfer_funds(account.id, 999, 200.0).await;
        assert!(matches!(result, Err(DatabaseError::ValidationError(_))));
        
        // Test negative amount
        let result = account_service.transfer_funds(account.id, 999, -50.0).await;
        assert!(matches!(result, Err(DatabaseError::ValidationError(_))));
    }
}
