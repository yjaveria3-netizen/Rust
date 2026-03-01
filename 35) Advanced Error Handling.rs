// 35_advanced_error_handling.rs
// Comprehensive examples of advanced error handling in Rust

use std::collections::HashMap;
use std::fmt;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

// =========================================
// CUSTOM ERROR TYPES
// =========================================

#[derive(Debug, Clone)]
pub struct User {
    pub id: u32,
    pub name: String,
    pub email: String,
}

// Using thiserror for comprehensive error definitions
#[derive(Debug, thiserror::Error)]
pub enum DatabaseError {
    #[error("Database connection failed: {0}")]
    ConnectionError(#[from] sqlx::Error),
    
    #[error("Query failed: {0}")]
    QueryError(String),
    
    #[error("User not found: {id}")]
    UserNotFound { id: u32 },
    
    #[error("Duplicate user: {email}")]
    DuplicateUser { email: String },
    
    #[error("Transaction failed: {0}")]
    TransactionError(String),
    
    #[error("Migration failed: {0}")]
    MigrationError(String),
}

#[derive(Debug, thiserror::Error)]
pub enum ValidationError {
    #[error("Invalid email format: {email}")]
    InvalidEmail { email: String },
    
    #[error("Name too short: {min_length} characters minimum")]
    NameTooShort { min_length: usize },
    
    #[error("Name too long: {max_length} characters maximum")]
    NameTooLong { max_length: usize },
    
    #[error("Invalid age: {age} (must be between {min} and {max})")]
    InvalidAge { age: u32, min: u32, max: u32 },
    
    #[error("Required field missing: {field}")]
    RequiredField { field: String },
}

#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("Database error: {0}")]
    Database(#[from] DatabaseError),
    
    #[error("Validation error: {0}")]
    Validation(#[from] ValidationError),
    
    #[error("Authentication failed: {0}")]
    Authentication(String),
    
    #[error("Authorization failed: {0}")]
    Authorization(String),
    
    #[error("Rate limit exceeded: {limit} requests per {window}")]
    RateLimit { limit: u32, window: Duration },
    
    #[error("Resource not found: {resource}")]
    NotFound { resource: String },
    
    #[error("Internal server error: {0}")]
    Internal(String),
    
    #[error("Service unavailable: {service}")]
    ServiceUnavailable { service: String },
}

// Manual error implementation for complex scenarios
#[derive(Debug)]
pub struct ProcessingError {
    pub operation: String,
    pub stage: String,
    pub message: String,
    pub error_code: String,
    pub timestamp: Instant,
    pub context: HashMap<String, String>,
    pub recoverable: bool,
    pub retry_count: u32,
}

impl ProcessingError {
    pub fn new(operation: String, stage: String, message: String) -> Self {
        ProcessingError {
            operation,
            stage,
            message,
            error_code: "PROC_ERROR".to_string(),
            timestamp: Instant::now(),
            context: HashMap::new(),
            recoverable: true,
            retry_count: 0,
        }
    }
    
    pub fn with_error_code(mut self, code: String) -> Self {
        self.error_code = code;
        self
    }
    
    pub fn with_context(mut self, key: String, value: String) -> Self {
        self.context.insert(key, value);
        self
    }
    
    pub fn unrecoverable(mut self) -> Self {
        self.recoverable = false;
        self
    }
    
    pub fn increment_retry(&mut self) {
        self.retry_count += 1;
    }
    
    pub fn can_retry(&self) -> bool {
        self.recoverable && self.retry_count < 3
    }
}

impl fmt::Display for ProcessingError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} error in stage {}: {} ({})", 
                self.operation, self.stage, self.message, self.error_code)
    }
}

impl std::error::Error for ProcessingError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

// =========================================
// ERROR CONTEXT AND CHAINS
// =========================================

#[derive(Debug, Clone)]
pub struct ErrorContext {
    pub operation: String,
    pub user_id: Option<u32>,
    pub request_id: String,
    pub session_id: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub metadata: HashMap<String, String>,
    pub stack_trace: Vec<String>,
}

