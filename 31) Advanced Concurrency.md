# Advanced Concurrency in Rust

## Overview

Rust's concurrency model goes far beyond basic threads and channels. This guide explores advanced concurrency patterns including async/await deep dive, custom executors, lock-free data structures, actor models, and sophisticated synchronization primitives.

---

## Advanced Async Patterns

### Custom Futures

```rust
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::{Duration, Instant};

// Custom future that delays execution
struct Delay {
    when: Instant,
}

impl Delay {
    fn new(duration: Duration) -> Self {
        Delay {
            when: Instant::now() + duration,
        }
    }
}

impl Future for Delay {
    type Output = ();
    
    fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        if Instant::now() >= self.when {
            Poll::Ready(())
        } else {
            Poll::Pending
        }
    }
}

// Custom future that combines two futures
struct Join<F1, F2> {
    future1: F1,
    future2: F2,
}

impl<F1, F2> Future for Join<F1, F2>
where
    F1: Future,
    F2: Future,
{
    type Output = (F1::Output, F2::Output);
    
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = unsafe { self.get_unchecked_mut() };
        
        let future1 = unsafe { Pin::new_unchecked(&mut this.future1) };
        let future2 = unsafe { Pin::new_unchecked(&mut this.future2) };
        
        let result1 = future1.poll(cx);
        let result2 = future2.poll(cx);
        
        match (result1, result2) {
            (Poll::Ready(val1), Poll::Ready(val2)) => {
                Poll::Ready((val1, val2))
            }
            _ => Poll::Pending,
        }
    }
}
```

### Async Iterators

```rust
use futures::stream::{self, Stream, StreamExt};
use std::pin::Pin;
use std::task::{Context, Poll};

// Custom async iterator
struct AsyncCounter {
    current: u32,
    max: u32,
}

impl AsyncCounter {
    fn new(max: u32) -> Self {
        AsyncCounter { current: 0, max }
    }
}

impl Stream for AsyncCounter {
    type Item = u32;
    
    fn poll_next(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        if self.current < self.max {
            let result = self.current;
            self.current += 1;
            Poll::Ready(Some(result))
        } else {
            Poll::Ready(None)
        }
    }
}

// Usage with async iteration
async fn process_async_counter() {
    let mut counter = AsyncCounter::new(5);
    
    while let Some(value) = counter.next().await {
        println!("Counter value: {}", value);
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}
```

### Async Traits

```rust
use async_trait::async_trait;

#[async_trait]
trait AsyncProcessor {
    async fn process(&self, data: &str) -> Result<String, Box<dyn std::error::Error>>;
}

struct TextProcessor;

#[async_trait]
impl AsyncProcessor for TextProcessor {
    async fn process(&self, data: &str) -> Result<String, Box<dyn std::error::Error>> {
        // Simulate async processing
        tokio::time::sleep(Duration::from_millis(100)).await;
        Ok(data.to_uppercase())
    }
}

struct NumberProcessor;

#[async_trait]
impl AsyncProcessor for NumberProcessor {
    async fn process(&self, data: &str) -> Result<String, Box<dyn std::error::Error>> {
        tokio::time::sleep(Duration::from_millis(50)).await;
        let num: i32 = data.parse()?;
        Ok((num * 2).to_string())
    }
}

// Generic async function using the trait
async fn process_with<P: AsyncProcessor>(processor: &P, data: &str) -> Result<String, Box<dyn std::error::Error>> {
    processor.process(data).await
}
```

---

## Custom Executors

### Basic Executor Implementation

