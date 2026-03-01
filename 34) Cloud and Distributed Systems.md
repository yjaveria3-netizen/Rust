# Cloud and Distributed Systems in Rust

## Overview

Rust's performance, safety, and concurrency features make it excellent for cloud-native and distributed systems. This guide covers microservices, distributed computing, cloud platforms, and building scalable systems in Rust.

---

## Cloud Ecosystem

### Core Cloud/Distributed Crates

| Crate | Purpose | Features |
|-------|---------|----------|
| `tokio` | Async runtime | High-performance async I/O |
| `axum` | Web framework | HTTP services and APIs |
| `tonic` | gRPC framework | High-performance RPC |
| `k8s-openapi` | Kubernetes | Kubernetes client |
| `aws-sdk-rust` | AWS SDK | AWS services integration |
| `serde` | Serialization | JSON, protobuf, etc. |
| `tracing` | Structured logging | Distributed tracing |
| `tower` | Middleware | HTTP middleware stack |
| `consul` | Service discovery | Consul integration |
| `etcd-rs` | Distributed KV store | etcd client |

### Choosing the Right Cloud Tools

- **tokio** - Foundation for async cloud services
- **axum** - Modern HTTP web framework
- **tonic** - gRPC for microservices
- **k8s-openapi** - Kubernetes orchestration
- **aws-sdk-rust** - AWS cloud services

---

## Microservices Architecture

### HTTP Microservice with Axum

```rust
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;

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

type AppState = Arc<Mutex<HashMap<u32, User>>>;

#[tokio::main]
async fn main() {
    let state: AppState = Arc::new(Mutex::new(HashMap::new()));
    
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/users", get(list_users))
        .route("/users/:id", get(get_user))
        .route("/users", post(create_user))
        .with_state(state)
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
        );
    
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("Microservice listening on 0.0.0.0:3000");
    
    axum::serve(listener, app).await.unwrap();
}

async fn health_check() -> &'static str {
    "OK"
}

async fn list_users(State(state): State<AppState>) -> Json<Vec<User>> {
    let users = state.lock().unwrap();
    let user_list: Vec<User> = users.values().cloned().collect();
    Json(user_list)
}

async fn get_user(
    Path(user_id): Path<u32>,
    State(state): State<AppState>,
) -> Result<Json<User>, StatusCode> {
    let users = state.lock().unwrap();
    
    match users.get(&user_id) {
        Some(user) => Ok(Json(user.clone())),
        None => Err(StatusCode::NOT_FOUND),
    }
}

async fn create_user(
    State(state): State<AppState>,
    Json(payload): Json<CreateUser>,
) -> Result<Json<User>, StatusCode> {
    let mut users = state.lock().unwrap();
    let new_id = users.len() as u32 + 1;
    
    let new_user = User {
        id: new_id,
        name: payload.name,
        email: payload.email,
    };
    
    users.insert(new_id, new_user.clone());
    Ok(Json(new_user))
}
```

### gRPC Microservice

