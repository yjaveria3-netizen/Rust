// 34_cloud_and_distributed_systems.rs
// Comprehensive examples of cloud and distributed systems in Rust

// Note: This file demonstrates distributed systems concepts but requires proper
// cloud services and infrastructure for production use

use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

// =========================================
// MICROSERVICES
// =========================================

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct User {
    pub id: u32,
    pub name: String,
    pub email: String,
}

#[derive(Debug, serde::Deserialize)]
pub struct CreateUser {
    pub name: String,
    pub email: String,
}

pub type AppState = Arc<Mutex<HashMap<u32, User>>>;

// Simulated HTTP microservice
pub struct UserService {
    users: AppState,
    port: u16,
}

impl UserService {
    pub fn new(port: u16) -> Self {
        UserService {
            users: Arc::new(Mutex::new(HashMap::new())),
            port,
        }
    }
    
    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("User service starting on port {}", self.port);
        
        // Simulate service startup
        tokio::spawn(async move {
            // Simulate handling requests
            let mut request_count = 0;
            loop {
                tokio::time::sleep(Duration::from_millis(100)).await;
                request_count += 1;
                
                if request_count % 10 == 0 {
                    println!("User service: handled {} requests", request_count);
                }
            }
        });
        
        Ok(())
    }
    
    pub async fn create_user(&self, user_data: CreateUser) -> Result<User, String> {
        let mut users = self.users.lock().unwrap();
        let new_id = users.len() as u32 + 1;
        
        let new_user = User {
            id: new_id,
            name: user_data.name,
            email: user_data.email,
        };
        
        users.insert(new_id, new_user.clone());
        println!("Created user: {:?}", new_user);
        
        Ok(new_user)
    }
    
    pub async fn get_user(&self, user_id: u32) -> Result<User, String> {
        let users = self.users.lock().unwrap();
        
        match users.get(&user_id) {
            Some(user) => {
                println!("Retrieved user: {:?}", user);
                Ok(user.clone())
            }
            None => Err(format!("User {} not found", user_id)),
        }
    }
    
    pub async fn list_users(&self) -> Vec<User> {
        let users = self.users.lock().unwrap();
        let user_list: Vec<User> = users.values().cloned().collect();
        println!("Retrieved {} users", user_list.len());
        user_list
    }
}

// =========================================
// SERVICE DISCOVERY
// =========================================

#[derive(Debug, Clone)]
pub struct ServiceRegistration {
    pub id: String,
    pub name: String,
    pub address: String,
    pub port: u16,
    pub tags: Vec<String>,
    pub metadata: HashMap<String, String>,
    pub health_check_url: String,
}

#[derive(Debug, Clone)]
pub struct ServiceInstance {
    pub registration: ServiceRegistration,
    pub last_heartbeat: Instant,
    pub status: ServiceStatus,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ServiceStatus {
    Healthy,
    Unhealthy,
    Unknown,
}

pub struct ServiceRegistry {
    services: Arc<RwLock<HashMap<String, Vec<ServiceInstance>>>>,
    health_check_interval: Duration,
}

impl ServiceRegistry {
    pub fn new(health_check_interval: Duration) -> Self {
        ServiceRegistry {
            services: Arc::new(RwLock::new(HashMap::new())),
            health_check_interval,
        }
    }
    
    pub async fn register_service(&self, registration: ServiceRegistration) -> Result<(), String> {
        let mut services = self.services.write().await;
        let service_name = registration.name.clone();
        
        let instance = ServiceInstance {
            registration,
            last_heartbeat: Instant::now(),
            status: ServiceStatus::Unknown,
        };
        
        let service_list = services.entry(service_name).or_insert_with(Vec::new);
        service_list.push(instance);
        
        println!("Registered service: {}", service_name);
        Ok(())
    }
    
    pub async fn deregister_service(&self, service_id: &str, service_name: &str) -> Result<(), String> {
        let mut services = self.services.write().await;
        
        if let Some(service_list) = services.get_mut(service_name) {
            service_list.retain(|instance| instance.registration.id != service_id);
            println!("Deregistered service instance: {}", service_id);
            Ok(())
        } else {
            Err(format!("Service {} not found", service_name))
        }
    }
    
    pub async fn discover_services(&self, service_name: &str) -> Vec<ServiceInstance> {
        let services = self.services.read().await;
        
        if let Some(service_list) = services.get(service_name) {
            service_list.clone()
        } else {
            Vec::new()
        }
    }
    
