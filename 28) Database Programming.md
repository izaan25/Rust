# Database Programming in Rust

## Overview

Rust provides excellent support for database programming through various crates and libraries. This guide covers database access patterns, ORMs, connection pooling, and best practices for working with databases in Rust.

---

## Database Ecosystem

### Popular Database Crates

| Crate | Database | Type | Features |
|-------|----------|------|----------|
| `sqlx` | Multiple | Async/Sync | Compile-time checked queries |
| `diesel` | Multiple | Sync | ORM with migrations |
| `tokio-postgres` | PostgreSQL | Async | Low-level PostgreSQL |
| `mysql` | MySQL | Sync/Async | MySQL client |
| `rusqlite` | SQLite | Sync | SQLite bindings |
| `redis` | Redis | Async | Redis client |

### Choosing the Right Library

- **sqlx** - Modern, async, compile-time query checking
- **diesel** - Mature ORM with strong typing
- **tokio-postgres** - High-performance PostgreSQL
- **rusqlite** - Simple SQLite integration
- **redis** - Redis key-value store

---

## SQLx - Modern Database Access

### Setup

```toml
[dependencies]
sqlx = { version = "0.7", features = ["runtime-tokio-rustls", "postgres", "mysql", "sqlite", "uuid", "chrono"] }
tokio = { version = "1.0", features = ["full"] }
uuid = { version = "1.0", features = ["v4"] }
chrono = { version = "0.4", features = ["serde"] }
```

### Basic Connection

```rust
use sqlx::{PgPool, Row, SqlitePool, MySqlPool};
use std::env;

// PostgreSQL connection
async fn connect_postgres() -> Result<PgPool, sqlx::Error> {
    let database_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://user:password@localhost/db".to_string());
    
    PgPool::connect(&database_url).await
}

// SQLite connection
async fn connect_sqlite() -> Result<SqlitePool, sqlx::Error> {
    SqlitePool::connect("sqlite:./database.db").await
}

// MySQL connection
async fn connect_mysql() -> Result<MySqlPool, sqlx::Error> {
    let database_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "mysql://user:password@localhost/db".to_string());
    
    MySqlPool::connect(&database_url).await
}
```

### Query Types

```rust
use sqlx::prelude::*;
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, sqlx::FromRow)]
struct User {
    id: Uuid,
    name: String,
    email: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

// Query macro with compile-time checking
async fn get_user_by_id(pool: &PgPool, user_id: Uuid) -> Result<Option<User>, sqlx::Error> {
    sqlx::query_as!(
        User,
        "SELECT id, name, email, created_at, updated_at FROM users WHERE id = $1",
        user_id
    )
    .fetch_optional(pool)
    .await
}

// Dynamic query
async fn search_users(pool: &PgPool, name_pattern: &str) -> Result<Vec<User>, sqlx::Error> {
    sqlx::query_as!(
        User,
        "SELECT id, name, email, created_at, updated_at FROM users WHERE name ILIKE $1",
        format!("%{}%", name_pattern)
    )
    .fetch_all(pool)
    .await
}

// Raw query with manual mapping
async fn get_user_count(pool: &PgPool) -> Result<i64, sqlx::Error> {
    let result = sqlx::query("SELECT COUNT(*) FROM users")
        .fetch_one(pool)
        .await?;
    
    result.get::<i64, _>(0)
}
```

### Transactions

```rust
async fn transfer_funds(
    pool: &PgPool,
    from_account: i64,
    to_account: i64,
    amount: f64,
) -> Result<(), sqlx::Error> {
    let mut tx = pool.begin().await?;
    
    // Check sender balance
    let balance: f64 = sqlx::query_scalar!(
        "SELECT balance FROM accounts WHERE id = $1",
        from_account
    )
    .fetch_one(&mut tx)
    .await?;
    
    if balance < amount {
        tx.rollback().await?;
        return Err(sqlx::Error::Protocol("Insufficient funds".into()));
    }
    
    // Update sender balance
    sqlx::query!(
        "UPDATE accounts SET balance = balance - $1 WHERE id = $2",
        amount,
        from_account
    )
    .execute(&mut tx)
    .await?;
    
    // Update receiver balance
    sqlx::query!(
        "UPDATE accounts SET balance = balance + $1 WHERE id = $2",
        amount,
        to_account
    )
    .execute(&mut tx)
    .await?;
    
    // Commit transaction
    tx.commit().await?;
    Ok(())
}
```

---

## Diesel ORM

### Setup