```rust
use tonic::{transport::Server, Request, Response, Status, Code};
use serde::{Deserialize, Serialize};

// Proto definitions (simplified)
pub mod user_service {
    tonic::include_proto!("user_service");
}

use user_service::{
    user_service_server::{UserService, UserServiceServer},
    UserRequest, UserResponse, CreateUserRequest, CreateUserResponse,
};

#[derive(Debug)]
pub struct UserServiceImpl {
    users: Arc<Mutex<HashMap<u32, UserResponse>>>,
}

#[tonic::async_trait]
impl UserService for UserServiceImpl {
    async fn get_user(
        &self,
        request: Request<UserRequest>,
    ) -> Result<Response<UserResponse>, Status> {
        let user_id = request.into_inner().id;
        let users = self.users.lock().unwrap();
        
        match users.get(&user_id) {
            Some(user) => Ok(Response::new(user.clone())),
            None => Err(Status::new(Code::NotFound, "User not found")),
        }
    }
    
    async fn create_user(
        &self,
        request: Request<CreateUserRequest>,
    ) -> Result<Response<CreateUserResponse>, Status> {
        let create_req = request.into_inner();
        let mut users = self.users.lock().unwrap();
        
        let new_id = users.len() as u32 + 1;
        let new_user = UserResponse {
            id: new_id,
            name: create_req.name,
            email: create_req.email,
        };
        
        users.insert(new_id, new_user.clone());
        
        Ok(Response::new(CreateUserResponse {
            user: Some(new_user),
        }))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "127.0.0.1:50051".parse()?;
    let user_service = UserServiceImpl {
        users: Arc::new(Mutex::new(HashMap::new())),
    };
    
    println!("gRPC service listening on {}", addr);
    
    Server::builder()
        .add_service(
            UserServiceServer::new(user_service)
        )
        .serve(addr)
        .await?;
    
    Ok(())
}
```

---

## Service Discovery

### Consul Service Registration

```rust
use consul::{Agent, AgentOptions, CatalogRegistration, HealthService};
use std::time::Duration;

struct ServiceRegistry {
    agent: Agent,
    service_name: String,
    service_id: String,
}

impl ServiceRegistry {
    async fn new(service_name: String, service_id: String) -> Result<Self, Box<dyn std::error::Error>> {
        let agent = Agent::connect(AgentOptions::default()).await?;
        
        Ok(ServiceRegistry {
            agent,
            service_name,
            service_id,
        })
    }
    
    async fn register_service(&self, address: &str, port: u16) -> Result<(), Box<dyn std::error::Error>> {
        let registration = CatalogRegistration {
            id: Some(self.service_id.clone()),
            name: self.service_name.clone(),
            address: Some(address.to_string()),
            port: Some(port),
            tags: Some(vec!["rust".to_string(), "microservice".to_string()]),
            meta: Some(HashMap::from([
                ("version".to_string(), "1.0.0".to_string()),
                ("environment".to_string(), "production".to_string()),
            ])),
            ..Default::default()
        };
        
        self.agent.register_service(registration).await?;
        
        // Register health check
        let health_service = HealthService {
            id: format!("{}-health", self.service_id),
            name: format!("{}-health", self.service_name),
            check: Some(format!("http://{}:{}/health", address, port)),
            interval: Some("10s".to_string()),
            timeout: Some("3s".to_string()),
            ..Default::default()
        };
        
        self.agent.register_health_service(health_service).await?;
        
        println!("Service {} registered at {}:{}", self.service_name, address, port);
        Ok(())
    }
    
    async fn discover_services(&self) -> Result<Vec<CatalogRegistration>, Box<dyn std::error::Error>> {
        let services = self.agent.catalog_service(&self.service_name).await?;
        println!("Discovered {} instances of {}", services.len(), self.service_name);
        Ok(services)
    }
    
    async fn deregister_service(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.agent.deregister_service(&self.service_id).await?;
        println!("Service {} deregistered", self.service_id);
        Ok(())
    }
}
```

### etcd Distributed Configuration