```rust
use std::collections::VecDeque;
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll, Waker};
use std::thread;

struct Task {
    future: Pin<Box<dyn Future<Output = ()>>>,
    waker: Waker,
}

struct Executor {
    tasks: Arc<Mutex<VecDeque<Task>>>,
}

impl Executor {
    fn new() -> Self {
        Executor {
            tasks: Arc::new(Mutex::new(VecDeque::new())),
        }
    }
    
    fn spawn<F>(&self, future: F)
    where
        F: Future<Output = ()> + 'static,
    {
        let task = Task {
            future: Box::pin(future),
            waker: todo!(), // Create waker
        };
        
        self.tasks.lock().unwrap().push_back(task);
    }
    
    fn run(&self) {
        loop {
            let mut tasks = self.tasks.lock().unwrap();
            
            if let Some(mut task) = tasks.pop_front() {
                drop(tasks); // Release lock before polling
                
                let cx = Context::from_waker(&task.waker);
                match task.future.as_mut().poll(&cx) {
                    Poll::Ready(_) => {
                        // Task completed
                    }
                    Poll::Pending => {
                        // Task still pending, push back to queue
                        self.tasks.lock().unwrap().push_back(task);
                    }
                }
            } else {
                thread::sleep(Duration::from_millis(10));
            }
        }
    }
}
```

### Work-Stealing Executor

```rust
use std::collections::VecDeque;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::thread;

struct WorkStealingQueue {
    tasks: Arc<Mutex<VecDeque<Task>>>,
    workers: Arc<AtomicUsize>,
    condvar: Arc<Condvar>,
}

impl WorkStealingQueue {
    fn new(num_workers: usize) -> Self {
        WorkStealingQueue {
            tasks: Arc::new(Mutex::new(VecDeque::new())),
            workers: Arc::new(AtomicUsize::new(num_workers)),
            condvar: Arc::new(Condvar::new()),
        }
    }
    
    fn push_task(&self, task: Task) {
        let mut tasks = self.tasks.lock().unwrap();
        tasks.push_back(task);
        self.condvar.notify_one();
    }
    
    fn steal_task(&self) -> Option<Task> {
        let mut tasks = self.tasks.lock().unwrap();
        tasks.pop_front()
    }
    
    fn worker_loop(&self) {
        loop {
            let task = self.steal_task();
            
            match task {
                Some(mut task) => {
                    let cx = Context::from_waker(&task.waker);
                    match task.future.as_mut().poll(&cx) {
                        Poll::Ready(_) => {
                            // Task completed
                        }
                        Poll::Pending => {
                            // Push back to queue
                            self.push_task(task);
                        }
                    }
                }
                None => {
                    // No tasks available, wait
                    let mut tasks = self.tasks.lock().unwrap();
                    self.condvar.wait(tasks).unwrap();
                }
            }
        }
    }
}
```

---

## Lock-Free Data Structures

### Lock-Free Stack

```rust
use std::sync::atomic::{AtomicPtr, Ordering};

struct Node<T> {
    data: T,
    next: *const Node<T>,
}

pub struct LockFreeStack<T> {
    head: AtomicPtr<Node<T>>,
}

impl<T> LockFreeStack<T> {
    pub fn new() -> Self {
        LockFreeStack {
            head: AtomicPtr::new(std::ptr::null()),
        }
    }
    
    pub fn push(&self, data: T) {
        let node = Box::into_raw(Box::new(Node {
            data,
            next: std::ptr::null(),
        }));
        
        loop {
            let head = self.head.load(Ordering::Acquire);
            unsafe {
                (*node).next = head;
            }
            
            match self.head.compare_exchange_weak(
                head,
                node,
                Ordering::Release,
                Ordering::Relaxed,
            ) {
                Ok(_) => break,
                Err(_) => continue,
            }
        }
    }
    
    pub fn pop(&self) -> Option<T> {
        loop {
            let head = self.head.load(Ordering::Acquire);
            
            if head.is_null() {
                return None;
            }
            
            let next = unsafe { (*head).next };
            
            match self.head.compare_exchange_weak(
                head,
                next,
                Ordering::Release,
                Ordering::Relaxed,
            ) {
                Ok(_) => {
                    let data = unsafe { Box::from_raw(head) };
                    return Some(data.data);
                }
                Err(_) => continue,
            }
        }
    }
}

impl<T> Drop for LockFreeStack<T> {
    fn drop(&mut self) {
        while let Some(_) = self.pop() {
            // Clean up remaining nodes
        }
    }
}
```

### Lock-Free Queue