impl ErrorContext {
    pub fn new(operation: String) -> Self {
        ErrorContext {
            operation,
            user_id: None,
            request_id: uuid::Uuid::new_v4().to_string(),
            session_id: uuid::Uuid::new_v4().to_string(),
            timestamp: chrono::Utc::now(),
            metadata: HashMap::new(),
            stack_trace: Vec::new(),
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
    
    pub fn add_stack_frame(mut self, frame: String) -> Self {
        self.stack_trace.push(frame);
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

impl<E> fmt::Display for ContextualError<E>
where
    E: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
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

// Error chain builder
pub struct ErrorChain {
    errors: Vec<Box<dyn std::error::Error + Send + Sync>>,
    context: ErrorContext,
}

impl ErrorChain {
    pub fn new(context: ErrorContext) -> Self {
        ErrorChain {
            errors: Vec::new(),
            context,
        }
    }
    
    pub fn add_error<E: std::error::Error + Send + Sync + 'static>(mut self, error: E) -> Self {
        self.errors.push(Box::new(error));
        self
    }
    
    pub fn add_context(mut self, key: String, value: String) -> Self {
        self.context.metadata.insert(key, value);
        self
    }
}

impl fmt::Display for ErrorChain {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.errors.is_empty() {
            write!(f, "No errors in chain")
        } else {
            write!(f, "Error chain in operation '{}':", self.context.operation)?;
            for (i, error) in self.errors.iter().enumerate() {
                write!(f, "  {}. {}", i + 1, error)?;
            }
            Ok(())
        }
    }
}

impl std::error::Error for ErrorChain {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.errors.last().map(|e| e.as_ref())
    }
}

// =========================================
// RETRY MECHANISMS
// =========================================

#[derive(Debug, Clone)]
pub struct RetryConfig {
    pub max_attempts: u32,
    pub initial_delay: Duration,
    pub max_delay: Duration,
    pub multiplier: f64,
    pub jitter: bool,
    pub exponential_base: f64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        RetryConfig {
            max_attempts: 3,
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(30),
            multiplier: 2.0,
            jitter: true,
            exponential_base: 2.0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct RetryPolicy {
    pub config: RetryConfig,
    pub retryable_errors: Vec<String>,
    pub max_retryable_errors: u32,
}

impl RetryPolicy {
    pub fn new(config: RetryConfig) -> Self {
        RetryPolicy {
            config,
            retryable_errors: vec![
                "timeout".to_string(),
                "connection_refused".to_string(),
                "service_unavailable".to_string(),
            ],
            max_retryable_errors: 10,
        }
    }
    
    pub fn should_retry(&self, error: &str, attempt: u32) -> bool {
        attempt < self.config.max_attempts &&
            self.retryable_errors.contains(&error.to_string()) &&
            attempt < self.max_retryable_errors
    }
    
    pub fn calculate_delay(&self, attempt: u32) -> Duration {
        let base_delay = self.config.initial_delay;
        let delay = base_delay.as_millis() as f64 * 
            self.config.exponential_base.powi(atlet as i32 - 1);
        
        let calculated_delay = Duration::from_millis(delay as u64);
        let capped_delay = std::cmp::min(calculated_delay, self.config.max_delay);
        
        if self.config.jitter {
            let jitter_factor = rand::random::<f64>() * 0.1 + 0.9;
            Duration::from_millis((capped_delay.as_millis() as f64 * jitter_factor) as u64)
        } else {
            capped_delay
        }
    }
}

// Generic retry function
pub async fn retry_with_policy<F, T, E, Fut>(
    policy: RetryPolicy,
    operation: F,
) -> Result<T, RetryError<E>>
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = Result<T, E>>,
    E: std::fmt::Display,
{
    let mut last_error = None;
    
    for attempt in 1..=policy.config.max_attempts {
        match operation().await {
            Ok(result) => {
                if attempt > 1 {
                    println!("Operation succeeded on attempt {}", attempt);
                }
                return Ok(result);
            }
            Err(error) => {
                let error_str = error.to_string();
                
                if !policy.should_retry(&error_str, attempt) {
                    return Err(RetryError::NonRetryable(error));
                }
                
                println!("Attempt {} failed: {}. Retrying in {:?}", 
                         attempt, error, policy.calculate_delay(attempt));
                
                last_error = Some(RetryError::Retryable(error, attempt));
                
                tokio::time::sleep(policy.calculate_delay(attempt)).await;
            }
        }
    }
    
    last_error.unwrap_or_else(|| RetryError::NonRetryable(
        std::fmt::Error::new("No error recorded".to_string())
    ))
}

#[derive(Debug)]
pub enum RetryError<E> {
    Retryable(E, u32),
    NonRetryable(E),
}

// =========================================
// CIRCUIT BREAKER PATTERN
// =========================================

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
    last_success: Arc<RwLock<Option<Instant>>>,
    request_count: Arc<RwLock<u64>>,
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
            last_success: Arc::new(RwLock::new(None)),
            request_count: Arc::new(RwLock::new(0)),
        }
    }
    