```toml
[dependencies]
diesel = { version = "2.0", features = ["postgres", "uuid", "chrono"] }
diesel_migrations = "2.0"
uuid = { version = "1.0", features = ["v4"] }
chrono = { version = "0.4", features = ["serde"] }
```

### Schema Definition

```sql
-- diesel print-schema > src/schema.rs
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR NOT NULL,
    email VARCHAR NOT NULL UNIQUE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE TABLE posts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    title VARCHAR NOT NULL,
    content TEXT,
    user_id UUID NOT NULL REFERENCES users(id),
    published_at TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);
```

### Models

```rust
use diesel::prelude::*;
use diesel::pg::PgConnection;
use uuid::Uuid;
use chrono::{DateTime, Utc};

// Generated by diesel
table! {
    users (id) {
        id -> Uuid,
        name -> Varchar,
        email -> Varchar,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

table! {
    posts (id) {
        id -> Uuid,
        title -> Varchar,
        content -> Nullable<Text>,
        user_id -> Uuid,
        published_at -> Nullable<Timestamp>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

// Models
#[derive(Debug, Queryable, Selectable)]
#[diesel(table_name = users)]
pub struct User {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Queryable, Selectable, Associations)]
#[diesel(belongs_to(User))]
#[diesel(table_name = posts)]
pub struct Post {
    pub id: Uuid,
    pub title: String,
    pub content: Option<String>,
    pub user_id: Uuid,
    pub published_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// Insert structs
#[derive(Debug, Insertable)]
#[diesel(table_name = users)]
pub struct NewUser {
    pub name: String,
    pub email: String,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = posts)]
pub struct NewPost {
    pub title: String,
    pub content: Option<String>,
    pub user_id: Uuid,
}
```

### CRUD Operations

```rust
use diesel::prelude::*;
use diesel::result::Error as DieselError;

impl User {
    pub fn create(conn: &mut PgConnection, new_user: NewUser) -> Result<User, DieselError> {
        diesel::insert_into(users::table)
            .values(&new_user)
            .returning(User::as_returning())
            .get_result(conn)
    }
    
    pub fn find_by_id(conn: &mut PgConnection, user_id: Uuid) -> Result<User, DieselError> {
        users::table
            .filter(users::id.eq(user_id))
            .first(conn)
    }
    
    pub fn find_by_email(conn: &mut PgConnection, email: &str) -> Result<User, DieselError> {
        users::table
            .filter(users::email.eq(email))
            .first(conn)
    }
    
    pub fn update(&mut self, conn: &mut PgConnection) -> Result<User, DieselError> {
        diesel::update(users::table.filter(users::id.eq(self.id)))
            .set((
                users::name.eq(&self.name),
                users::email.eq(&self.email),
                users::updated_at.eq(Utc::now()),
            ))
            .returning(User::as_returning())
            .get_result(conn)
    }
    
    pub fn delete(conn: &mut PgConnection, user_id: Uuid) -> Result<usize, DieselError> {
        diesel::delete(users::table.filter(users::id.eq(user_id)))
            .execute(conn)
    }
}

impl Post {
    pub fn create(conn: &mut PgConnection, new_post: NewPost) -> Result<Post, DieselError> {
        diesel::insert_into(posts::table)
            .values(&new_post)
            .returning(Post::as_returning())
            .get_result(conn)
    }
    
    pub fn find_by_user(conn: &mut PgConnection, user_id: Uuid) -> Result<Vec<Post>, DieselError> {
        posts::table
            .filter(posts::user_id.eq(user_id))
            .order(posts::created_at.desc())
            .load(conn)
    }
    
    pub fn find_published(conn: &mut PgConnection) -> Result<Vec<Post>, DieselError> {
        posts::table
            .filter(posts::published_at.is_not_null())
            .order(posts::published_at.desc())
            .load(conn)
    }
}
```

---

## Connection Pooling

### SQLx Connection Pool

```rust
use sqlx::PgPool;
use std::time::Duration;

async fn setup_connection_pool() -> Result<PgPool, sqlx::Error> {
    let database_url = "postgres://user:password@localhost/db";
    
    let pool = PgPool::builder()
        .max_connections(20)
        .min_connections(5)
        .acquire_timeout(Duration::from_secs(30))
        .idle_timeout(Duration::from_secs(600))
        .max_lifetime(Duration::from_secs(1800))
        .build(database_url)
        .await?;
    
    // Test connection
    sqlx::query("SELECT 1")
        .fetch_one(&pool)
        .await?;
    
    Ok(pool)
}

// Using the pool
async fn get_user_with_pool(pool: &PgPool, user_id: Uuid) -> Result<Option<User>, sqlx::Error> {
    let mut conn = pool.acquire().await?;
    get_user_by_id(&mut conn, user_id).await
}
```