```rust
use etcd_rs::{Client, ClientConfig, DeleteOptions, GetOptions, PutOptions, TxnOp, TxnCmp};
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Serialize, Deserialize)]
struct ServiceConfig {
    host: String,
    port: u16,
    max_connections: u32,
    timeout_ms: u64,
}

struct DistributedConfig {
    client: Client,
    service_name: String,
}

impl DistributedConfig {
    async fn new(etcd_endpoints: Vec<String>, service_name: String) -> Result<Self, Box<dyn std::error::Error>> {
        let config = ClientConfig {
            endpoints: etcd_endpoints,
            auth: None,
            cache_size: 32,
            cache_enable: true,
        };
        
        let client = Client::connect(config).await?;
        
        Ok(DistributedConfig {
            client,
            service_name,
        })
    }
    
    async fn set_config(&self, config: &ServiceConfig) -> Result<(), Box<dyn std::error::Error>> {
        let key = format!("/services/{}/config", self.service_name);
        let value = serde_json::to_string(config)?;
        
        self.client.put(
            key,
            Some(value),
            Some(PutOptions::new().with_ttl(300)), // 5 minutes TTL
        ).await?;
        
        println!("Configuration updated for service {}", self.service_name);
        Ok(())
    }
    
    async fn get_config(&self) -> Result<Option<ServiceConfig>, Box<dyn std::error::Error>> {
        let key = format!("/services/{}/config", self.service_name);
        
        match self.client.get(key, Some(GetOptions::default())).await? {
            Some(kv) => {
                let config: ServiceConfig = serde_json::from_str(&kv.value)?;
                Ok(Some(config))
            }
            None => Ok(None),
        }
    }
    
    async fn watch_config(&self) -> Result<(), Box<dyn std::error::Error>> {
        let key = format!("/services/{}/config", self.service_name);
        let (mut watcher, mut stream) = self.client.watch(key).await?;
        
        println!("Watching configuration changes for service {}", self.service_name);
        
        while let Some(event) = stream.next().await {
            match event {
                etcd_rs::Event::Put(kv) => {
                    if let Some(value) = kv.value {
                        let config: ServiceConfig = serde_json::from_str(&value)?;
                        println!("Configuration updated: {:?}", config);
                    }
                }
                etcd_rs::Event::Delete => {
                    println!("Configuration deleted");
                }
            }
        }
        
        Ok(())
    }
    
    async fn distributed_lock(&self, lock_name: &str, ttl: u64) -> Result<(), Box<dyn std::error::Error>> {
        let lock_key = format!("/locks/{}/{}", self.service_name, lock_name);
        let lock_value = format!("{}:{}", self.service_name, std::process::id());
        
        // Compare-and-swap transaction
        let txn = vec![
            TxnOp::compare(lock_key.clone(), TxnCmp::Equal(None)),
            TxnOp::put(lock_key.clone(), Some(lock_value), Some(PutOptions::new().with_ttl(ttl))),
        ];
        
        let response = self.client.txn(txn).await?;
        
        if response.succeeded {
            println!("Acquired lock: {}", lock_name);
            Ok(())
        } else {
            Err(format!("Failed to acquire lock: {}", lock_name).into())
        }
    }
}
```

---

## Message Queues and Event Streaming

### Apache Kafka Producer

```rust
use kafka::producer::{Producer, Record, ProducerConfig};
use serde::Serialize;
use std::time::Duration;

struct KafkaProducer {
    producer: Producer,
    topic: String,
}

impl KafkaProducer {
    async fn new(brokers: &str, topic: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let config = ProducerConfig {
            brokers: brokers.to_owned(),
            client_id: Some("rust-producer".to_string()),
            ..Default::default()
        };
        
        let producer = Producer::from_config(config)?;
        
        Ok(KafkaProducer {
            producer,
            topic: topic.to_string(),
        })
    }
    
    async fn send_message<T: Serialize>(&mut self, message: &T) -> Result<(), Box<dyn std::error::Error>> {
        let payload = serde_json::to_string(message)?;
        let record = Record::from_value(&self.topic, payload);
        
        match self.producer.send(&record) {
            Ok(Ok(_)) => {
                println!("Message sent to topic {}", self.topic);
                Ok(())
            }
            Ok(Err(e)) => Err(format!("Failed to send message: {}", e).into()),
            Err(e) => Err(format!("Kafka error: {}", e).into()),
        }
    }
    
    async fn send_batch<T: Serialize>(&mut self, messages: &[T]) -> Result<(), Box<dyn std::error::Error>> {
        for message in messages {
            self.send_message(message).await?;
        }
        Ok(())
    }
}
```

### Redis Pub/Sub