```rust
use std::sync::atomic::{AtomicPtr, AtomicUsize, Ordering};

struct QueueNode<T> {
    data: Option<T>,
    next: AtomicPtr<QueueNode<T>>,
}

pub struct LockFreeQueue<T> {
    head: AtomicPtr<QueueNode<T>>,
    tail: AtomicPtr<QueueNode<T>>,
    size: AtomicUsize,
}

impl<T> LockFreeQueue<T> {
    pub fn new() -> Self {
        let dummy = Box::into_raw(Box::new(QueueNode {
            data: None,
            next: AtomicPtr::new(std::ptr::null()),
        }));
        
        LockFreeQueue {
            head: AtomicPtr::new(dummy),
            tail: AtomicPtr::new(dummy),
            size: AtomicUsize::new(0),
        }
    }
    
    pub fn enqueue(&self, data: T) {
        let node = Box::into_raw(Box::new(QueueNode {
            data: Some(data),
            next: AtomicPtr::new(std::ptr::null()),
        }));
        
        loop {
            let tail = self.tail.load(Ordering::Acquire);
            let next = unsafe { (*tail).next.load(Ordering::Acquire) };
            
            if next.is_null() {
                if unsafe { (*tail).next.compare_exchange_weak(
                    std::ptr::null(),
                    node,
                    Ordering::Release,
                    Ordering::Relaxed,
                ).is_ok() } {
                    self.tail.compare_exchange(
                        tail,
                        node,
                        Ordering::Release,
                        Ordering::Relaxed,
                    ).ok();
                    self.size.fetch_add(1, Ordering::Relaxed);
                    break;
                }
            } else {
                self.tail.compare_exchange(
                    tail,
                    next,
                    Ordering::Release,
                    Ordering::Relaxed,
                ).ok();
            }
        }
    }
    
    pub fn dequeue(&self) -> Option<T> {
        loop {
            let head = self.head.load(Ordering::Acquire);
            let tail = self.tail.load(Ordering::Acquire);
            let next = unsafe { (*head).next.load(Ordering::Acquire) };
            
            if head == tail {
                if next.is_null() {
                    return None;
                }
                self.tail.compare_exchange(
                    tail,
                    next,
                    Ordering::Release,
                    Ordering::Relaxed,
                ).ok();
            } else {
                let data = unsafe { (*next).data.take() };
                if self.head.compare_exchange(
                    head,
                    next,
                    Ordering::Release,
                    Ordering::Relaxed,
                ).is_ok() {
                    self.size.fetch_sub(1, Ordering::Relaxed);
                    unsafe { 
                        Box::from_raw(head);
                    }
                    return data;
                }
            }
        }
    }
    
    pub fn len(&self) -> usize {
        self.size.load(Ordering::Relaxed)
    }
}
```

---

## Actor Model

### Basic Actor Implementation

```rust
use std::collections::VecDeque;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;

trait Message: Send + 'static {}

#[derive(Debug)]
struct Stop;

impl Message for Stop {}

struct Actor<M: Message> {
    receiver: Receiver<M>,
    handlers: Vec<Box<dyn Fn(M) + Send>>,
}

impl<M: Message> Actor<M> {
    fn new() -> (Sender<M>, Self) {
        let (sender, receiver) = mpsc::channel();
        
        let actor = Actor {
            receiver,
            handlers: Vec::new(),
        };
        
        (sender, actor)
    }
    
    fn add_handler<F>(&mut self, handler: F)
    where
        F: Fn(M) + Send + 'static,
    {
        self.handlers.push(Box::new(handler));
    }
    
    fn run(mut self) {
        thread::spawn(move || {
            while let Ok(message) = self.receiver.recv() {
                for handler in &self.handlers {
                    handler(message);
                }
            }
        });
    }
}

// Example usage
struct Ping;

impl Message for Ping {}

struct Pong;

impl Message for Pong {}

fn create_ping_pong_actors() {
    let (ping_sender, mut ping_actor) = Actor::<Ping>::new();
    let (pong_sender, mut pong_actor) = Actor::<Pong>::new();
    
    // Setup ping actor
    let pong_sender_clone = pong_sender.clone();
    ping_actor.add_handler(move |_ping: Ping| {
        println!("Ping received, sending Pong");
        pong_sender_clone.send(Pong).unwrap();
    });
    
    // Setup pong actor
    let ping_sender_clone = ping_sender.clone();
    pong_actor.add_handler(move |_pong: Pong| {
        println!("Pong received, sending Ping");
        ping_sender_clone.send(Ping).unwrap();
    });
    
    // Start actors
    ping_actor.run();
    pong_actor.run();
    
    // Start the ping-pong
    ping_sender.send(Ping).unwrap();
    
    thread::sleep(Duration::from_secs(1));
}
```