### Custom Pool Implementation

```rust
use std::sync::Arc;
use tokio::sync::Mutex;
use sqlx::PgConnection;

struct ConnectionPool {
    connections: Arc<Mutex<Vec<PgConnection>>>,
    max_size: usize,
    database_url: String,
}

impl ConnectionPool {
    async fn new(database_url: String, max_size: usize) -> Result<Self, sqlx::Error> {
        let mut connections = Vec::with_capacity(max_size);
        
        for _ in 0..max_size {
            let conn = PgConnection::connect(&database_url).await?;
            connections.push(conn);
        }
        
        Ok(ConnectionPool {
            connections: Arc::new(Mutex::new(connections)),
            max_size,
            database_url,
        })
    }
    
    async fn acquire(&self) -> Result<PooledConnection, sqlx::Error> {
        let mut connections = self.connections.lock().await;
        
        match connections.pop() {
            Some(conn) => Ok(PooledConnection {
                conn: Some(conn),
                pool: self.connections.clone(),
            }),
            None => {
                // Create new connection if pool is empty
                let conn = PgConnection::connect(&self.database_url).await?;
                Ok(PooledConnection {
                    conn: Some(conn),
                    pool: self.connections.clone(),
                })
            }
        }
    }
}

struct PooledConnection {
    conn: Option<PgConnection>,
    pool: Arc<Mutex<Vec<PgConnection>>>,
}

impl PooledConnection {
    async fn execute(&mut self, query: &str) -> Result<u64, sqlx::Error> {
        let conn = self.conn.as_mut().ok_or(sqlx::Error::Protocol("No connection".into()))?;
        let result = sqlx::query(query).execute(conn).await?;
        Ok(result.rows_affected())
    }
}

impl Drop for PooledConnection {
    fn drop(&mut self) {
        if let Some(conn) = self.conn.take() {
            let pool = self.pool.clone();
            tokio::spawn(async move {
                let mut connections = pool.lock().await;
                connections.push(conn);
            });
        }
    }
}
```

---

## Migrations

### SQLx Migrations

```rust
// migrations/20230101000000_create_users.sql
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR NOT NULL,
    email VARCHAR NOT NULL UNIQUE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE INDEX idx_users_email ON users(email);
CREATE INDEX idx_users_created_at ON users(created_at);

// migrations/20230102000000_create_posts.sql
CREATE TABLE posts (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    title VARCHAR NOT NULL,
    content TEXT,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    published_at TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE INDEX idx_posts_user_id ON posts(user_id);
CREATE INDEX idx_posts_published_at ON posts(published_at);
```

```rust
// Running migrations
use sqlx::migrate::MigrateDatabase;

async fn run_migrations(database_url: &str) -> Result<(), sqlx::Error> {
    // Create database if it doesn't exist
    if !sqlx::postgres::PgPool::connect(database_url)
        .await
        .is_ok()
    {
        sqlx::postgres::PgPool::create_database(database_url).await?;
    }
    
    // Run migrations
    let pool = PgPool::connect(database_url).await?;
    sqlx::migrate!("./migrations").run(&pool).await?;
    
    Ok(())
}
```

### Diesel Migrations

```bash
# Setup migrations
diesel setup

# Create new migration
diesel migration generate create_users

# Run migrations
diesel migration run

# Revert last migration
diesel migration revert
```

```rust
// migrations/up.sql
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR NOT NULL,
    email VARCHAR NOT NULL UNIQUE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

// migrations/down.sql
DROP TABLE users;
```

---

## Advanced Patterns

### Repository Pattern