    pub async fn heartbeat(&self, service_id: &str, service_name: &str) -> Result<(), String> {
        let mut services = self.services.write().await;
        
        if let Some(service_list) = services.get_mut(service_name) {
            if let Some(instance) = service_list.iter_mut().find(|i| i.registration.id == service_id) {
                instance.last_heartbeat = Instant::now();
                instance.status = ServiceStatus::Healthy;
                println!("Heartbeat received for service: {}", service_id);
                Ok(())
            } else {
                Err(format!("Service instance {} not found", service_id))
            }
        } else {
            Err(format!("Service {} not found", service_name))
        }
    }
    
    pub async fn start_health_checks(&self) {
        let services = self.services.clone();
        let interval = self.health_check_interval;
        
        tokio::spawn(async move {
            let mut interval_timer = tokio::time::interval(interval);
            
            loop {
                interval_timer.tick().await;
                
                let mut services_guard = services.write().await;
                
                for (service_name, service_list) in services_guard.iter_mut() {
                    for instance in service_list.iter_mut() {
                        let elapsed = instance.last_heartbeat.elapsed();
                        
                        if elapsed > Duration::from_secs(30) {
                            instance.status = ServiceStatus::Unhealthy;
                            println!("Service {} marked as unhealthy (no heartbeat for {:?})", 
                                     service_name, elapsed);
                        }
                    }
                }
            }
        });
    }
}

// =========================================
// MESSAGE QUEUES
// =========================================

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Message {
    pub id: String,
    pub payload: String,
    pub timestamp: u64,
    pub headers: HashMap<String, String>,
}

impl Message {
    pub fn new(payload: String) -> Self {
        Message {
            id: uuid::Uuid::new_v4().to_string(),
            payload,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            headers: HashMap::new(),
        }
    }
    
    pub fn with_headers(mut self, headers: HashMap<String, String>) -> Self {
        self.headers = headers;
        self
    }
}

pub struct MessageQueue {
    messages: Arc<Mutex<VecDeque<Message>>>,
    max_size: usize,
}

impl MessageQueue {
    pub fn new(max_size: usize) -> Self {
        MessageQueue {
            messages: Arc::new(Mutex::new(VecDeque::new())),
            max_size,
        }
    }
    
    pub async fn enqueue(&self, message: Message) -> Result<(), String> {
        let mut queue = self.messages.lock().unwrap();
        
        if queue.len() >= self.max_size {
            return Err("Queue is full".to_string());
        }
        
        queue.push_back(message);
        println!("Enqueued message: {}", message.id);
        Ok(())
    }
    
    pub async fn dequeue(&self) -> Option<Message> {
        let mut queue = self.messages.lock().unwrap();
        
        if let Some(message) = queue.pop_front() {
            println!("Dequeued message: {}", message.id);
            Some(message)
        } else {
            None
        }
    }
    
    pub async fn peek(&self) -> Option<Message> {
        let queue = self.messages.lock().unwrap();
        queue.front().cloned()
    }
    
    pub async fn size(&self) -> usize {
        let queue = self.messages.lock().unwrap();
        queue.len()
    }
    
    pub async fn is_empty(&self) -> bool {
        self.size().await == 0
    }
}

// Simulated Kafka producer
pub struct KafkaProducer {
    topic: String,
    queue: MessageQueue,
}

impl KafkaProducer {
    pub fn new(topic: String, queue_size: usize) -> Self {
        KafkaProducer {
            topic,
            queue: MessageQueue::new(queue_size),
        }
    }
    
    pub async fn send(&self, message: Message) -> Result<(), String> {
        self.queue.enqueue(message).await
    }
    
    pub async fn start_producer(&self) {
        let topic = self.topic.clone();
        let queue = self.queue.clone();
        
        tokio::spawn(async move {
            loop {
                if let Some(message) = queue.dequeue().await {
                    // Simulate sending to Kafka
                    println!("Producing to topic {}: {:?}", topic, message);
                    tokio::time::sleep(Duration::from_millis(10)).await;
                } else {
                    tokio::time::sleep(Duration::from_millis(100)).await;
                }
            }
        });
    }
}

// Simulated Kafka consumer
pub struct KafkaConsumer {
    topic: String,
    queue: MessageQueue,
}

impl KafkaConsumer {
    pub fn new(topic: String, queue_size: usize) -> Self {
        KafkaConsumer {
            topic,
            queue: MessageQueue::new(queue_size),
        }
    }
    