### Advanced Actor with State

```rust
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

struct ActorState {
    data: HashMap<String, String>,
}

struct StatefulActor {
    state: Arc<Mutex<ActorState>>,
    receiver: Receiver<StatefulMessage>,
}

enum StatefulMessage {
    Get(String, Sender<Option<String>>),
    Set(String, String),
    Delete(String),
}

impl Message for StatefulMessage {}

impl StatefulActor {
    fn new() -> (Sender<StatefulMessage>, Self) {
        let (sender, receiver) = mpsc::channel();
        
        let actor = StatefulActor {
            state: Arc::new(Mutex::new(ActorState {
                data: HashMap::new(),
            })),
            receiver,
        };
        
        (sender, actor)
    }
    
    fn run(self) {
        thread::spawn(move || {
            while let Ok(message) = self.receiver.recv() {
                let mut state = self.state.lock().unwrap();
                
                match message {
                    StatefulMessage::Get(key, sender) => {
                        let value = state.data.get(&key).cloned();
                        sender.send(value).unwrap();
                    }
                    StatefulMessage::Set(key, value) => {
                        state.data.insert(key, value);
                    }
                    StatefulMessage::Delete(key) => {
                        state.data.remove(&key);
                    }
                }
            }
        });
    }
}
```

---

## Advanced Synchronization Primitives

### Read-Write Lock with Priority