```rust
use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;
use crate::models::User;

#[async_trait]
pub trait UserRepository {
    async fn create(&self, user: NewUser) -> Result<User, DatabaseError>;
    async fn find_by_id(&self, id: Uuid) -> Result<Option<User>, DatabaseError>;
    async fn find_by_email(&self, email: &str) -> Result<Option<User>, DatabaseError>;
    async fn update(&self, user: &User) -> Result<User, DatabaseError>;
    async fn delete(&self, id: Uuid) -> Result<bool, DatabaseError>;
    async fn list(&self, limit: u32, offset: u32) -> Result<Vec<User>, DatabaseError>;
}

pub struct SqlUserRepository {
    pool: PgPool,
}

#[async_trait]
impl UserRepository for SqlUserRepository {
    async fn create(&self, user: NewUser) -> Result<User, DatabaseError> {
        let user = sqlx::query_as!(
            User,
            "INSERT INTO users (name, email) VALUES ($1, $2) RETURNING *",
            user.name,
            user.email
        )
        .fetch_one(&self.pool)
        .await?;
        
        Ok(user)
    }
    
    async fn find_by_id(&self, id: Uuid) -> Result<Option<User>, DatabaseError> {
        let user = sqlx::query_as!(
            User,
            "SELECT * FROM users WHERE id = $1",
            id
        )
        .fetch_optional(&self.pool)
        .await?;
        
        Ok(user)
    }
    
    async fn find_by_email(&self, email: &str) -> Result<Option<User>, DatabaseError> {
        let user = sqlx::query_as!(
            User,
            "SELECT * FROM users WHERE email = $1",
            email
        )
        .fetch_optional(&self.pool)
        .await?;
        
        Ok(user)
    }
    
    async fn update(&self, user: &User) -> Result<User, DatabaseError> {
        let user = sqlx::query_as!(
            User,
            "UPDATE users SET name = $1, email = $2, updated_at = NOW() WHERE id = $3 RETURNING *",
            user.name,
            user.email,
            user.id
        )
        .fetch_one(&self.pool)
        .await?;
        
        Ok(user)
    }
    
    async fn delete(&self, id: Uuid) -> Result<bool, DatabaseError> {
        let result = sqlx::query!("DELETE FROM users WHERE id = $1", id)
            .execute(&self.pool)
            .await?;
        
        Ok(result.rows_affected() > 0)
    }
    
    async fn list(&self, limit: u32, offset: u32) -> Result<Vec<User>, DatabaseError> {
        let users = sqlx::query_as!(
            User,
            "SELECT * FROM users ORDER BY created_at DESC LIMIT $1 OFFSET $2",
            limit as i64,
            offset as i64
        )
        .fetch_all(&self.pool)
        .await?;
        
        Ok(users)
    }
}

#[derive(Debug)]
pub enum DatabaseError {
    NotFound,
    Duplicate,
    ConnectionError,
    QueryError(String),
}

impl From<sqlx::Error> for DatabaseError {
    fn from(err: sqlx::Error) -> Self {
        match err {
            sqlx::Error::RowNotFound => DatabaseError::NotFound,
            sqlx::Error::Database(db_err) => {
                match db_err.code() {
                    Some("23505") => DatabaseError::Duplicate, // Unique violation
                    _ => DatabaseError::QueryError(db_err.message().to_string()),
                }
            }
            _ => DatabaseError::QueryError(err.to_string()),
        }
    }
}
```

### Unit of Work Pattern

```rust
pub struct UnitOfWork {
    pool: PgPool,
    transaction: Option<sqlx::Transaction<'static, sqlx::Postgres>>,
}

impl UnitOfWork {
    pub async fn begin(pool: PgPool) -> Result<Self, DatabaseError> {
        let transaction = pool.begin().await?;
        Ok(UnitOfWork {
            pool,
            transaction: Some(transaction),
        })
    }
    
    pub async fn commit(mut self) -> Result<(), DatabaseError> {
        if let Some(tx) = self.transaction.take() {
            tx.commit().await?;
        }
        Ok(())
    }
    
    pub async fn rollback(mut self) -> Result<(), DatabaseError> {
        if let Some(tx) = self.transaction.take() {
            tx.rollback().await?;
        }
        Ok(())
    }
    
    pub fn user_repository(&self) -> SqlUserRepository {
        SqlUserRepository {
            pool: self.pool.clone(),
        }
    }
}

// Usage
async fn create_user_with_posts(
    pool: &PgPool,
    user_data: NewUser,
    posts_data: Vec<NewPost>,
) -> Result<(User, Vec<Post>), DatabaseError> {
    let mut uow = UnitOfWork::begin(pool.clone()).await?;
    
    // Create user
    let user = uow.user_repository().create(user_data).await?;
    
    // Create posts
    let mut posts = Vec::new();
    for post_data in posts_data {
        let post_data = NewPost {
            user_id: user.id,
            ..post_data
        };
        let post = uow.post_repository().create(post_data).await?;
        posts.push(post);
    }
    
    // Commit transaction
    uow.commit().await?;
    
    Ok((user, posts))
}
```

---

## Performance Optimization

### Query Optimization