    pub async fn call<F, T, E>(&self, operation: F) -> Result<T, CircuitBreakerError<E>>
    where
        F: FnOnce() -> Result<T, E>,
    {
        // Increment request count
        *self.request_count.write().await += 1;
        
        let state = *self.state.read().await;
        
        match state {
            CircuitState::Open => {
                let last_failure = *self.last_failure.read().await;
                if let Some(failure_time) = last_failure {
                    if failure_time.elapsed() > self.timeout {
                        println!("Circuit transitioning to half-open after timeout");
                        *self.state.write().await = CircuitState::HalfOpen;
                    } else {
                        return Err(CircuitBreakerError::CircuitOpen);
                    }
                } else {
                    return Err(CircuitBreakerError::CircuitOpen);
                }
            }
            CircuitState::HalfOpen => {
                // Allow one request through in half-open state
                println!("Circuit in half-open state, allowing one request");
            }
            CircuitState::Closed => {
                // Normal operation
            }
        }
        
        match operation() {
            Ok(result) => {
                // Success - reset failure count and potentially close circuit
                *self.failure_count.write().await = 0;
                let mut success_count = self.success_count.write().await;
                *success_count += 1;
                *self.last_success.write().await = Some(Instant::now());
                
                if *success_count >= self.success_threshold {
                    *self.state.write().await = CircuitState::Closed;
                    println!("Circuit closed after {} successes", success_count);
                }
                
                Ok(result)
            }
            Err(error) => {
                // Failure - increment count and potentially open circuit
                let mut failure_count = self.failure_count.write().await;
                *failure_count += 1;
                *self.last_failure.write().await = Some(Instant::now());
                
                if *failure_count >= self.failure_threshold {
                    *self.state.write().await = CircuitState::Open;
                    *self.success_count.write().await = 0;
                    println!("Circuit opened after {} failures", failure_count);
                }
                
                Err(CircuitBreakerError::ServiceError(error))
            }
        }
    }
    
    pub async fn get_state(&self) -> CircuitState {
        *self.state.read().await
    }
    
    pub async fn get_stats(&self) -> CircuitBreakerStats {
        let request_count = *self.request_count.read().await;
        let failure_count = *self.failure_count.read().await;
        let success_count = *self.success_count.read().await;
        
        CircuitBreakerStats {
            request_count,
            failure_count,
            success_count,
            failure_rate: if request_count > 0 {
                failure_count as f64 / request_count as f64
            } else {
                0.0
            },
        }
    }
    
    pub async fn reset(&self) {
        *self.state.write().await = CircuitState::Closed;
        *self.failure_count.write().await = 0;
        *self.success_count.write().await = 0;
        *self.last_failure.write().await = None;
        *self.last_success.write().await = None;
        *self.request_count.write().await = 0;
        println!("Circuit breaker reset");
    }
}

#[derive(Debug)]
pub struct CircuitBreakerStats {
    pub request_count: u64,
    pub failure_count: u32,
    pub success_count: u32,
    pub failure_rate: f64,
}

#[derive(Debug)]
pub enum CircuitBreakerError<E> {
    ServiceError(E),
    CircuitOpen,
}

// =========================================
// ERROR COLLECTION AND ANALYSIS
// =========================================

#[derive(Debug, Clone)]
pub struct ErrorReport {
    pub error_id: String,
    pub error_type: String,
    pub message: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub count: u32,
    pub first_occurrence: chrono::DateTime<chrono::Utc>,
    pub last_occurrence: chrono::DateTime<chrono::Utc>,
    pub context: HashMap<String, String>,
    pub severity: ErrorSeverity,
    pub resolved: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ErrorSeverity {
    Low,
    Medium,
    High,
    Critical,
}

impl ErrorSeverity {
    pub fn from_count(count: u32, duration: Duration) -> Self {
        if count >= 10 || duration > Duration::from_secs(300) {
            ErrorSeverity::Critical
        } else if count >= 5 || duration > Duration::from_secs(60) {
            ErrorSeverity::High
        } else if count >= 2 || duration > Duration::from_secs(10) {
            ErrorSeverity::Medium
        } else {
            ErrorSeverity::Low
        }
    }
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
                