```rust
use redis::{Client, Commands, PubSubCommands};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;

struct RedisPubSub {
    client: Client,
}

impl RedisPubSub {
    async fn new(redis_url: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let client = Client::open(redis_url)?;
        Ok(RedisPubSub { client })
    }
    
    async fn publish<T: Serialize>(&mut self, channel: &str, message: &T) -> Result<(), Box<dyn std::error::Error>> {
        let payload = serde_json::to_string(message)?;
        let _: i32 = self.client.publish(channel, payload).await?;
        
        println!("Published to {}: {:?}", channel, message);
        Ok(())
    }
    
    async fn subscribe<F, T>(&mut self, channel: &str, callback: F) -> Result<(), Box<dyn std::error::Error>>
    where
        F: Fn(T) + Send + Sync + 'static,
        T: for<'de> Deserialize<'de> + Send + 'static,
    {
        let callback = Arc::new(callback);
        let mut pubsub = self.client.get_async_pubsub().await?;
        pubsub.subscribe(channel).await?;
        
        println!("Subscribed to channel: {}", channel);
        
        loop {
            match pubsub.get_message().await {
                Ok(msg) => {
                    if let Ok(message) = serde_json::from_str::<T>(&msg.get_payload()) {
                        callback(message);
                    }
                }
                Err(e) => {
                    eprintln!("Error receiving message: {}", e);
                }
            }
        }
    }
}

// Usage example
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut redis = RedisPubSub::new("redis://localhost:6379").await?;
    
    // Publisher
    let mut publisher = redis.clone();
    tokio::spawn(async move {
        let message = UserEvent {
            user_id: 123,
            action: "login".to_string(),
            timestamp: chrono::Utc::now(),
        };
        
        tokio::time::sleep(Duration::from_secs(2)).await;
        publisher.publish("user_events", &message).await.unwrap();
    });
    
    // Subscriber
    redis.subscribe("user_events", |event: UserEvent| {
        println!("Received user event: {:?}", event);
    }).await?;
    
    Ok(())
}

#[derive(Debug, Serialize, Deserialize)]
struct UserEvent {
    user_id: u32,
    action: String,
    timestamp: chrono::DateTime<chrono::Utc>,
}
```

---

## Distributed Caching

### Redis Cache Layer

