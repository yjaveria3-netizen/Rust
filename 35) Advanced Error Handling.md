# Advanced Error Handling in Rust

## Overview

Rust's error handling system is powerful and expressive. This guide explores advanced error handling patterns, custom error types, error propagation strategies, and building robust error handling systems in Rust applications.

---

## Error Handling Fundamentals

### Result and Option Types

```rust
use std::fmt;

// Basic Result usage
fn divide(a: f64, b: f64) -> Result<f64, String> {
    if b == 0.0 {
        Err("Division by zero".to_string())
    } else {
        Ok(a / b)
    }
}

// Option for potentially missing values
fn find_user_by_id(id: u32) -> Option<User> {
    // Simulate database lookup
    if id == 42 {
        Some(User { id, name: "Alice".to_string() })
    } else {
        None
    }
}

// Combining Results
fn process_user(id: u32) -> Result<String, String> {
    find_user_by_id(id)
        .map(|user| format!("Found user: {}", user.name))
        .ok_or_else(|| format!("User {} not found", id))
}
```

### Error Propagation with `?`

```rust
use std::io;
use std::fs;

fn read_config_file() -> Result<String, Box<dyn std::error::Error>> {
    let content = fs::read_to_string("config.toml")?; // Propagate IO error
    parse_config(&content) // Propagate parse error
}

fn parse_config(content: &str) -> Result<String, Box<dyn std::error::Error>> {
    // Simulate parsing
    if content.is_empty() {
        Err("Empty config file".into())
    } else {
        Ok(content.to_string())
    }
}
```

---

## Custom Error Types

### Using thiserror for Error Definitions

```toml
[dependencies]
thiserror = "1.0"
```

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DatabaseError {
    #[error("Database connection failed: {0}")]
    ConnectionError(#[from] sqlx::Error),
    
    #[error("Query failed: {0}")]
    QueryError(String),
    
    #[error("User not found: {id}")]
    UserNotFound { id: u32 },
    
    #[error("Validation error: {field} - {message}")]
    ValidationError { field: String, message: String },
}

#[derive(Error, Debug)]
pub enum ApiError {
    #[error("Database error: {0}")]
    Database(#[from] DatabaseError),
    
    #[error("Authentication failed: {0}")]
    Authentication(String),
    
    #[error("Authorization failed: {0}")]
    Authorization(String),
    
    #[error("Rate limit exceeded")]
    RateLimit,
    
    #[error("Internal server error: {0}")]
    Internal(String),
}

// Convert from other error types
impl From<sqlx::Error> for DatabaseError {
    fn from(err: sqlx::Error) -> Self {
        DatabaseError::ConnectionError(err)
    }
}
```

### Manual Error Implementation

```rust
#[derive(Debug)]
pub struct ValidationError {
    pub field: String,
    pub message: String,
    pub source: Option<Box<dyn std::error::Error>>,
}

impl ValidationError {
    pub fn new(field: String, message: String) -> Self {
        ValidationError {
            field,
            message,
            source: None,
        }
    }
    
    pub fn with_source<E: std::error::Error + 'static>(field: String, message: String, source: E) -> Self {
        ValidationError {
            field,
            message,
            source: Some(Box::new(source)),
        }
    }
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Validation error in field '{}': {}", self.field, self.message)
    }
}

impl std::error::Error for ValidationError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.source.as_ref().map(|e| e.as_ref() as &dyn std::error::Error)
    }
}
```

---

## Error Context and Chains

### Error Context with anyhow

```toml
[dependencies]
anyhow = "1.0"
```

```rust
use anyhow::{Context, Result, anyhow};

fn process_user_data(user_id: u32) -> Result<User> {
    let user = fetch_user_from_database(user_id)
        .context("Failed to fetch user from database")?;
    
    validate_user(&user)
        .context("User validation failed")?;
    
    Ok(user)
}

fn fetch_user_from_database(id: u32) -> Result<User> {
    // Simulate database operation
    if id == 0 {
        Err(anyhow!("Invalid user ID: {}", id))
    } else {
        Ok(User { id, name: "User".to_string() })
    }
}