                // Update severity based on count and time since first occurrence
                let duration = now.signed_duration_since(report.first_occurrence).to_std().unwrap();
                report.severity = ErrorSeverity::from_count(report.count, duration);
                
                // Merge context
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
                    first_occurrence: now,
                    last_occurrence: now,
                    context,
                    severity: ErrorSeverity::Low,
                    resolved: false,
                });
            }
        }
    }
    
    pub async fn resolve_error(&self, error_id: &str) {
        let mut errors = self.errors.write().await;
        
        if let Some(report) = errors.get_mut(error_id) {
            report.resolved = true;
            println!("Resolved error: {}", error_id);
        }
    }
    
    pub async fn get_error_summary(&self) -> Vec<ErrorReport> {
        let errors = self.errors.read().await;
        let mut reports: Vec<ErrorReport> = errors.values().cloned().collect();
        
        // Sort by severity and timestamp
        reports.sort_by(|a, b| {
            b.severity.cmp(&a.severity)
                .then_with(|| a.timestamp.cmp(&b.timestamp))
        });
        
        reports
    }
    
    pub async fn get_error_metrics(&self) -> ErrorMetrics {
        let errors = self.errors.read().await;
        let now = chrono::Utc::now();
        
        let mut metrics = ErrorMetrics {
            total_errors: errors.len() as u32,
            errors_by_type: HashMap::new(),
            errors_by_severity: HashMap::new(),
            errors_in_last_hour: 0,
            errors_in_last_day: 0,
            resolved_errors: 0,
            unresolved_errors: 0,
        };
        
        let one_hour_ago = now - chrono::Duration::hours(1);
        let one_day_ago = now - chrono::Duration::days(1);
        
        for report in errors.values() {
            // Count by type
            *metrics.errors_by_type.entry(report.error_type.clone()).or_insert(0) += 1;
            
            // Count by severity
            *metrics.errors_by_severity.entry(format!("{:?}", report.severity)).or_insert(0) += 1;
            
            // Count time-based metrics
            if report.last_occurrence > one_hour_ago {
                metrics.errors_in_last_hour += 1;
            }
            
            if report.last_occurrence > one_day_ago {
                metrics.errors_in_last_day += 1;
            }
            
            if report.resolved {
                metrics.resolved_errors += 1;
            } else {
                metrics.unresolved_errors += 1;
            }
        }
        
        metrics
    }
}

#[derive(Debug)]
pub struct ErrorMetrics {
    pub total_errors: u32,
    pub errors_by_type: HashMap<String, u32>,
    pub errors_by_severity: HashMap<String, u32>,
    pub errors_in_last_hour: u32,
    pub errors_in_last_day: u32,
    pub resolved_errors: u32,
    pub unresolved_errors: u32,
}

// =========================================
// ASYNC ERROR HANDLING
// =========================================

pub type AsyncResult<T> = Result<T, Box<dyn std::error::Error + Send + Sync>>;

// Async operation with timeout
pub async fn async_operation_with_timeout<T>(
    operation: impl std::future::Future<Output = AsyncResult<T>>,
    timeout: Duration,
) -> AsyncResult<T> {
    match tokio::time::timeout(timeout, operation).await {
        Ok(result) => result,
        Err(_) => Err("Operation timed out".into()),
    }
}

// Concurrent error handling
pub async fn concurrent_operations() -> AsyncResult<Vec<String>> {
    let operations = vec![
        async_operation_with_timeout(
            async { Ok("Operation 1".to_string()) },
            Duration::from_millis(100)
        ),
        async_operation_with_timeout(
            async { 
                tokio::time::sleep(Duration::from_millis(200)).await;
                Ok("Operation 2".to_string())
            },
            Duration::from_millis(100)
        ),
        async_operation_with_timeout(
            async {
                tokio::time::sleep(Duration::from_millis(50)).await;
                Err("Operation 3 failed".into())
            },
            Duration::from_millis(100)
        ),
    ];
    
    let results = futures::future::join_all(operations).await;
    
    let mut successful_results = Vec::new();
    let mut errors = Vec::new();
    
    for (index, result) in results.into_iter().enumerate() {
        match result {
            Ok(value) => {
                println!("Operation {} succeeded: {}", index + 1, value);
                successful_results.push(value);
            }
            Err(e) => {
                println!("Operation {} failed: {}", index + 1, e);
                errors.push(e);
            }
        }
    }
    
    if !errors.is_empty() {
        eprintln!("{} operations failed", errors.len());
    }
    
    Ok(successful_results)
}