```rust
use std::collections::VecDeque;
use std::sync::{Arc, Condvar, Mutex};
use std::thread;

struct PriorityRwLock<T> {
    data: Arc<Mutex<T>>,
    readers: Arc<Mutex<VecDeque<thread::Thread>>>,
    writers: Arc<Mutex<VecDeque<thread::Thread>>>,
    active_readers: Arc<Mutex<usize>>,
    active_writer: Arc<Mutex<bool>>,
    read_condvar: Arc<Condvar>,
    write_condvar: Arc<Condvar>,
}

impl<T> PriorityRwLock<T> {
    fn new(data: T) -> Self {
        PriorityRwLock {
            data: Arc::new(Mutex::new(data)),
            readers: Arc::new(Mutex::new(VecDeque::new())),
            writers: Arc::new(Mutex::new(VecDeque::new())),
            active_readers: Arc::new(Mutex::new(0)),
            active_writer: Arc::new(Mutex::new(false)),
            read_condvar: Arc::new(Condvar::new()),
            write_condvar: Arc::new(Condvar::new()),
        }
    }
    
    fn read_lock(&self) -> ReadGuard<T> {
        let mut readers = self.readers.lock().unwrap();
        let mut writers = self.writers.lock().unwrap();
        let mut active_readers = self.active_readers.lock().unwrap();
        let active_writer = self.active_writer.lock().unwrap();
        
        readers.push_back(thread::current());
        
        while !writers.is_empty() || *active_writer {
            self.read_condvar.wait(readers).unwrap();
        }
        
        readers.pop_front();
        *active_readers += 1;
        
        ReadGuard {
            data: self.data.clone(),
            active_readers: self.active_readers.clone(),
            read_condvar: self.read_condvar.clone(),
        }
    }
    
    fn write_lock(&self) -> WriteGuard<T> {
        let mut writers = self.writers.lock().unwrap();
        writers.push_back(thread::current());
        
        let mut active_readers = self.active_readers.lock().unwrap();
        let mut active_writer = self.active_writer.lock().unwrap();
        
        while *active_readers > 0 || *active_writer {
            self.write_condvar.wait(writers).unwrap();
        }
        
        writers.pop_front();
        *active_writer = true;
        
        WriteGuard {
            data: self.data.clone(),
            active_writer: self.active_writer.clone(),
            active_readers: self.active_readers.clone(),
            write_condvar: self.write_condvar.clone(),
            read_condvar: self.read_condvar.clone(),
        }
    }
}

struct ReadGuard<T> {
    data: Arc<Mutex<T>>,
    active_readers: Arc<Mutex<usize>>,
    read_condvar: Arc<Condvar>,
}

impl<T> std::ops::Deref for ReadGuard<T> {
    type Target = T;
    
    fn deref(&self) -> &Self::Target {
        unsafe { &*self.data.lock().unwrap() }
    }
}

impl<T> Drop for ReadGuard<T> {
    fn drop(&mut self) {
        let mut active_readers = self.active_readers.lock().unwrap();
        *active_readers -= 1;
        
        if *active_readers == 0 {
            self.read_condvar.notify_all();
        }
    }
}

struct WriteGuard<T> {
    data: Arc<Mutex<T>>,
    active_writer: Arc<Mutex<bool>>,
    active_readers: Arc<Mutex<usize>>,
    write_condvar: Arc<Condvar>,
    read_condvar: Arc<Condvar>,
}

impl<T> std::ops::Deref for WriteGuard<T> {
    type Target = T;
    
    fn deref(&self) -> &Self::Target {
        unsafe { &*self.data.lock().unwrap() }
    }
}

impl<T> std::ops::DerefMut for WriteGuard<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.data.lock().unwrap() }
    }
}

impl<T> Drop for WriteGuard<T> {
    fn drop(&mut self) {
        let mut active_writer = self.active_writer.lock().unwrap();
        *active_writer = false;
        
        let active_readers = self.active_readers.lock().unwrap();
        if *active_readers == 0 {
            self.write_condvar.notify_one();
        } else {
            self.read_condvar.notify_all();
        }
    }
}
```

### Semaphore with Fairness

```rust
use std::collections::VecDeque;
use std::sync::{Arc, Condvar, Mutex};
use std::thread;

struct FairSemaphore {
    permits: Arc<Mutex<usize>>,
    waiters: Arc<Mutex<VecDeque<thread::Thread>>>,
    condvar: Arc<Condvar>,
}

impl FairSemaphore {
    fn new(permits: usize) -> Self {
        FairSemaphore {
            permits: Arc::new(Mutex::new(permits)),
            waiters: Arc::new(Mutex::new(VecDeque::new())),
            condvar: Arc::new(Condvar::new()),
        }
    }
    
    fn acquire(&self) -> SemaphoreGuard {
        let mut waiters = self.waiters.lock().unwrap();
        let mut permits = self.permits.lock().unwrap();
        
        waiters.push_back(thread::current());
        
        while *permits == 0 {
            self.condvar.wait(permits).unwrap();
        }
        
        waiters.pop_front();
        *permits -= 1;
        
        SemaphoreGuard {
            permits: self.permits.clone(),
            waiters: self.waiters.clone(),
            condvar: self.condvar.clone(),
        }
    }
}

struct SemaphoreGuard {
    permits: Arc<Mutex<usize>>,
    waiters: Arc<Mutex<VecDeque<thread::Thread>>>,
    condvar: Arc<Condvar>,
}

impl Drop for SemaphoreGuard {
    fn drop(&mut self) {
        let mut permits = self.permits.lock().unwrap();
        let waiters = self.waiters.lock().unwrap();
        
        *permits += 1;
        
        if !waiters.is_empty() {
            self.condvar.notify_one();
        }
    }
}
```

---

## Async Channels and Streams

### Bounded Async Channel