    pub async fn start_consumer(&self) {
        let topic = self.topic.clone();
        let queue = self.queue.clone();
        
        tokio::spawn(async move {
            let mut message_id = 1;
            
            loop {
                // Simulate receiving from Kafka
                let message = Message::new(format!("Message {}", message_id));
                
                if let Err(e) = queue.enqueue(message).await {
                    println!("Failed to enqueue message: {}", e);
                }
                
                message_id += 1;
                tokio::time::sleep(Duration::from_millis(50)).await;
            }
        });
    }
    
    pub async fn consume(&self) -> Option<Message> {
        self.queue.dequeue().await
    }
}

// =========================================
// DISTRIBUTED CACHING
// =========================================

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CacheEntry<T> {
    pub value: T,
    pub timestamp: Instant,
    pub ttl: Option<Duration>,
    pub access_count: u64,
}

impl<T> CacheEntry<T> {
    pub fn new(value: T, ttl: Option<Duration>) -> Self {
        CacheEntry {
            value,
            timestamp: Instant::now(),
            ttl,
            access_count: 0,
        }
    }
    
    pub fn is_expired(&self) -> bool {
        if let Some(ttl) = self.ttl {
            self.timestamp.elapsed() > ttl
        } else {
            false
        }
    }
    
    pub fn access(&mut self) -> &T {
        self.access_count += 1;
        &self.value
    }
}

pub struct DistributedCache<T> {
    entries: Arc<RwLock<HashMap<String, CacheEntry<T>>>>,
    default_ttl: Duration,
    max_size: usize,
}

impl<T: Clone> DistributedCache<T> {
    pub fn new(default_ttl: Duration, max_size: usize) -> Self {
        DistributedCache {
            entries: Arc::new(RwLock::new(HashMap::new())),
            default_ttl,
            max_size,
        }
    }
    
    pub async fn get(&self, key: &str) -> Option<T> {
        let mut entries = self.entries.write().await;
        
        if let Some(entry) = entries.get_mut(key) {
            if entry.is_expired() {
                entries.remove(key);
                None
            } else {
                Some(entry.access().clone())
            }
        } else {
            None
        }
    }
    
    pub async fn put(&self, key: String, value: T) -> Option<T> {
        let mut entries = self.entries.write().await;
        
        // Check size limit
        if entries.len() >= self.max_size {
            // Evict oldest entry (simplified LRU)
            if let Some(oldest_key) = entries.keys().next().cloned() {
                entries.remove(&oldest_key);
            }
        }
        
        let old_value = entries.get(&key).map(|entry| entry.value.clone());
        
        entries.insert(key, CacheEntry::new(value, Some(self.default_ttl)));
        
        old_value
    }
    
    pub async fn remove(&self, key: &str) -> Option<T> {
        let mut entries = self.entries.write().await;
        
        entries.remove(key).map(|entry| entry.value)
    }
    
    pub async fn clear(&self) {
        let mut entries = self.entries.write().await;
        entries.clear();
    }
    
    pub async fn size(&self) -> usize {
        let entries = self.entries.read().await;
        entries.len()
    }
    
    pub async fn cleanup_expired(&self) {
        let mut entries = self.entries.write().await;
        
        let expired_keys: Vec<String> = entries.iter()
            .filter(|(_, entry)| entry.is_expired())
            .map(|(key, _)| key.clone())
            .collect();
        
        for key in expired_keys {
            entries.remove(&key);
        }
        
        if !expired_keys.is_empty() {
            println!("Cleaned up {} expired cache entries", expired_keys.len());
        }
    }
}

// =========================================
// LOAD BALANCING
// =========================================

#[derive(Debug, Clone)]
pub struct BackendServer {
    pub id: String,
    pub address: String,
    pub port: u16,
    pub weight: u32,
    pub current_connections: Arc<Mutex<u32>>,
    pub is_healthy: Arc<Mutex<bool>>,
}

impl BackendServer {
    pub fn new(id: String, address: String, port: u16, weight: u32) -> Self {
        BackendServer {
            id,
            address,
            port,
            weight,
            current_connections: Arc::new(Mutex::new(0)),
            is_healthy: Arc::new(Mutex::new(true)),
        }
    }
    
    pub fn url(&self) -> String {
        format!("http://{}:{}", self.address, self.port)
    }
    