```rust
// Use prepared statements
async fn get_users_optimized(pool: &PgPool, limit: u32) -> Result<Vec<User>, sqlx::Error> {
    let query = sqlx::query_as!(
        User,
        "SELECT id, name, email, created_at, updated_at 
         FROM users 
         ORDER BY created_at DESC 
         LIMIT $1",
        limit as i64
    );
    
    query.fetch_all(pool).await
}

// Batch operations
async fn create_users_batch(pool: &PgPool, users: Vec<NewUser>) -> Result<Vec<User>, sqlx::Error> {
    let mut transaction = pool.begin().await?;
    
    let mut created_users = Vec::new();
    for user in users {
        let created = sqlx::query_as!(
            User,
            "INSERT INTO users (name, email) VALUES ($1, $2) RETURNING *",
            user.name,
            user.email
        )
        .fetch_one(&mut transaction)
        .await?;
        
        created_users.push(created);
    }
    
    transaction.commit().await?;
    Ok(created_users)
}

// Use indexes effectively
async fn search_users_with_index(pool: &PgPool, email_pattern: &str) -> Result<Vec<User>, sqlx::Error> {
    // This query can use the email index
    sqlx::query_as!(
        User,
        "SELECT * FROM users WHERE email LIKE $1 ORDER BY created_at DESC",
        format!("{}%", email_pattern)
    )
    .fetch_all(pool)
    .await
}
```

### Connection Pool Tuning

```rust
async fn optimized_pool_setup() -> Result<PgPool, sqlx::Error> {
    let database_url = "postgres://user:password@localhost/db";
    
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(20)           // Maximum connections
        .min_connections(5)            // Minimum connections
        .acquire_timeout(Duration::from_secs(30))  // Wait time for connection
        .idle_timeout(Duration::from_secs(600))    // Close idle connections
        .max_lifetime(Duration::from_secs(1800))    // Connection lifetime
        .test_before_acquire(true)      // Test connection before use
        .after_connect(|conn| Box::pin(async move {
            // Set connection parameters
            sqlx::query("SET timezone = 'UTC'")
                .execute(conn)
                .await?;
            Ok(conn)
        }))
        .connect(database_url)
        .await?;
    
    Ok(pool)
}
```

---

## Error Handling

### Custom Error Types

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DatabaseError {
    #[error("Record not found")]
    NotFound,
    
    #[error("Duplicate record: {0}")]
    Duplicate(String),
    
    #[error("Connection error: {0}")]
    ConnectionError(#[from] sqlx::Error),
    
    #[error("Validation error: {0}")]
    ValidationError(String),
    
    #[error("Migration error: {0}")]
    MigrationError(String),
}

// Result type alias
pub type DbResult<T> = Result<T, DatabaseError>;

// Error mapping
impl From<sqlx::Error> for DatabaseError {
    fn from(err: sqlx::Error) -> Self {
        match err {
            sqlx::Error::RowNotFound => DatabaseError::NotFound,
            sqlx::Error::Database(db_err) => {
                match db_err.code() {
                    Some("23505") => DatabaseError::Duplicate(db_err.message().to_string()),
                    _ => DatabaseError::ConnectionError(sqlx::Error::Database(db_err)),
                }
            }
            _ => DatabaseError::ConnectionError(err),
        }
    }
}
```

### Error Recovery

```rust
async fn resilient_operation(pool: &PgPool, user_id: Uuid) -> Result<User, DatabaseError> {
    let mut retries = 3;
    let mut delay = Duration::from_millis(100);
    
    loop {
        match get_user_by_id(pool, user_id).await {
            Ok(user) => return Ok(user),
            Err(DatabaseError::ConnectionError(err)) => {
                if retries == 0 {
                    return Err(DatabaseError::ConnectionError(err));
                }
                
                retries -= 1;
                tokio::time::sleep(delay).await;
                delay *= 2; // Exponential backoff
            }
            Err(err) => return Err(err),
        }
    }
}
```

---

## Key Takeaways

- **sqlx** provides modern, async database access with compile-time checking
- **diesel** offers a mature ORM with strong typing and migrations
- **Connection pooling** is essential for performance
- **Transactions** ensure data consistency
- **Error handling** should be comprehensive and user-friendly
- **Performance** depends on proper indexing and query optimization
- **Migrations** manage database schema changes safely

---

## Database Best Practices

| Practice | Description | Implementation |
|----------|-------------|----------------|
| **Connection Pooling** | Reuse database connections | Use built-in pool or custom implementation |
| **Prepared Statements** | Prevent SQL injection and improve performance | Use query macros or prepare statements |
| **Transactions** | Ensure data consistency | Use explicit transactions for multi-step operations |
| **Indexing** | Improve query performance | Add indexes for frequently queried columns |
| **Error Handling** | Handle database errors gracefully | Use custom error types and recovery strategies |
| **Migrations** | Manage schema changes | Use migration tools for version control |
| **Testing** | Test database interactions | Use test databases and fixtures |
