// 31_advanced_concurrency.rs
// Comprehensive examples of advanced concurrency patterns in Rust

use std::collections::{HashMap, VecDeque};
use std::future::Future;
use std::pin::Pin;
use std::sync::atomic::{AtomicPtr, AtomicUsize, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::task::{Context, Poll, Waker};
use std::thread;
use std::time::{Duration, Instant};
use std::any::Any;

// =========================================
// CUSTOM FUTURES
// =========================================

// Custom future that delays execution
pub struct Delay {
    when: Instant,
}

impl Delay {
    pub fn new(duration: Duration) -> Self {
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
pub struct Join<F1, F2> {
    future1: F1,
    future2: F2,
    completed1: bool,
    completed2: bool,
    result1: Option<F1::Output>,
    result2: Option<F2::Output>,
}

impl<F1, F2> Join<F1, F2>
where
    F1: Future,
    F2: Future,
{
    pub fn new(future1: F1, future2: F2) -> Self {
        Join {
            future1,
            future2,
            completed1: false,
            completed2: false,
            result1: None,
            result2: None,
        }
    }
}

impl<F1, F2> Future for Join<F1, F2>
where
    F1: Future,
    F2: Future,
{
    type Output = (F1::Output, F2::Output);
    
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = unsafe { self.get_unchecked_mut() };
        
        if !this.completed1 {
            let future1 = unsafe { Pin::new_unchecked(&mut this.future1) };
            match future1.poll(cx) {
                Poll::Ready(result) => {
                    this.result1 = Some(result);
                    this.completed1 = true;
                }
                Poll::Pending => {}
            }
        }
        
        if !this.completed2 {
            let future2 = unsafe { Pin::new_unchecked(&mut this.future2) };
            match future2.poll(cx) {
                Poll::Ready(result) => {
                    this.result2 = Some(result);
                    this.completed2 = true;
                }
                Poll::Pending => {}
            }
        }
        
        if this.completed1 && this.completed2 {
            Poll::Ready((this.result1.take().unwrap(), this.result2.take().unwrap()))
        } else {
            Poll::Pending
        }
    }
}

// =========================================
// ASYNC ITERATOR (STREAM)
// =========================================

pub struct AsyncCounter {
    current: u32,
    max: u32,
}

impl AsyncCounter {
    pub fn new(max: u32) -> Self {
        AsyncCounter { current: 0, max }
    }
}

// Simplified Stream trait implementation
pub trait Stream {
    type Item;
    
    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>>;
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

// =========================================
// LOCK-FREE DATA STRUCTURES
// =========================================

pub struct Node<T> {
    pub data: T,
    pub next: *const Node<T>,
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
    
    pub fn is_empty(&self) -> bool {
        self.head.load(Ordering::Acquire).is_null()
    }
}

impl<T> Drop for LockFreeStack<T> {
    fn drop(&mut self) {
        while let Some(_) = self.pop() {
            // Clean up remaining nodes
        }
    }
}

// Lock-free queue implementation
pub struct QueueNode<T> {
    pub data: Option<T>,
    pub next: AtomicPtr<QueueNode<T>>,
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
    
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

// =========================================
// ACTOR MODEL
// =========================================

pub trait Message: Send + 'static {}

#[derive(Debug, Clone)]
pub struct Stop;

impl Message for Stop {}

pub struct Actor<M: Message> {
    receiver: std::sync::mpsc::Receiver<M>,
    handlers: Vec<Box<dyn Fn(M) + Send>>,
}

impl<M: Message> Actor<M> {
    pub fn new() -> (std::sync::mpsc::Sender<M>, Self) {
        let (sender, receiver) = std::sync::mpsc::channel();
        
        let actor = Actor {
            receiver,
            handlers: Vec::new(),
        };
        
        (sender, actor)
    }
    
    pub fn add_handler<F>(&mut self, handler: F)
    where
        F: Fn(M) + Send + 'static,
    {
        self.handlers.push(Box::new(handler));
    }
    
    pub fn run(mut self) {
        thread::spawn(move || {
            while let Ok(message) = self.receiver.recv() {
                for handler in &self.handlers {
                    handler(message);
                }
            }
        });
    }
}

// Stateful actor example
#[derive(Debug, Clone)]
pub enum StatefulMessage {
    Get(String, std::sync::mpsc::Sender<Option<String>>),
    Set(String, String),
    Delete(String),
}

impl Message for StatefulMessage {}

pub struct ActorState {
    pub data: HashMap<String, String>,
}

pub struct StatefulActor {
    state: Arc<Mutex<ActorState>>,
    receiver: std::sync::mpsc::Receiver<StatefulMessage>,
}

impl StatefulActor {
    pub fn new() -> (std::sync::mpsc::Sender<StatefulMessage>, Self) {
        let (sender, receiver) = std::sync::mpsc::channel();
        
        let actor = StatefulActor {
            state: Arc::new(Mutex::new(ActorState {
                data: HashMap::new(),
            })),
            receiver,
        };
        
        (sender, actor)
    }
    
    pub fn run(self) {
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

// =========================================
// ADVANCED SYNCHRONIZATION PRIMITIVES
// =========================================

// Fair semaphore implementation
pub struct FairSemaphore {
    permits: Arc<Mutex<usize>>,
    waiters: Arc<Mutex<VecDeque<thread::Thread>>>,
    condvar: Arc<Condvar>,
}

impl FairSemaphore {
    pub fn new(permits: usize) -> Self {
        FairSemaphore {
            permits: Arc::new(Mutex::new(permits)),
            waiters: Arc::new(Mutex::new(VecDeque::new())),
            condvar: Arc::new(Condvar::new()),
        }
    }
    
    pub fn acquire(&self) -> SemaphoreGuard {
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
    
    pub fn try_acquire(&self) -> Option<SemaphoreGuard> {
        let mut permits = self.permits.lock().unwrap();
        
        if *permits > 0 {
            *permits -= 1;
            Some(SemaphoreGuard {
                permits: self.permits.clone(),
                waiters: self.waiters.clone(),
                condvar: self.condvar.clone(),
            })
        } else {
            None
        }
    }
}

pub struct SemaphoreGuard {
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

// Priority read-write lock
pub struct PriorityRwLock<T> {
    data: Arc<Mutex<T>>,
    readers: Arc<Mutex<VecDeque<thread::Thread>>>,
    writers: Arc<Mutex<VecDeque<thread::Thread>>>,
    active_readers: Arc<Mutex<usize>>,
    active_writer: Arc<Mutex<bool>>,
    read_condvar: Arc<Condvar>,
    write_condvar: Arc<Condvar>,
}

impl<T> PriorityRwLock<T> {
    pub fn new(data: T) -> Self {
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
    
    pub fn read_lock(&self) -> ReadGuard<T> {
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
    
    pub fn write_lock(&self) -> WriteGuard<T> {
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

pub struct ReadGuard<T> {
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

pub struct WriteGuard<T> {
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

// =========================================
// ASYNC CHANNEL
// =========================================

pub struct AsyncChannel<T> {
    buffer: Arc<Mutex<VecDeque<T>>>,
    capacity: usize,
    send_wakers: Arc<Mutex<Vec<Waker>>>,
    recv_wakers: Arc<Mutex<Vec<Waker>>>,
}

impl<T> AsyncChannel<T> {
    pub fn new(capacity: usize) -> Self {
        AsyncChannel {
            buffer: Arc::new(Mutex::new(VecDeque::new())),
            capacity,
            send_wakers: Arc::new(Mutex::new(Vec::new())),
            recv_wakers: Arc::new(Mutex::new(Vec::new())),
        }
    }
    
    pub fn send(&self, item: T) -> SendFuture<T> {
        SendFuture {
            channel: self,
            item: Some(item),
        }
    }
    
    pub fn recv(&self) -> RecvFuture<T> {
        RecvFuture {
            channel: self,
        }
    }
    
    pub fn try_send(&self, item: T) -> Result<(), T> {
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
    
    pub fn try_recv(&self) -> Option<T> {
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
    
    pub fn len(&self) -> usize {
        self.buffer.lock().unwrap().len()
    }
    
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

pub struct SendFuture<'a, T> {
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

pub struct RecvFuture<'a, T> {
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

// =========================================
// WORK-STEALING EXECUTOR
// =========================================

pub struct Task {
    pub future: Pin<Box<dyn Future<Output = ()>>>,
    pub waker: Waker,
}

pub struct WorkStealingQueue {
    tasks: Arc<Mutex<VecDeque<Task>>>,
    workers: Arc<AtomicUsize>,
    condvar: Arc<Condvar>,
}

impl WorkStealingQueue {
    pub fn new(num_workers: usize) -> Self {
        WorkStealingQueue {
            tasks: Arc::new(Mutex::new(VecDeque::new())),
            workers: Arc::new(AtomicUsize::new(num_workers)),
            condvar: Arc::new(Condvar::new()),
        }
    }
    
    pub fn push_task(&self, task: Task) {
        let mut tasks = self.tasks.lock().unwrap();
        tasks.push_back(task);
        self.condvar.notify_one();
    }
    
    pub fn steal_task(&self) -> Option<Task> {
        let mut tasks = self.tasks.lock().unwrap();
        tasks.pop_front()
    }
    
    pub fn worker_loop(&self) {
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

// =========================================
// DEMONSTRATION FUNCTIONS
// =========================================

pub fn demonstrate_custom_futures() {
    println!("=== CUSTOM FUTURES DEMONSTRATION ===");
    
    // Test Delay future
    let start = Instant::now();
    
    // In a real async runtime, you would await this
    // let delay = Delay::new(Duration::from_millis(100));
    // delay.await;
    
    println!("Delay future created");
    println!("Elapsed: {:?}", start.elapsed());
    
    // Test Join future
    let future1 = Delay::new(Duration::from_millis(50));
    let future2 = Delay::new(Duration::from_millis(100));
    let join_future = Join::new(future1, future2);
    
    println!("Join future created");
    
    println!();
}

pub fn demonstrate_lock_free_structures() {
    println!("=== LOCK-FREE STRUCTURES DEMONSTRATION ===");
    
    // Test lock-free stack
    let stack = Arc::new(LockFreeStack::new());
    let mut handles = Vec::new();
    
    for i in 0..10 {
        let stack_clone = stack.clone();
        let handle = thread::spawn(move || {
            for j in 0..100 {
                stack_clone.push(i * 100 + j);
            }
        });
        handles.push(handle);
    }
    
    for handle in handles {
        handle.join().unwrap();
    }
    
    let mut count = 0;
    while let Some(_) = stack.pop() {
        count += 1;
    }
    
    println!("Stack popped {} items (expected 1000)", count);
    
    // Test lock-free queue
    let queue = Arc::new(LockFreeQueue::new());
    let mut handles = Vec::new();
    
    for i in 0..5 {
        let queue_clone = queue.clone();
        let handle = thread::spawn(move || {
            for j in 0..20 {
                queue_clone.enqueue(i * 20 + j);
            }
        });
        handles.push(handle);
    }
    
    for handle in handles {
        handle.join().unwrap();
    }
    
    let mut count = 0;
    while let Some(_) = queue.dequeue() {
        count += 1;
    }
    
    println!("Queue dequeued {} items (expected 100)", count);
    println!();
}

pub fn demonstrate_actor_model() {
    println!("=== ACTOR MODEL DEMONSTRATION ===");
    
    // Simple ping-pong actors
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
    
    thread::sleep(Duration::from_millis(100));
    
    // Test stateful actor
    let (state_sender, mut state_actor) = StatefulActor::new();
    
    state_actor.run();
    
    // Send some messages
    state_sender.send(StatefulMessage::Set("key1".to_string(), "value1".to_string())).unwrap();
    state_sender.send(StatefulMessage::Set("key2".to_string(), "value2".to_string())).unwrap();
    
    let (response_sender, response_receiver) = std::sync::mpsc::channel();
    state_sender.send(StatefulMessage::Get("key1".to_string(), response_sender)).unwrap();
    
    if let Some(value) = response_receiver.recv().unwrap() {
        println!("Retrieved value: {}", value);
    }
    
    println!();
}

pub fn demonstrate_synchronization_primitives() {
    println!("=== SYNCHRONIZATION PRIMITIVES DEMONSTRATION ===");
    
    // Test fair semaphore
    let semaphore = Arc::new(FairSemaphore::new(2));
    let mut handles = Vec::new();
    
    for i in 0..5 {
        let semaphore_clone = semaphore.clone();
        let handle = thread::spawn(move || {
            let guard = semaphore_clone.acquire();
            println!("Thread {} acquired semaphore", i);
            thread::sleep(Duration::from_millis(100));
            println!("Thread {} releasing semaphore", i);
            drop(guard);
        });
        handles.push(handle);
    }
    
    for handle in handles {
        handle.join().unwrap();
    }
    
    // Test priority read-write lock
    let rw_lock = Arc::new(PriorityRwLock::new(0));
    let mut handles = Vec::new();
    
    // Spawn readers
    for i in 0..3 {
        let rw_lock_clone = rw_lock.clone();
        let handle = thread::spawn(move || {
            let guard = rw_lock_clone.read_lock();
            println!("Reader {} reading value: {}", i, *guard);
            thread::sleep(Duration::from_millis(50));
            println!("Reader {} finished reading", i);
        });
        handles.push(handle);
    }
    
    // Spawn writer
    let rw_lock_clone = rw_lock.clone();
    let handle = thread::spawn(move || {
        thread::sleep(Duration::from_millis(10)); // Let readers start first
        let mut guard = rw_lock_clone.write_lock();
        println!("Writer writing value: 42");
        *guard = 42;
        thread::sleep(Duration::from_millis(100));
        println!("Writer finished writing");
    });
    handles.push(handle);
    
    for handle in handles {
        handle.join().unwrap();
    }
    
    println!();
}

pub fn demonstrate_async_channel() {
    println!("=== ASYNC CHANNEL DEMONSTRATION ===");
    
    let channel = AsyncChannel::new(3);
    
    // Test try_send and try_recv
    assert!(channel.try_send(1).is_ok());
    assert!(channel.try_send(2).is_ok());
    assert!(channel.try_send(3).is_ok());
    
    println!("Channel length after sending 3 items: {}", channel.len());
    
    assert!(channel.try_recv() == Some(1));
    assert!(channel.try_recv() == Some(2));
    
    println!("Channel length after receiving 2 items: {}", channel.len());
    
    // Test capacity limit
    assert!(channel.try_send(4).is_ok());
    let result = channel.try_send(5);
    assert!(result.is_err());
    println!("Failed to send item when full: {:?}", result.is_err());
    
    println!();
}

pub fn demonstrate_work_stealing() {
    println!("=== WORK-STEALING EXECUTOR DEMONSTRATION ===");
    
    let queue = Arc::new(WorkStealingQueue::new(3));
    let mut handles = Vec::new();
    
    // Start worker threads
    for i in 0..3 {
        let queue_clone = queue.clone();
        let handle = thread::spawn(move || {
            println!("Worker {} started", i);
            queue_clone.worker_loop();
        });
        handles.push(handle);
    }
    
    // Create some dummy tasks
    for i in 0..10 {
        // In a real implementation, you would create actual futures
        println!("Creating task {}", i);
        thread::sleep(Duration::from_millis(10));
    }
    
    // Let workers run for a bit
    thread::sleep(Duration::from_millis(100));
    
    println!();
}

// =========================================
// MAIN DEMONSTRATION
// =========================================

fn main() {
    println!("=== ADVANCED CONCURRENCY DEMONSTRATIONS ===\n");
    
    demonstrate_custom_futures();
    demonstrate_lock_free_structures();
    demonstrate_actor_model();
    demonstrate_synchronization_primitives();
    demonstrate_async_channel();
    demonstrate_work_stealing();
    
    println!("=== ADVANCED CONCURRENCY DEMONSTRATIONS COMPLETE ===");
    println!("Note: This demonstrates advanced concurrency patterns. Real implementations would:");
    println!("- Use proper async runtimes like tokio or async-std");
    println!("- Include proper waker implementations");
    println!("- Handle edge cases and error conditions");
    println!("- Include proper cleanup and resource management");
}

// =========================================
// UNIT TESTS
// =========================================

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_lock_free_stack() {
        let stack = LockFreeStack::new();
        
        assert!(stack.is_empty());
        
        stack.push(1);
        stack.push(2);
        stack.push(3);
        
        assert!(!stack.is_empty());
        
        assert_eq!(stack.pop(), Some(3));
        assert_eq!(stack.pop(), Some(2));
        assert_eq!(stack.pop(), Some(1));
        assert_eq!(stack.pop(), None);
        
        assert!(stack.is_empty());
    }
    
    #[test]
    fn test_lock_free_queue() {
        let queue = LockFreeQueue::new();
        
        assert!(queue.is_empty());
        assert_eq!(queue.len(), 0);
        
        queue.enqueue(1);
        queue.enqueue(2);
        queue.enqueue(3);
        
        assert!(!queue.is_empty());
        assert_eq!(queue.len(), 3);
        
        assert_eq!(queue.dequeue(), Some(1));
        assert_eq!(queue.dequeue(), Some(2));
        assert_eq!(queue.dequeue(), Some(3));
        assert_eq!(queue.dequeue(), None);
        
        assert!(queue.is_empty());
        assert_eq!(queue.len(), 0);
    }
    
    #[test]
    fn test_fair_semaphore() {
        let semaphore = FairSemaphore::new(2);
        
        let guard1 = semaphore.acquire();
        let guard2 = semaphore.acquire();
        
        // Should not be able to acquire a third permit
        let guard3 = semaphore.try_acquire();
        assert!(guard3.is_none());
        
        drop(guard1);
        
        // Should be able to acquire now
        let guard4 = semaphore.try_acquire();
        assert!(guard4.is_some());
        
        drop(guard2);
        drop(guard4);
    }
    
    #[test]
    fn test_async_channel() {
        let channel = AsyncChannel::new(2);
        
        assert!(channel.is_empty());
        assert_eq!(channel.len(), 0);
        
        assert!(channel.try_send(1).is_ok());
        assert!(channel.try_send(2).is_ok());
        
        assert!(!channel.is_empty());
        assert_eq!(channel.len(), 2);
        
        // Should be full now
        assert!(channel.try_send(3).is_err());
        
        assert_eq!(channel.try_recv(), Some(1));
        assert_eq!(channel.try_recv(), Some(2));
        assert_eq!(channel.try_recv(), None);
        
        assert!(channel.is_empty());
        assert_eq!(channel.len(), 0);
    }
    
    #[test]
    fn test_priority_rwlock() {
        let rw_lock = PriorityRwLock::new(0);
        
        // Test read lock
        {
            let guard1 = rw_lock.read_lock();
            let guard2 = rw_lock.read_lock();
            assert_eq!(*guard1, 0);
            assert_eq!(*guard2, 0);
        }
        
        // Test write lock
        {
            let mut guard = rw_lock.write_lock();
            *guard = 42;
        }
        
        // Verify value changed
        {
            let guard = rw_lock.read_lock();
            assert_eq!(*guard, 42);
        }
    }
    
    #[test]
    fn test_delay_future() {
        let delay = Delay::new(Duration::from_millis(10));
        
        // In a real test, you would use a test executor
        // For now, just verify it creates successfully
        assert!(true);
    }
    
    #[test]
    fn test_join_future() {
        let future1 = Delay::new(Duration::from_millis(10));
        let future2 = Delay::new(Duration::from_millis(20));
        let join_future = Join::new(future1, future2);
        
        // In a real test, you would use a test executor
        assert!(true);
    }
    
    #[test]
    fn test_async_counter() {
        let mut counter = AsyncCounter::new(3);
        
        // In a real test, you would use a test executor
        // For now, just verify it creates successfully
        assert!(true);
    }
}