    pub async fn health_check(&self) -> bool {
        // Simulate health check
        let is_healthy = *self.is_healthy.lock().unwrap();
        println!("Health check for {}: {}", self.id, is_healthy);
        is_healthy
    }
    
    pub async fn increment_connections(&self) {
        let mut connections = self.current_connections.lock().unwrap();
        *connections += 1;
    }
    
    pub async fn decrement_connections(&self) {
        let mut connections = self.current_connections.lock().unwrap();
        *connections = connections.saturating_sub(1);
    }
    
    pub async fn get_connections(&self) -> u32 {
        *self.current_connections.lock().unwrap()
    }
}

pub trait LoadBalancer: Send + Sync {
    async fn get_next_backend(&self) -> Option<BackendServer>;
    async fn add_backend(&self, backend: BackendServer);
    async fn remove_backend(&self, backend_id: &str);
}

pub struct RoundRobinBalancer {
    backends: Arc<RwLock<VecDeque<BackendServer>>>,
}

impl RoundRobinBalancer {
    pub fn new() -> Self {
        RoundRobinBalancer {
            backends: Arc::new(RwLock::new(VecDeque::new())),
        }
    }
}

#[async_trait::async_trait]
impl LoadBalancer for RoundRobinBalancer {
    async fn get_next_backend(&self) -> Option<BackendServer> {
        let mut backends = self.backends.write().await;
        
        if backends.is_empty() {
            return None;
        }
        
        // Move first backend to end
        if let Some(backend) = backends.pop_front() {
            backends.push_back(backend);
            Some(backend)
        } else {
            None
        }
    }
    
    async fn add_backend(&self, backend: BackendServer) {
        let mut backends = self.backends.write().await;
        backends.push_back(backend);
        println!("Added backend {} to round-robin pool", backend.id);
    }
    
    async fn remove_backend(&self, backend_id: &str) {
        let mut backends = self.backends.write().await;
        backends.retain(|backend| backend.id != backend_id);
        println!("Removed backend {} from round-robin pool", backend_id);
    }
}

pub struct LeastConnectionsBalancer {
    backends: Arc<RwLock<Vec<BackendServer>>>,
}

impl LeastConnectionsBalancer {
    pub fn new() -> Self {
        LeastConnectionsBalancer {
            backends: Arc::new(RwLock::new(Vec::new())),
        }
    }
}

#[async_trait::async_trait]
impl LoadBalancer for LeastConnectionsBalancer {
    async fn get_next_backend(&self) -> Option<BackendServer> {
        let backends = self.backends.read().await;
        
        if backends.is_empty() {
            return None;
        }
        
        // Find backend with least connections
        let mut best_backend = None;
        let mut min_connections = u32::MAX;
        
        for backend in backends.iter() {
            let connections = backend.get_connections().await;
            if connections < min_connections {
                min_connections = connections;
                best_backend = Some(backend.clone());
            }
        }
        
        best_backend
    }
    
    async fn add_backend(&self, backend: BackendServer) {
        let mut backends = self.backends.write().await;
        backends.push(backend);
        println!("Added backend {} to least-connections pool", backend.id);
    }
    
    async fn remove_backend(&self, backend_id: &str) {
        let mut backends = self.backends.write().await;
        backends.retain(|backend| backend.id != backend_id);
        println!("Removed backend {} from least-connections pool", backend_id);
    }
}

// Load balanced service
pub struct LoadBalancedService {
    balancer: Arc<dyn LoadBalancer>,
}

impl LoadBalancedService {
    pub fn new(balancer: Arc<dyn LoadBalancer>) -> Self {
        LoadBalancedService { balancer }
    }
    