fn validate_user(user: &User) -> Result<()> {
    if user.name.is_empty() {
        Err(anyhow!("User name cannot be empty"))
    } else {
        Ok(())
    }
}
```

### Custom Error Context

```rust
use std::collections::HashMap;

#[derive(Debug)]
pub struct ErrorContext {
    pub operation: String,
    pub user_id: Option<u32>,
    pub request_id: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub metadata: HashMap<String, String>,
}

impl ErrorContext {
    pub fn new(operation: String) -> Self {
        ErrorContext {
            operation,
            user_id: None,
            request_id: uuid::Uuid::new_v4().to_string(),
            timestamp: chrono::Utc::now(),
            metadata: HashMap::new(),
        }
    }
    
    pub fn with_user(mut self, user_id: u32) -> Self {
        self.user_id = Some(user_id);
        self
    }
    
    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }
}

#[derive(Debug)]
pub struct ContextualError<E> {
    pub error: E,
    pub context: ErrorContext,
}

impl<E> ContextualError<E> {
    pub fn new(error: E, context: ErrorContext) -> Self {
        ContextualError { error, context }
    }
}

impl<E> std::fmt::Display for ContextualError<E>
where
    E: std::fmt::Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{} in operation '{}'", self.error, self.context.operation)
    }
}

impl<E> std::error::Error for ContextualError<E>
where
    E: std::error::Error + 'static,
{
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.error as &dyn std::error::Error)
    }
}
```

---

## Error Recovery Strategies

### Retry Mechanisms

```rust
use std::time::Duration;
use tokio::time::sleep;

#[derive(Debug, Clone)]
pub struct RetryConfig {
    pub max_attempts: u32,
    pub initial_delay: Duration,
    pub max_delay: Duration,
    pub multiplier: f64,
    pub jitter: bool,
}

impl Default for RetryConfig {
    fn default() -> Self {
        RetryConfig {
            max_attempts: 3,
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(10),
            multiplier: 2.0,
            jitter: true,
        }
    }
}

async fn retry_with_backoff<F, T, E, Fut>(
    config: RetryConfig,
    operation: F,
) -> Result<T, E>
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = Result<T, E>>,
    E: std::fmt::Display,
{
    let mut delay = config.initial_delay;
    
    for attempt in 1..=config.max_attempts {
        match operation().await {
            Ok(result) => {
                if attempt > 1 {
                    println!("Operation succeeded on attempt {}", attempt);
                }
                return Ok(result);
            }
            Err(e) => {
                if attempt == config.max_attempts {
                    return Err(e);
                }
                
                let actual_delay = if config.jitter {
                    let jitter_factor = rand::random::<f64>() * 0.1 + 0.9;
                    Duration::from_millis((delay.as_millis() as f64 * jitter_factor) as u64)
                } else {
                    delay
                };
                
                println!("Attempt {} failed: {}. Retrying in {:?}", attempt, e, actual_delay);
                sleep(actual_delay).await;
                
                delay = std::cmp::min(
                    Duration::from_millis((delay.as_millis() as f64 * config.multiplier) as u64),
                    config.max_delay,
                );
            }
        }
    }
}

// Usage
async fn fetch_with_retry(url: &str) -> Result<String, reqwest::Error> {
    let config = RetryConfig {
        max_attempts: 5,
        initial_delay: Duration::from_millis(100),
        max_delay: Duration::from_secs(5),
        multiplier: 1.5,
        jitter: true,
    };
    
    retry_with_backoff(config, || async {
        reqwest::get(url).send().await?.text().await
    }).await
}
```

### Circuit Breaker Pattern

```rust
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CircuitState {
    Closed,
    Open,
    HalfOpen,
}

pub struct CircuitBreaker {
    state: Arc<RwLock<CircuitState>>,
    failure_count: Arc<RwLock<u32>>,
    success_count: Arc<RwLock<u32>>,
    failure_threshold: u32,
    success_threshold: u32,
    timeout: Duration,
    last_failure: Arc<RwLock<Option<Instant>>>,
}

impl CircuitBreaker {
    pub fn new(failure_threshold: u32, success_threshold: u32, timeout: Duration) -> Self {
        CircuitBreaker {
            state: Arc::new(RwLock::new(CircuitState::Closed)),
            failure_count: Arc::new(RwLock::new(0)),
            success_count: Arc::new(RwLock::new(0)),
            failure_threshold,
            success_threshold,
            timeout,
            last_failure: Arc::new(RwLock::new(None)),
        }
    }
    