// Race between operations
pub async fn race_operations() -> AsyncResult<String> {
    let op1 = async_operation_with_timeout(
        async { Ok("Fast operation".to_string()) },
        Duration::from_millis(100)
    );
    
    let op2 = async_operation_with_timeout(
        async {
            tokio::time::sleep(Duration::from_millis(200)).await;
            Ok("Slow operation".to_string())
        },
        Duration::from_millis(100)
    );
    
    tokio::select! {
        result = op1 => {
            println!("Fast operation completed first");
            result
        }
        result = op2 => {
            println!("Slow operation completed first");
            result
        }
    }
}

// =========================================
// ERROR HANDLING UTILITIES
// =========================================

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

// Error handling macro
macro_rules! handle_error {
    ($result:expr, $operation:expr) => {
        match $result {
            Ok(value) => value,
            Err(e) => {
                tracing::error!("{} failed: {}", $operation, e);
                None
            }
        }
    };
}

// Safe unwrap with context
pub trait SafeUnwrap<T> {
    fn safe_unwrap(self, context: &str) -> T;
}

impl<T> SafeUnwrap<T> for Option<T> {
    fn safe_unwrap(self, context: &str) -> T {
        match self {
            Some(value) => value,
            None => {
                tracing::error!("{}: None value", context);
                panic!("{}: None value", context);
            }
        }
    }
}

impl<T, E> SafeUnwrap<T> for Result<T, E> {
    fn safe_unwrap(self, context: &str) -> T {
        match self {
            Ok(value) => value,
            Err(e) => {
                tracing::error!("{}: {}", context, e);
                panic!("{}: {}", context, e);
            }
        }
    }
}

// =========================================
// DEMONSTRATION FUNCTIONS
// =========================================

pub async fn demonstrate_custom_errors() {
    println!("=== CUSTOM ERRORS DEMONSTRATION ===");
    
    // Database errors
    let db_error = DatabaseError::UserNotFound { id: 42 };
    println!("Database error: {}", db_error);
    
    // Validation errors
    let validation_error = ValidationError::InvalidEmail {
        email: "invalid-email".to_string(),
    };
    println!("Validation error: {}", validation_error);
    
    // Processing errors
    let mut proc_error = ProcessingError::new(
        "user_registration".to_string(),
        "validation".to_string(),
        "Email validation failed".to_string(),
    );
    proc_error = proc_error.with_error_code("VALIDATION_FAILED".to_string());
    proc_error = proc_error.with_context("user_id".to_string(), "123".to_string());
    
    println!("Processing error: {}", proc_error);
    println!("Can retry: {}", proc_error.can_retry());
    
    println!();
}

pub async fn demonstrate_error_context() {
    println!("=== ERROR CONTEXT DEMONSTRATION ===");
    
    let context = ErrorContext::new("user_login".to_string())
        .with_user(123)
        .with_metadata("ip".to_string(), "192.168.1.1".to_string())
        .add_stack_frame("authenticate_user".to_string())
        .add_stack_frame("validate_credentials".to_string());
    
    let error = DatabaseError::UserNotFound { id: 123 };
    let contextual_error = ContextualError::new(error, context);
    
    println!("Contextual error: {}", contextual_error);
    
    // Error chain
    let mut chain = ErrorChain::new(context);
    chain = chain.add_error(DatabaseError::ConnectionError(
        sqlx::Error::Protocol("Connection closed".to_string())
    ));
    chain = chain.add_context("retry_count".to_string(), "3".to_string());
    
    println!("Error chain: {}", chain);
    
    println!();
}