    pub async fn handle_request(&self) -> Result<String, String> {
        if let Some(backend) = self.balancer.get_next_backend().await {
            if !backend.health_check().await {
                return Err("Backend is unhealthy".to_string());
            }
            
            backend.increment_connections().await;
            println!("Routing request to backend: {}", backend.url());
            
            // Simulate request processing
            let response = format!("Response from {}", backend.id);
            
            backend.decrement_connections().await;
            Ok(response)
        } else {
            Err("No available backends".to_string())
        }
    }
}

// =========================================
// CIRCUIT BREAKER
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
        }
    }
    
    pub async fn call<F, T, E>(&self, f: F) -> Result<T, CircuitBreakerError<E>>
    where
        F: FnOnce() -> Result<T, E>,
    {
        let state = *self.state.read().await;
        
        match state {
            CircuitState::Open => {
                // Check if timeout has passed
                let last_failure = *self.last_failure.read().await;
                if let Some(failure_time) = last_failure {
                    if failure_time.elapsed() > self.timeout {
                        // Try half-open state
                        *self.state.write().await = CircuitState::HalfOpen;
                        println!("Circuit breaker transitioning to half-open");
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
        
        // Execute the function
        match f() {
            Ok(result) => {
                // Success - reset failure count and potentially close circuit
                *self.failure_count.write().await = 0;
                *self.last_success.write().await = Some(Instant::now());
                
                let mut success_count = self.success_count.write().await;
                *success_count += 1;
                
                if *success_count >= self.success_threshold {
                    *self.state.write().await = CircuitState::Closed;
                    println!("Circuit breaker closed after {} successes", success_count);
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
                    println!("Circuit breaker opened after {} failures", failure_count);
                }
                
                Err(CircuitBreakerError::ServiceError(error))
            }
        }
    }
    
    pub async fn get_state(&self) -> CircuitState {
        *self.state.read().await
    }
    
    pub async fn reset(&self) {
        *self.state.write().await = CircuitState::Closed;
        *self.failure_count.write().await = 0;
        *self.success_count.write().await = 0;
        *self.last_failure.write().await = None;
        *self.last_success.write().await = None;
        println!("Circuit breaker reset");
    }
}

#[derive(Debug)]
pub enum CircuitBreakerError<E> {
    ServiceError(E),
    CircuitOpen,
}

impl<E> std::fmt::Display for CircuitBreakerError<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            CircuitBreakerError::ServiceError(e) => write!(f, "Service error: {:?}", e),
            CircuitBreakerError::CircuitOpen => write!(f, "Circuit breaker is open"),
        }
    }
}

// =========================================
// DISTRIBUTED TRACING
// =========================================

#[derive(Debug, Clone)]
pub struct Span {
    pub trace_id: String,
    pub span_id: String,
    pub parent_span_id: Option<String>,
    pub operation_name: String,
    pub start_time: Instant,
    pub end_time: Option<Instant>,
    pub tags: HashMap<String, String>,
    pub logs: Vec<String>,
}

impl Span {
    pub fn new(trace_id: String, span_id: String, operation_name: String) -> Self {
        Span {
            trace_id,
            span_id,
            parent_span_id: None,
            operation_name,
            start_time: Instant::now(),
            end_time: None,
            tags: HashMap::new(),
            logs: Vec::new(),
        }
    }
    
    pub fn with_parent(mut self, parent_span_id: String) -> Self {
        self.parent_span_id = Some(parent_span_id);
        self
    }
    
    pub fn add_tag(mut self, key: String, value: String) -> Self {
        self.tags.insert(key, value);
        self
    }
    
    pub fn add_log(mut self, message: String) -> Self {
        self.logs.push(message);
        self
    }
    
    pub fn finish(&mut self) {
        self.end_time = Some(Instant::now());
        let duration = self.end_time.unwrap().duration_since(self.start_time);
        println!("Span {} completed in {:?}", self.operation_name, duration);
    }
}

pub struct DistributedTracer {
    spans: Arc<Mutex<HashMap<String, Span>>>,
}

impl DistributedTracer {
    pub fn new() -> Self {
        DistributedTracer {
            spans: Arc::new(Mutex::new(HashMap::new())),
        }
    }
    
    pub fn start_span(&self, operation_name: String) -> Span {
        let trace_id = uuid::Uuid::new_v4().to_string();
        let span_id = uuid::Uuid::new_v4().to_string();
        
        let span = Span::new(trace_id, span_id, operation_name);
        
        let mut spans = self.spans.lock().unwrap();
        spans.insert(span_id.clone(), span.clone());
        
        println!("Started span: {} ({})", operation_name, span_id);
        span
    }
    
    pub fn finish_span(&self, span_id: &str) {
        let mut spans = self.spans.lock().unwrap();
        
        if let Some(span) = spans.get_mut(span_id) {
            span.finish();
        }
    }
    
    pub fn get_span(&self, span_id: &str) -> Option<Span> {
        let spans = self.spans.lock().unwrap();
        spans.get(span_id).cloned()
    }
}

// =========================================
// DEMONSTRATION FUNCTIONS
// =========================================