```rust
use redis::{Client, Commands, AsyncCommands};
use serde::{Deserialize, Serialize};
use std::time::Duration;

struct CacheLayer {
    client: Client,
    default_ttl: Duration,
}

impl CacheLayer {
    async fn new(redis_url: &str, default_ttl: Duration) -> Result<Self, Box<dyn std::error::Error>> {
        let client = Client::open(redis_url)?;
        
        Ok(CacheLayer {
            client,
            default_ttl,
        })
    }
    
    async fn set<T: Serialize>(&mut self, key: &str, value: &T) -> Result<(), Box<dyn std::error::Error>> {
        let serialized = serde_json::to_string(value)?;
        let _: () = self.client
            .set_ex(key, serialized, self.default_ttl.as_secs())
            .await?;
        
        Ok(())
    }
    
    async fn set_with_ttl<T: Serialize>(&mut self, key: &str, value: &T, ttl: Duration) -> Result<(), Box<dyn std::error::Error>> {
        let serialized = serde_json::to_string(value)?;
        let _: () = self.client
            .set_ex(key, serialized, ttl.as_secs())
            .await?;
        
        Ok(())
    }
    
    async fn get<T: for<'de> Deserialize<'de>>(&mut self, key: &str) -> Result<Option<T>, Box<dyn std::error::Error>> {
        let result: Option<String> = self.client.get(key).await?;
        
        match result {
            Some(serialized) => {
                let value: T = serde_json::from_str(&serialized)?;
                Ok(Some(value))
            }
            None => Ok(None),
        }
    }
    
    async fn delete(&mut self, key: &str) -> Result<bool, Box<dyn std::error::Error>> {
        let deleted: bool = self.client.del(key).await?;
        Ok(deleted)
    }
    
    async fn exists(&mut self, key: &str) -> Result<bool, Box<dyn std::error::Error>> {
        let exists: bool = self.client.exists(key).await?;
        Ok(exists)
    }
    
    async fn increment(&mut self, key: &str) -> Result<i64, Box<dyn std::error::Error>> {
        let value: i64 = self.client.incr(key, 1).await?;
        Ok(value)
    }
    
    async fn expire(&mut self, key: &str, ttl: Duration) -> Result<bool, Box<dyn std::error::Error>> {
        let success: bool = self.client.expire(key, ttl.as_secs()).await?;
        Ok(success)
    }
}

// Cache-aside pattern
struct UserService {
    cache: CacheLayer,
    database: MockDatabase,
}

impl UserService {
    async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        Ok(UserService {
            cache: CacheLayer::new("redis://localhost:6379", Duration::from_secs(300)).await?,
            database: MockDatabase::new(),
        })
    }
    
    async fn get_user(&mut self, user_id: u32) -> Result<Option<User>, Box<dyn std::error::Error>> {
        let cache_key = format!("user:{}", user_id);
        
        // Try cache first
        if let Some(user) = self.cache.get::<User>(&cache_key).await? {
            println!("Cache hit for user {}", user_id);
            return Ok(Some(user));
        }
        
        // Cache miss - fetch from database
        println!("Cache miss for user {}", user_id);
        if let Some(user) = self.database.get_user(user_id).await? {
            // Store in cache
            self.cache.set(&cache_key, &user).await?;
            Ok(Some(user))
        } else {
            Ok(None)
        }
    }
    
    async fn update_user(&mut self, user: User) -> Result<(), Box<dyn std::error::Error>> {
        // Update database
        self.database.update_user(&user).await?;
        
        // Invalidate cache
        let cache_key = format!("user:{}", user.id);
        self.cache.delete(&cache_key).await?;
        
        Ok(())
    }
}
```

---

## Load Balancing

### Round Robin Load Balancer