    pub async fn call<F, T, E>(&self, operation: F) -> Result<T, CircuitBreakerError<E>>
    where
        F: FnOnce() -> Result<T, E>,
    {
        let state = *self.state.read().await;
        
        match state {
            CircuitState::Open => {
                let last_failure = *self.last_failure.read().await;
                if let Some(failure_time) = last_failure {
                    if failure_time.elapsed() > self.timeout {
                        *self.state.write().await = CircuitState::HalfOpen;
                    } else {
                        return Err(CircuitBreakerError::CircuitOpen);
                    }
                } else {
                    return Err(CircuitBreakerError::CircuitOpen);
                }
            }
            CircuitState::HalfOpen => {
                // Allow one request through
            }
            CircuitState::Closed => {
                // Normal operation
            }
        }
        
        match operation() {
            Ok(result) => {
                *self.failure_count.write().await = 0;
                let mut success_count = self.success_count.write().await;
                *success_count += 1;
                
                if *success_count >= self.success_threshold {
                    *self.state.write().await = CircuitState::Closed;
                }
                
                Ok(result)
            }
            Err(error) => {
                let mut failure_count = self.failure_count.write().await;
                *failure_count += 1;
                *self.last_failure.write().await = Some(Instant::now());
                
                if *failure_count >= self.failure_threshold {
                    *self.state.write().await = CircuitState::Open;
                    *self.success_count.write().await = 0;
                }
                
                Err(CircuitBreakerError::ServiceError(error))
            }
        }
    }
    
    pub async fn get_state(&self) -> CircuitState {
        *self.state.read().await
    }
}

#[derive(Debug)]
pub enum CircuitBreakerError<E> {
    ServiceError(E),
    CircuitOpen,
}
```

---

## Async Error Handling

### Async Result Types

```rust
use tokio::sync::mpsc;

type AsyncResult<T> = Result<T, Box<dyn std::error::Error + Send + Sync>>;

async fn async_operation() -> AsyncResult<String> {
    tokio::time::sleep(Duration::from_millis(100)).await;
    Ok("Operation completed".to_string())
}

async fn async_operation_with_error() -> AsyncResult<String> {
    tokio::time::sleep(Duration::from_millis(100)).await;
    Err("Async operation failed".into())
}

// Handling multiple async operations
async fn process_multiple_operations() -> AsyncResult<Vec<String>> {
    let mut results = Vec::new();
    
    let operations = vec![
        async_operation(),
        async_operation(),
        async_operation_with_error(),
    ];
    
    for operation in operations {
        match operation.await {
            Ok(result) => results.push(result),
            Err(e) => eprintln!("Operation failed: {}", e),
        }
    }
    
    Ok(results)
}
```

### Concurrent Error Handling

```rust
use futures::future::join_all;

async fn concurrent_operations() -> AsyncResult<Vec<String>> {
    let operations = vec![
        async_operation(),
        async_operation(),
        async_operation(),
    ];
    
    let results = join_all(operations).await;
    
    let mut successful_results = Vec::new();
    let mut errors = Vec::new();
    
    for result in results {
        match result {
            Ok(value) => successful_results.push(value),
            Err(e) => errors.push(e),
        }
    }
    
    if !errors.is_empty() {
        eprintln!("{} operations failed", errors.len());
    }
    
    Ok(successful_results)
}

// Error handling with tokio::select!
async fn race_operations() -> AsyncResult<String> {
    let op1 = async_operation();
    let op2 = async_operation_with_timeout();
    
    tokio::select! {
        result = op1 => {
            println!("Operation 1 completed first");
            result
        }
        result = op2 => {
            println!("Operation 2 completed first");
            result
        }
    }
}

async fn async_operation_with_timeout() -> AsyncResult<String> {
    tokio::time::timeout(Duration::from_millis(50), async_operation()).await
        .map_err(|_| "Operation timed out".into())
        .and_then(|inner| inner)
}
```

---

## Error Aggregation and Reporting

### Error Collection System

```rust
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone)]
pub struct ErrorReport {
    pub error_id: String,
    pub error_type: String,
    pub message: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub count: u32,
    pub last_occurrence: chrono::DateTime<chrono::Utc>,
    pub context: HashMap<String, String>,
}