pub async fn demonstrate_microservices() {
    println!("=== MICROSERVICES DEMONSTRATION ===");
    
    let user_service = UserService::new(3001);
    user_service.start().await.unwrap();
    
    // Create some users
    let user1 = user_service.create_user(CreateUser {
        name: "Alice".to_string(),
        email: "alice@example.com".to_string(),
    }).await.unwrap();
    
    let user2 = user_service.create_user(CreateUser {
        name: "Bob".to_string(),
        email: "bob@example.com".to_string(),
    }).await.unwrap();
    
    // Retrieve users
    let _ = user_service.get_user(user1.id).await.unwrap();
    let users = user_service.list_users().await;
    
    println!("Total users: {}", users.len());
    println!();
}

pub async fn demonstrate_service_discovery() {
    println!("=== SERVICE DISCOVERY DEMONSTRATION ===");
    
    let registry = ServiceRegistry::new(Duration::from_secs(30));
    registry.start_health_checks().await;
    
    // Register services
    let user_service_reg = ServiceRegistration {
        id: "user-service-1".to_string(),
        name: "user-service".to_string(),
        address: "127.0.0.1".to_string(),
        port: 3001,
        tags: vec!["v1".to_string(), "production".to_string()],
        metadata: HashMap::from([
            ("version".to_string(), "1.0.0".to_string()),
            ("environment".to_string(), "production".to_string()),
        ]),
        health_check_url: "http://127.0.0.1:3001/health".to_string(),
    };
    
    registry.register_service(user_service_reg).await.unwrap();
    
    let order_service_reg = ServiceRegistration {
        id: "order-service-1".to_string(),
        name: "order-service".to_string(),
        address: "127.0.0.1".to_string(),
        port: 3002,
        tags: vec!["v1".to_string(), "production".to_string()],
        metadata: HashMap::from([
            ("version".to_string(), "1.0.0".to_string()),
            ("environment".to_string(), "production".to_string()),
        ]),
        health_check_url: "http://127.0.0.1:3002/health".to_string(),
    };
    
    registry.register_service(order_service_reg).await.unwrap();
    
    // Discover services
    let user_services = registry.discover_services("user-service").await;
    println!("Found {} user service instances", user_services.len());
    
    let order_services = registry.discover_services("order-service").await;
    println!("Found {} order service instances", order_services.len());
    
    // Send heartbeats
    registry.heartbeat("user-service-1", "user-service").await.unwrap();
    registry.heartbeat("order-service-1", "order-service").await.unwrap();
    
    tokio::time::sleep(Duration::from_secs(2)).await;
    
    println!();
}

pub async fn demonstrate_message_queue() {
    println!("=== MESSAGE QUEUE DEMONSTRATION ===");
    
    let producer = KafkaProducer::new("user-events".to_string(), 100);
    let consumer = KafkaConsumer::new("user-events".to_string(), 100);
    
    producer.start_producer();
    consumer.start_consumer();
    
    // Send some messages
    for i in 1..=5 {
        let message = Message::new(format!("User event {}", i));
        producer.send(message).await.unwrap();
    }
    
    // Consume messages
    tokio::time::sleep(Duration::from_millis(500)).await;
    
    for _ in 0..5 {
        if let Some(message) = consumer.consume().await {
            println!("Consumed: {:?}", message);
        }
    }
    
    println!();
}

pub async fn demonstrate_distributed_cache() {
    println!("=== DISTRIBUTED CACHE DEMONSTRATION ===");
    
    let cache: DistributedCache<String> = DistributedCache::new(
        Duration::from_secs(60), // 1 minute TTL
        100, // Max 100 entries
    );
    
    // Put some values
    cache.put("user:1".to_string(), "Alice".to_string()).await;
    cache.put("user:2".to_string(), "Bob".to_string()).await;
    cache.put("user:3".to_string(), "Charlie".to_string()).await;
    
    println!("Cache size: {}", cache.size().await);
    
    // Get values
    if let Some(user) = cache.get("user:1").await {
        println!("User 1: {}", user);
    }
    
    if let Some(user) = cache.get("user:2").await {
        println!("User 2: {}", user);
    }
    
    // Update existing value
    let old_value = cache.put("user:1".to_string(), "Alice Smith".to_string()).await;
    println!("Old value: {:?}", old_value);
    
    if let Some(user) = cache.get("user:1").await {
        println!("Updated User 1: {}", user);
    }
    
    // Remove value
    let removed = cache.remove("user:2").await;
    println!("Removed user 2: {:?}", removed);
    
    println!("Final cache size: {}", cache.size().await);
    
    println!();
}