```rust
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use tokio::sync::RwLock;

#[derive(Debug, Clone)]
struct ServiceInstance {
    id: String,
    host: String,
    port: u16,
    weight: u32,
    current_connections: Arc<Mutex<u32>>,
}

impl ServiceInstance {
    fn new(id: String, host: String, port: u16, weight: u32) -> Self {
        ServiceInstance {
            id,
            host,
            port,
            weight,
            current_connections: Arc::new(Mutex::new(0)),
        }
    }
    
    fn url(&self) -> String {
        format!("http://{}:{}", self.host, self.port)
    }
    
    async fn health_check(&self) -> bool {
        // Simple health check - in production, use proper HTTP client
        true // Simplified
    }
}

trait LoadBalancer: Send + Sync {
    async fn get_next_instance(&self) -> Option<ServiceInstance>;
    async fn add_instance(&self, instance: ServiceInstance);
    async fn remove_instance(&self, instance_id: &str);
}

struct RoundRobinBalancer {
    instances: Arc<RwLock<VecDeque<ServiceInstance>>>,
}

impl RoundRobinBalancer {
    fn new() -> Self {
        RoundRobinBalancer {
            instances: Arc::new(RwLock::new(VecDeque::new())),
        }
    }
}

#[async_trait::async_trait]
impl LoadBalancer for RoundRobinBalancer {
    async fn get_next_instance(&self) -> Option<ServiceInstance> {
        let mut instances = self.instances.write().await;
        
        if instances.is_empty() {
            return None;
        }
        
        // Move first instance to back
        if let Some(instance) = instances.pop_front() {
            instances.push_back(instance);
            Some(instance)
        } else {
            None
        }
    }
    
    async fn add_instance(&self, instance: ServiceInstance) {
        let mut instances = self.instances.write().await;
        instances.push_back(instance);
    }
    
    async fn remove_instance(&self, instance_id: &str) {
        let mut instances = self.instances.write().await;
        instances.retain(|instance| instance.id != instance_id);
    }
}

struct WeightedRoundRobinBalancer {
    instances: Arc<RwLock<Vec<ServiceInstance>>>,
    current_weights: Arc<Mutex<HashMap<String, u32>>>,
}

impl WeightedRoundRobinBalancer {
    fn new() -> Self {
        WeightedRoundRobinBalancer {
            instances: Arc::new(RwLock::new(Vec::new())),
            current_weights: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

#[async_trait::async_trait]
impl LoadBalancer for WeightedRoundRobinBalancer {
    async fn get_next_instance(&self) -> Option<ServiceInstance> {
        let instances = self.instances.read().await;
        let mut current_weights = self.current_weights.lock().await;
        
        if instances.is_empty() {
            return None;
        }
        
        let mut max_weight = 0;
        let mut selected_instance = None;
        
        for instance in &*instances {
            let current_weight = current_weights.get(&instance.id).unwrap_or(&0);
            let weight = instance.weight + current_weight;
            
            if weight > max_weight {
                max_weight = weight;
                selected_instance = Some(instance.clone());
            }
        }
        
        // Update weights
        for instance in &*instances {
            let current_weight = current_weights.get(&instance.id).unwrap_or(&0);
            let new_weight = current_weight + instance.weight;
            current_weights.insert(instance.id.clone(), new_weight - max_weight);
        }
        
        selected_instance
    }
    
    async fn add_instance(&self, instance: ServiceInstance) {
        let mut instances = self.instances.write().await;
        instances.push(instance);
    }
    
    async fn remove_instance(&self, instance_id: &str) {
        let mut instances = self.instances.write().await;
        instances.retain(|instance| instance.id != instance_id);
    }
}

// Load balanced service
struct LoadBalancedService {
    balancer: Arc<dyn LoadBalancer>,
}

impl LoadBalancedService {
    fn new(balancer: Arc<dyn LoadBalancer>) -> Self {
        LoadBalancedService { balancer }
    }
    
    async fn handle_request(&self) -> Result<String, Box<dyn std::error::Error>> {
        if let Some(instance) = self.balancer.get_next_instance().await {
            println!("Routing request to: {}", instance.url());
            
            // Increment connection count
            let mut connections = instance.current_connections.lock().unwrap();
            *connections += 1;
            
            // Simulate request handling
            let response = format!("Response from {}", instance.id);
            
            // Decrement connection count
            let mut connections = instance.current_connections.lock().unwrap();
            *connections = connections.saturating_sub(1);
            
            Ok(response)
        } else {
            Err("No available instances".into())
        }
    }
}
```

---

## Circuit Breaker Pattern

```rust
use std::time::{Duration, Instant};
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Copy)]
enum CircuitState {
    Closed,
    Open,
    HalfOpen,
}

struct CircuitBreaker {
    state: Arc<RwLock<CircuitState>>,
    failure_count: Arc<RwLock<u32>>,
    failure_threshold: u32,
    timeout: Duration,
    last_failure: Arc<RwLock<Option<Instant>>>,
}

impl CircuitBreaker {
    fn new(failure_threshold: u32, timeout: Duration) -> Self {
        CircuitBreaker {
            state: Arc::new(RwLock::new(CircuitState::Closed)),
            failure_count: Arc::new(RwLock::new(0)),
            failure_threshold,
            timeout,
            last_failure: Arc::new(RwLock::new(None)),
        }
    }
    
    async fn call<F, T, E>(&self, f: F) -> Result<T, CircuitBreakerError<E>>
    where
        F: FnOnce() -> Result<T, E>,
    {
        // Check circuit state
        let state = *self.state.read().await;
        
        match state {
            CircuitState::Open => {
                // Check if timeout has passed
                let last_failure = *self.last_failure.read().await;
                if let Some(failure_time) = last_failure {
                    if failure_time.elapsed() > self.timeout {
                        // Try half-open state
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
        
        // Execute the function
        match f() {
            Ok(result) => {
                // Success - reset failure count and close circuit
                *self.failure_count.write().await = 0;
                *self.state.write().await = CircuitState::Closed;
                Ok(result)
            }
            Err(error) => {
                // Failure - increment count and potentially open circuit
                let mut failure_count = self.failure_count.write().await;
                *failure_count += 1;
                
                if *failure_count >= self.failure_threshold {
                    *self.state.write().await = CircuitState::Open;
                    *self.last_failure.write().await = Some(Instant::now());
                }
                
                Err(CircuitBreakerError::ServiceError(error))
            }
        }
    }
    
    async fn get_state(&self) -> CircuitState {
        *self.state.read().await
    }
}

#[derive(Debug)]
enum CircuitBreakerError<E> {
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
```