pub async fn demonstrate_retry_mechanisms() {
    println!("=== RETRY MECHANISMS DEMONSTRATION ===");
    
    let retry_policy = RetryPolicy::new(RetryConfig::default());
    
    // Simulate operation that fails twice then succeeds
    let mut attempt_count = 0;
    let result = retry_with_policy(&retry_policy, || async move {
        attempt_count += 1;
        println!("Attempt {}", attempt_count);
        
        if attempt_count <= 2 {
            Err("connection_refused".to_string())
        } else {
            Ok("Success!".to_string())
        }
    }).await;
    
    match result {
        Ok(success) => println!("Operation succeeded: {}", success),
        Err(e) => println!("Operation failed: {}", e),
    }
    
    println!();
}

pub async fn demonstrate_circuit_breaker() {
    println!("=== CIRCUIT BREAKER DEMONSTRATION ===");
    
    let circuit_breaker = CircuitBreaker::new(
        3, // failure threshold
        2, // success threshold
        Duration::from_secs(5), // timeout
    );
    
    // Simulate operations that fail then succeed
    for i in 0..8 {
        let result = circuit_breaker.call(|| async move {
            if i < 5 {
                Err(format!("Service error {}", i))
            } else {
                Ok(format!("Success {}", i))
            }
        }).await;
        
        match result {
            Ok(response) => println!("Call {}: {}", i + 1, response),
            Err(e) => println!("Call {}: {}", i + 1, e),
        }
        
        println!("Circuit state: {:?}", circuit_breaker.get_state().await);
        tokio::time::sleep(Duration::from_millis(200)).await;
    }
    
    let stats = circuit_breaker.get_stats().await;
    println!("Circuit breaker stats: {:?}", stats);
    
    println!();
}

pub async fn demonstrate_error_collection() {
    println!("=== ERROR COLLECTION DEMONSTRATION ===");
    
    let collector = ErrorCollector::new();
    
    // Report some errors
    collector.report_error(
        "database".to_string(),
        "Connection failed".to_string(),
        HashMap::from([
            ("host".to_string(), "db.example.com".to_string()),
            ("port".to_string(), "5432".to_string()),
        ]),
    ).await;
    
    collector.report_error(
        "validation".to_string(),
        "Invalid email format".to_string(),
        HashMap::from([
            ("email".to_string(), "user@example.com".to_string()),
        ]),
    ).await;
    
    collector.report_error(
        "validation".to_string(),
        "Invalid email format".to_string(),
        HashMap::from([
            ("email".to_string(), "user2@example.com".to_string()),
        ]),
    ).await;
    
    // Get error summary
    let summary = collector.get_error_summary().await;
    println!("Error summary: {} errors", summary.len());
    
    for report in &summary {
        println!("  {}: {} ({})", 
                 report.error_type, 
                 report.message, 
                 report.count,
                 report.severity);
    }
    
    // Get metrics
    let metrics = collector.get_error_metrics().await;
    println!("Error metrics: {:?}", metrics);
    
    // Resolve an error
    collector.resolve_error("validation:Invalid email format").await;
    
    println!();
}

pub async fn demonstrate_async_error_handling() {
    println!("=== ASYNC ERROR HANDLING DEMONSTRATION ===");
    
    // Timeout example
    let result = async_operation_with_timeout(
        async {
            tokio::time::sleep(Duration::from_millis(200)).await;
            Ok("Operation completed".to_string())
        },
        Duration::from_millis(100),
    ).await;
    
    match result {
        Ok(value) => println!("Operation succeeded: {}", value),
        Err(e) => println!("Operation failed: {}", e),
    }
    
    // Concurrent operations
    let concurrent_results = concurrent_operations().await;
    println!("Concurrent operations: {} successful results", concurrent_results.len());
    
    // Race operations
    let race_result = race_operations().await;
    println!("Race operation result: {}", race_result);
    
    println!();
}

pub async fn demonstrate_error_utilities() {
    println!("=== ERROR HANDLING UTILITIES DEMONSTRATION ===");
    
    // ResultExt usage
    let result: Result<String, DatabaseError> = Err(DatabaseError::UserNotFound { id: 42 });
    
    let logged_result = result.log_error().with_context(|| {
        "While fetching user 42".to_string()
    });
    
    match logged_result {
        Ok(_) => println!("This shouldn't happen"),
        Err(e) => println!("Expected error: {}", e),
    }
    
    // Safe unwrap
    let some_value: Option<i32> = Some(42);
    let unwrapped = some_value.safe_unwrap("Expected some value");
    println!("Safely unwrapped: {}", unwrapped);
    
    // Macro usage
    let error_result: Result<i32, DatabaseError> = Err(DatabaseError::UserNotFound { id: 42 });
    let handled = handle_error!(error_result, "fetching user");
    
    println!("Handled result: {:?}", handled);
    
    println!();
}