pub async fn demonstrate_load_balancing() {
    println!("=== LOAD BALANCING DEMONSTRATION ===");
    
    let rr_balancer: Arc<dyn LoadBalancer> = Arc::new(RoundRobinBalancer::new());
    let lc_balancer: Arc<dyn LoadBalancer> = Arc::new(LeastConnectionsBalancer::new());
    
    // Add backends to round-robin balancer
    rr_balancer.add_backend(BackendServer::new(
        "backend-1".to_string(),
        "127.0.0.1".to_string(),
        8081,
        1,
    )).await;
    
    rr_balancer.add_backend(BackendServer::new(
        "backend-2".to_string(),
        "127.0.0.1".to_string(),
        8082,
        1,
    )).await);
    
    // Add backends to least-connections balancer
    lc_balancer.add_backend(BackendServer::new(
        "backend-3".to_string(),
        "127.0.0.1".to_string(),
        8083,
        1,
    )).await);
    
    lc_balancer.add_backend(BackendServer::new(
        "backend-4".to_string(),
        "127.0.0.1".to_string(),
        8084,
        1,
    )).await);
    
    let rr_service = LoadBalancedService::new(rr_balancer);
    let lc_service = LoadBalancedService::new(lc_balancer);
    
    // Test round-robin
    println!("Round-robin load balancing:");
    for i in 0..6 {
        if let Ok(response) = rr_service.handle_request().await {
            println!("Request {}: {}", i + 1, response);
        }
    }
    
    // Test least-connections
    println!("Least-connections load balancing:");
    for i in 0..6 {
        if let Ok(response) = lc_service.handle_request().await {
            println!("Request {}: {}", i + 1, response);
        }
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
    
    // Simulate some calls
    for i in 0..10 {
        let result = circuit_breaker.call(|| {
            if i % 3 == 0 {
                // Simulate failure
                Err("Service error".to_string())
            } else {
                Ok(format!("Success {}", i))
            }
        }).await;
        
        match result {
            Ok(response) => println!("Call {}: {}", i + 1, response),
            Err(e) => println!("Call {}: {}", i + 1, e),
        }
        
        println!("Circuit state: {:?}", circuit_breaker.get_state().await);
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    
    // Reset circuit breaker
    circuit_breaker.reset().await;
    println!("Circuit breaker reset");
    
    println!();
}

pub async fn demonstrate_distributed_tracing() {
    println!("=== DISTRIBUTED TRACING DEMONSTRATION ===");
    
    let tracer = DistributedTracer::new();
    
    // Start a root span
    let root_span = tracer.start_span("user-registration".to_string());
    
    // Start a child span
    let validation_span = tracer.start_span("validation".to_string())
        .with_parent(root_span.span_id.clone())
        .add_tag("step".to_string(), "1".to_string());
    
    // Simulate validation work
    tokio::time::sleep(Duration::from_millis(50)).await;
    
    tracer.finish_span(&validation_span.span_id);
    
    // Start another child span
    let database_span = tracer.start_span("database-insert".to_string())
        .with_parent(root_span.span_id.clone())
        .add_tag("step".to_string(), "2".to_string())
        .add_log("Inserting user into database".to_string());
    
    // Simulate database work
    tokio::time::sleep(Duration::from_millis(100)).await);
    
    tracer.finish_span(&database_span.span_id);
    
    // Finish root span
    tracer.finish_span(&root_span.span_id);
    
    println!();
}

// =========================================
// MAIN DEMONSTRATION
// =========================================

#[tokio::main]
async fn main() {
    println!("=== CLOUD AND DISTRIBUTED SYSTEMS DEMONSTRATIONS ===\n");
    
    demonstrate_microservices();
    demonstrate_service_discovery();
    demonstrate_message_queue();
    demonstrate_distributed_cache();
    demonstrate_load_balancing();
    demonstrate_circuit_breaker();
    demonstrate_distributed_tracing();
    
    println!("=== CLOUD AND DISTRIBUTED SYSTEMS DEMONSTRATIONS COMPLETE ===");
    println!("Note: This uses simulated distributed systems. Real implementations should:");
    println!("- Use proper cloud SDKs (AWS, GCP, Azure)");
    println!("- Implement actual service discovery (Consul, etcd, Kubernetes)");
    println!("- Use real message brokers (Kafka, RabbitMQ, NATS)");
    println!("- Implement proper distributed caching (Redis, Memcached)");
    println!("- Use actual load balancing algorithms and health checks");
    println!("- Integrate with distributed tracing systems (Jaeger, Zipkin)");
    println!("- Follow cloud-native best practices and patterns");
}