---

## Distributed Tracing

### OpenTelemetry Integration

```rust
use opentelemetry::{
    global,
    trace::{TraceErrorExt, Tracer},
    KeyValue,
};
use opentelemetry::sdk::{
    trace as sdktrace,
    Resource,
};
use tracing::{info, error, instrument};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

fn init_tracing(service_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let tracer = sdktrace::TracerProvider::builder()
        .with_simple_exporter(sdktrace::SimpleExporter)
        .with_config(sdktrace::Config {
            default_sampler: Box::new(sdktrace::Sampler::AlwaysOn),
            ..Default::default()
        })
        .build()
        .install_batch(opentelemetry::runtime::Tokio)?;
    
    global::set_tracer_provider(tracer.provider());
    
    tracing_subscriber::registry()
        .with(tracing_opentelemetry::layer().with_tracer(tracer))
        .try_init()?;
    
    println!("Tracing initialized for service: {}", service_name);
    Ok(())
}

#[instrument(fields(user_id, request_id))]
async fn process_user_request(user_id: u32, request_id: String) -> Result<String, Box<dyn std::error::Error>> {
    info!("Processing user request");
    
    // Simulate processing
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    if user_id % 10 == 0 {
        error!("User {} not found", user_id);
        return Err(format!("User {} not found", user_id).into());
    }
    
    let result = format!("Processed request {} for user {}", request_id, user_id);
    info!("Request processed successfully");
    
    Ok(result)
}

// Custom span creation
async fn custom_tracing_example() {
    let tracer = global::tracer("custom-tracer");
    
    let span = tracer.start("custom-operation");
    span.set_attribute(KeyValue::new("operation.type", "custom"));
    
    // Simulate work
    tokio::time::sleep(Duration::from_millis(50)).await;
    
    span.add_event("work-completed", vec![KeyValue::new("work.duration", "50ms")]);
    span.end();
}
```

---

## Key Takeaways

- **Microservices** benefit from Rust's performance and safety
- **Service discovery** enables dynamic service registration
- **Message queues** provide reliable asynchronous communication
- **Distributed caching** improves performance and scalability
- **Load balancing** distributes traffic across instances
- **Circuit breakers** prevent cascade failures
- **Distributed tracing** provides observability
- **Cloud SDKs** integrate with major cloud providers

---

## Cloud and Distributed Systems Best Practices

| Practice | Description | Implementation |
|----------|-------------|----------------|
| **Health checks** | Monitor service health | Implement health endpoints |
| **Graceful shutdown** | Handle shutdown signals | Use signal handling |
| **Retry mechanisms** | Handle transient failures | Exponential backoff |
| **Rate limiting** | Prevent abuse | Token bucket algorithm |
| **Idempotency** | Safe retry operations | Idempotent APIs |
| **Observability** | Monitor system behavior | Structured logging and tracing |
| **Configuration management** | Dynamic configuration | Distributed config stores |
| **Security** | Protect services | TLS, authentication, authorization |