pub struct ErrorCollector {
    errors: Arc<RwLock<HashMap<String, ErrorReport>>>,
}

impl ErrorCollector {
    pub fn new() -> Self {
        ErrorCollector {
            errors: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    pub async fn report_error(&self, error_type: String, message: String, context: HashMap<String, String>) {
        let error_id = format!("{}:{}", error_type, message);
        let now = chrono::Utc::now();
        
        let mut errors = self.errors.write().await;
        
        match errors.get_mut(&error_id) {
            Some(report) => {
                report.count += 1;
                report.last_occurrence = now;
                for (key, value) in context {
                    report.context.insert(key, value);
                }
            }
            None => {
                errors.insert(error_id.clone(), ErrorReport {
                    error_id: error_id.clone(),
                    error_type,
                    message,
                    timestamp: now,
                    count: 1,
                    last_occurrence: now,
                    context,
                });
            }
        }
    }
    
    pub async fn get_error_summary(&self) -> Vec<ErrorReport> {
        let errors = self.errors.read().await;
        errors.values().cloned().collect()
    }
    
    pub async fn clear_errors(&self) {
        let mut errors = self.errors.write().await;
        errors.clear();
    }
}

// Error metrics
#[derive(Debug)]
pub struct ErrorMetrics {
    pub total_errors: u32,
    pub errors_by_type: HashMap<String, u32>,
    pub errors_in_last_hour: u32,
    pub errors_in_last_day: u32,
}

impl ErrorCollector {
    pub async fn get_metrics(&self) -> ErrorMetrics {
        let errors = self.errors.read().await;
        let now = chrono::Utc::now();
        let one_hour_ago = now - chrono::Duration::hours(1);
        let one_day_ago = now - chrono::Duration::days(1);
        
        let mut metrics = ErrorMetrics {
            total_errors: 0,
            errors_by_type: HashMap::new(),
            errors_in_last_hour: 0,
            errors_in_last_day: 0,
        };
        
        for report in errors.values() {
            metrics.total_errors += 1;
            
            *metrics.errors_by_type.entry(report.error_type.clone()).or_insert(0) += 1;
            
            if report.last_occurrence > one_hour_ago {
                metrics.errors_in_last_hour += 1;
            }
            
            if report.last_occurrence > one_day_ago {
                metrics.errors_in_last_day += 1;
            }
        }
        
        metrics
    }
}
```

---

## Error Handling in Web Applications

### Axum Error Handling

```rust
use axum::{
    extract::rejection::JsonRejection,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

#[derive(Debug)]
pub enum AppError {
    Database(#[from] DatabaseError),
    Validation(#[from] ValidationError),
    NotFound(String),
    Internal(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AppError::Database(ref e) => {
                tracing::error!("Database error: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error")
            }
            AppError::Validation(ref e) => {
                tracing::warn!("Validation error: {}", e);
                (StatusCode::BAD_REQUEST, e.to_string())
            }
            AppError::NotFound(ref message) => {
                (StatusCode::NOT_FOUND, message.as_str())
            }
            AppError::Internal(ref message) => {
                tracing::error!("Internal error: {}", message);
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error")
            }
        };
        
        let body = json!({
            "error": error_message,
            "status": status.as_u16(),
        });
        
        (status, Json(body)).into_response()
    }
}

// Error handling middleware
async fn handle_rejection(err: axum::response::IntoResponse) -> Response {
    tracing::error!("Request rejected: {:?}", err);
    err.into_response()
}

// Usage in handlers
async fn get_user(
    axum::extract::Path(user_id): axum::extract::Path<u32>,
) -> Result<Json<User>, AppError> {
    if user_id == 0 {
        Err(AppError::Validation(ValidationError::new(
            "user_id".to_string(),
            "User ID cannot be zero".to_string(),
        )))
    } else {
        // Simulate database fetch
        Ok(Json(User {
            id: user_id,
            name: "User".to_string(),
        }))
    }
}
```

### Error Response Formatting

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
    pub details: Option<serde_json::Value>,
    pub request_id: Option<String>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl ErrorResponse {
    pub fn new(error: String, message: String) -> Self {
        ErrorResponse {
            error,
            message,
            details: None,
            request_id: None,
            timestamp: chrono::Utc::now(),
        }
    }
    
    pub fn with_details(mut self, details: serde_json::Value) -> Self {
        self.details = Some(details);
        self
    }
    
    pub fn with_request_id(mut self, request_id: String) -> Self {
        self.request_id = Some(request_id);
        self
    }
}

#[derive(Debug, Deserialize)]
pub struct ValidationErrorDetail {
    pub field: String,
    pub message: String,
}

pub fn validation_error_response(errors: Vec<ValidationErrorDetail>) -> ErrorResponse {
    let error_details = serde_json::json!(errors);
    
    ErrorResponse::new(
        "validation_error".to_string(),
        "Request validation failed".to_string(),
    )
    .with_details(error_details)
}
```

---

## Error Handling Best Practices

### Error Design Principles

```rust
// 1. Be specific about error conditions
#[derive(Debug, Error)]
pub enum FileProcessingError {
    #[error("File not found: {path}")]
    FileNotFound { path: String },
    
    #[error("Permission denied: {path}")]
    PermissionDenied { path: String },
    
    #[error("Invalid file format: {format}")]
    InvalidFormat { format: String },
    
    #[error("File too large: {size} bytes (max: {max_size})")]
    FileTooLarge { size: u64, max_size: u64 },
}

// 2. Provide context and recovery information
impl FileProcessingError {
    pub fn can_retry(&self) -> bool {
        match self {
            FileProcessingError::FileNotFound { .. } => false,
            FileProcessingError::PermissionDenied { .. } => false,
            FileProcessingError::InvalidFormat { .. } => true,
            FileProcessingError::FileTooLarge { .. } => true,
        }
    }
    
    pub fn suggested_action(&self) -> &'static str {
        match self {
            FileProcessingError::FileNotFound { .. } => "Check file path and permissions",
            FileProcessingError::PermissionDenied { .. } => "Check file permissions",
            FileProcessingError::InvalidFormat { .. } => "Verify file format requirements",
            FileProcessingError::FileTooLarge { .. } => "Compress file or increase size limit",
        }
    }
}

// 3. Use consistent error handling patterns
pub trait ResultExt<T, E> {
    fn log_error(self) -> Self;
    fn with_context<F>(self, f: F) -> Self
    where
        F: FnOnce() -> String;
}

impl<T, E> ResultExt<T, E> for Result<T, E>
where
    E: std::fmt::Display,
{
    fn log_error(self) -> Self {
        match &self {
            Ok(_) => {}
            Err(e) => tracing::error!("Operation failed: {}", e),
        }
        self
    }
    
    fn with_context<F>(self, f: F) -> Self
    where
        F: FnOnce() -> String,
    {
        match &self {
            Ok(_) => {}
            Err(e) => tracing::error!("Operation failed: {} - {}", e, f()),
        }
        self
    }
}

// Usage
fn process_file(path: &str) -> Result<String, FileProcessingError> {
    read_file(path)
        .log_error()
        .with_context(|| format!("While processing file: {}", path))
}
```

---

## Key Takeaways

- **Custom error types** provide better error information
- **Error context** helps with debugging and monitoring
- **Retry mechanisms** handle transient failures gracefully
- **Circuit breakers** prevent cascade failures
- **Error aggregation** provides insights into system health
- **Structured error responses** improve API usability
- **Consistent patterns** make error handling predictable

---

## Advanced Error Handling Best Practices

| Practice | Description | Implementation |
|----------|-------------|----------------|
| **Specific errors** | Clear error conditions | Use detailed error variants |
| **Error context** | Additional debugging info | Include operation metadata |
| **Recovery guidance** | Help users resolve issues | Provide suggested actions |
| **Structured logging** | Better observability | Use tracing with context |
| **Retry strategies** | Handle transient failures | Exponential backoff with jitter |
| **Circuit breaking** | Prevent cascade failures | Monitor failure rates |
| **Error aggregation** | System health insights | Collect and analyze errors |
| **User-friendly messages** | Clear error communication | Provide actionable error info |