// =========================================
// UNIT TESTS
// =========================================

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_user_service() {
        let service = UserService::new(3000);
        
        let user = service.create_user(CreateUser {
            name: "Test User".to_string(),
            email: "test@example.com".to_string(),
        }).await.unwrap();
        
        let retrieved = service.get_user(user.id).await.unwrap();
        assert_eq!(retrieved.name, "Test User");
        assert_eq!(retrieved.email, "test@example.com");
    }
    
    #[tokio::test]
    async fn test_service_registry() {
        let registry = ServiceRegistry::new(Duration::from_secs(30));
        
        let registration = ServiceRegistration {
            id: "test-service".to_string(),
            name: "test".to_string(),
            address: "127.0.0.1".to_string(),
            port: 3000,
            tags: vec!["test".to_string()],
            metadata: HashMap::new(),
            health_check_url: "http://127.0.0.1:3000/health".to_string(),
        };
        
        registry.register_service(registration).await.unwrap();
        
        let services = registry.discover_services("test").await;
        assert_eq!(services.len(), 1);
        assert_eq!(services[0].registration.id, "test-service");
    }
    
    #[tokio::test]
    async fn test_message_queue() {
        let queue = MessageQueue::new(5);
        
        let message = Message::new("test message".to_string());
        assert!(queue.enqueue(message).await.is_ok());
        
        let dequeued = queue.dequeue().await.unwrap();
        assert_eq!(dequeued.payload, "test message");
        assert_eq!(dequeued.id, message.id);
    }
    
    #[tokio::test]
    async fn test_distributed_cache() {
        let cache: DistributedCache<String> = DistributedCache::new(
            Duration::from_secs(60),
            10,
        );
        
        let old_value = cache.put("key1".to_string(), "value1".to_string()).await;
        assert!(old_value.is_none());
        
        let value = cache.get("key1").await.unwrap();
        assert_eq!(value, "value1");
        
        let old_value = cache.put("key1".to_string(), "value2".to_string()).await;
        assert_eq!(old_value.unwrap(), "value1");
        
        let updated_value = cache.get("key1").await.unwrap();
        assert_eq!(updated_value, "value2");
    }
    
    #[tokio::test]
    async fn test_round_robin_balancer() {
        let balancer = RoundRobinBalancer::new();
        
        let backend1 = BackendServer::new("b1".to_string(), "127.0.0.1".to_string(), 8081, 1);
        let backend2 = BackendServer::new("b2".to_string(), "127.0.0.1".to_string(), 8082, 1);
        
        balancer.add_backend(backend1).await;
        balancer.add_backend(backend2).await;
        
        let first = balancer.get_next_backend().await.unwrap();
        let second = balancer.get_next_backend().await.unwrap();
        let third = balancer.get_next_backend().await.unwrap();
        
        assert_eq!(first.id, "b1");
        assert_eq!(second.id, "b2");
        assert_eq!(third.id, "b1"); // Should wrap around
    }
    
    #[tokio::test]
    async fn test_circuit_breaker() {
        let breaker = CircuitBreaker::new(2, 2, Duration::from_secs(1));
        
        // First call succeeds
        let result1 = breaker.call(|| Ok("success1")).await;
        assert!(result1.is_ok());
        assert_eq!(breaker.get_state().await, CircuitState::Closed);
        
        // Second call fails
        let result2 = breaker.call(|| Err("error1".to_string())).await;
        assert!(result2.is_err());
        assert_eq!(breaker.get_state().await, CircuitState::Closed);
        
        // Third call fails - should open circuit
        let result3 = breaker.call(|| Err("error2".to_string())).await;
        assert!(result3.is_err());
        assert_eq!(breaker.get_state().await, CircuitState::Open);
        
        // Fourth call should fail due to open circuit
        let result4 = breaker.call(|| Ok("success2")).await;
        assert!(matches!(result4, Err(CircuitBreakerError::CircuitOpen)));
    }
    
    #[tokio::test]
    async fn test_distributed_tracer() {
        let tracer = DistributedTracer::new();
        
        let span = tracer.start_span("test-operation".to_string());
        tracer.finish_span(&span.span_id);
        
        let retrieved = tracer.get_span(&span.span_id).unwrap();
        assert_eq!(retrieved.operation_name, "test-operation");
        assert!(retrieved.end_time.is_some());
    }
}