```rust
use futures::sink::SinkExt;
use futures::stream::{Stream, StreamExt};
use std::collections::VecDeque;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll, Waker};

struct AsyncChannel<T> {
    buffer: Arc<Mutex<VecDeque<T>>>,
    capacity: usize,
    send_wakers: Arc<Mutex<Vec<Waker>>>,
    recv_wakers: Arc<Mutex<Vec<Waker>>>,
}

impl<T> AsyncChannel<T> {
    fn new(capacity: usize) -> Self {
        AsyncChannel {
            buffer: Arc::new(Mutex::new(VecDeque::new())),
            capacity,
            send_wakers: Arc::new(Mutex::new(Vec::new())),
            recv_wakers: Arc::new(Mutex::new(Vec::new())),
        }
    }
    
    fn send(&self, item: T) -> SendFuture<T> {
        SendFuture {
            channel: self,
            item: Some(item),
        }
    }
    
    fn recv(&self) -> RecvFuture<T> {
        RecvFuture {
            channel: self,
        }
    }
    
    fn try_send(&self, item: T) -> Result<(), T> {
        let mut buffer = self.buffer.lock().unwrap();
        
        if buffer.len() < self.capacity {
            buffer.push_back(item);
            
            // Wake up a receiver
            let mut recv_wakers = self.recv_wakers.lock().unwrap();
            if let Some(waker) = recv_wakers.pop() {
                waker.wake();
            }
            
            Ok(())
        } else {
            Err(item)
        }
    }
    
    fn try_recv(&self) -> Option<T> {
        let mut buffer = self.buffer.lock().unwrap();
        
        if let Some(item) = buffer.pop_front() {
            // Wake up a sender
            let mut send_wakers = self.send_wakers.lock().unwrap();
            if let Some(waker) = send_wakers.pop() {
                waker.wake();
            }
            
            Some(item)
        } else {
            None
        }
    }
}

struct SendFuture<'a, T> {
    channel: &'a AsyncChannel<T>,
    item: Option<T>,
}

impl<'a, T> Future for SendFuture<'a, T> {
    type Output = Result<(), T>;
    
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if let Some(item) = self.item.take() {
            match self.channel.try_send(item) {
                Ok(()) => Poll::Ready(Ok(())),
                Err(item) => {
                    self.item = Some(item);
                    
                    // Register waker
                    let mut send_wakers = self.channel.send_wakers.lock().unwrap();
                    send_wakers.push(cx.waker().clone());
                    
                    Poll::Pending
                }
            }
        } else {
            Poll::Ready(Ok(()))
        }
    }
}

struct RecvFuture<'a, T> {
    channel: &'a AsyncChannel<T>,
}

impl<'a, T> Future for RecvFuture<'a, T> {
    type Output = Option<T>;
    
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match self.channel.try_recv() {
            Some(item) => Poll::Ready(Some(item)),
            None => {
                // Register waker
                let mut recv_wakers = self.channel.recv_wakers.lock().unwrap();
                recv_wakers.push(cx.waker().clone());
                
                Poll::Pending
            }
        }
    }
}
```

---

## Key Takeaways

- **Custom futures** allow fine-grained control over async behavior
- **Lock-free data structures** provide better performance under contention
- **Actor model** simplifies concurrent programming with message passing
- **Custom executors** enable specialized runtime behavior
- **Advanced synchronization** primitives provide fairness and priority
- **Async channels** form the foundation of concurrent communication
- **Stream processing** enables efficient data flow handling

---

## Advanced Concurrency Best Practices

| Practice | Description | Implementation |
|----------|-------------|----------------|
| **Lock-free algorithms** | Avoid blocking for better performance | Use atomic operations and CAS |
| **Actor model** | Message-based concurrency | Use channels and isolated state |
| **Custom executors** | Optimize for specific workloads | Implement task scheduling |
| **Fair synchronization** | Prevent starvation | Use queues and priority systems |
| **Async streams** | Process data incrementally | Use Stream trait and combinators |
| **Backpressure handling** | Manage resource limits | Use bounded channels and semaphores |
| **Cancellation** | Graceful shutdown handling | Use cooperative cancellation tokens |