// =========================================
// MAIN DEMONSTRATION
// =========================================

#[tokio::main]
async fn main() {
    println!("=== ADVANCED ERROR HANDLING DEMONSTRATIONS ===\n");
    
    demonstrate_custom_errors();
    demonstrate_error_context();
    demonstrate_retry_mechanisms();
    demonstrate_circuit_breaker();
    demonstrate_error_collection();
    demonstrate_async_error_handling();
    demonstrate_error_utilities();
    
    println!("=== ADVANCED ERROR HANDLING DEMONSTRATIONS COMPLETE ===");
    println!("Note: This demonstrates advanced error handling patterns. Real implementations should:");
    println!("- Use thiserror for clean error definitions");
    println!("- Implement proper error context and chaining");
    println!("- Use structured logging with tracing");
    println!("- Implement retry mechanisms with exponential backoff");
    println!("- Use circuit breakers for fault tolerance");
    println!("- Collect and analyze errors for monitoring");
    println!("- Provide clear error messages to users");
    println!("- Follow error handling best practices");
}

// =========================================
// UNIT TESTS
// =========================================

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_custom_errors() {
        let error = DatabaseError::UserNotFound { id: 42 };
        assert!(error.to_string().contains("User not found"));
    }
    
    #[test]
    fn test_validation_errors() {
        let error = ValidationError::InvalidEmail {
            email: "invalid".to_string(),
        };
        assert!(error.to_string().contains("Invalid email format"));
    }
    
    #[test]
    fn test_processing_error() {
        let mut error = ProcessingError::new(
            "test".to_string(),
            "stage1".to_string(),
            "message".to_string(),
        );
        
        assert!(error.can_retry());
        error.increment_retry();
        assert_eq!(error.retry_count, 1);
    }
    
    #[test]
    fn test_circuit_breaker() {
        let circuit = CircuitBreaker::new(2, 2, Duration::from_secs(1));
        
        // Should start in closed state
        assert_eq!(circuit.get_state().await, CircuitState::Closed);
        
        // Simulate failures
        let _ = circuit.call(|| Err("error".to_string())).await;
        assert_eq!(circuit.get_state().await, CircuitState::Closed);
        
        let _ = circuit.call(|| Err("error".to_string())).await;
        assert_eq!(circuit.get_state().await, CircuitState::Open);
        
        // Should fail due to open circuit
        let result = circuit.call(|| Ok("success".to_string())).await;
        assert!(matches!(result, Err(CircuitBreakerError::CircuitOpen)));
    }
    
    #[test]
    fn test_error_collector() {
        let collector = ErrorCollector::new();
        
        collector.report_error(
            "test".to_string(),
            "message".to_string(),
            HashMap::new(),
        ).await;
        
        let summary = collector.get_error_summary().await;
        assert_eq!(summary.len(), 1);
        
        let metrics = collector.get_error_metrics().await;
        assert_eq!(metrics.total_errors, 1);
    }
    
    #[test]
    fn test_retry_policy() {
        let policy = RetryPolicy::new(RetryConfig::default());
        
        assert!(policy.should_retry("timeout", 1));
        assert!(!policy.should_retry("authentication", 1));
        assert!(policy.should_retry("timeout", 4));
        assert!(!policy.should_retry("timeout", 11));
    }
    
    #[test]
    fn test_safe_unwrap() {
        let some_value: Option<i32> = Some(42);
        let unwrapped = some_value.safe_unwrap("test");
        assert_eq!(unwrapped, 42);
        
        let none_value: Option<i32> = None;
        
        // This should panic in debug mode
        std::panic::catch_unwind(|| {
            none_value.safe_unwrap("test");
        }).unwrap_err();
    }
    
    #[test]
    fn test_error_context() {
        let context = ErrorContext::new("test".to_string());
        assert_eq!(context.operation, "test");
        
        let with_user = context.with_user(123);
        assert_eq!(with_user.user_id, Some(123));
    }
}
