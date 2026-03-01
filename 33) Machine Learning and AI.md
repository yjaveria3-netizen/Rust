# Machine Learning and AI in Rust

## Overview

Rust's performance, safety, and growing ecosystem make it increasingly popular for machine learning and AI applications. This guide covers ML frameworks, neural networks, data processing, and AI algorithms in Rust.

---

## ML/AI Ecosystem

### Core ML/AI Crates

| Crate | Purpose | Features |
|-------|---------|----------|
| `tch` | PyTorch bindings | Deep learning, tensors, GPU support |
| `candle-core` | ML framework | Pure Rust ML, no Python dependency |
| `linfa` | Traditional ML | Classical ML algorithms |
| `smartcore` | ML algorithms | Comprehensive ML library |
| `ndarray` | N-dimensional arrays | Numerical computing |
| `polars` | Data processing | Fast dataframes |
| `serde_json` | JSON handling | Data serialization |
| `rayon` | Parallel processing | Data parallelism |

### Choosing the Right ML Library

- **tch** - Best for deep learning with PyTorch
- **candle** - Pure Rust ML framework
- **linfa** - Traditional ML algorithms
- **smartcore** - Comprehensive ML toolkit
- **ndarray** - Numerical computing foundation

---

## Data Processing and Preparation

### DataFrames with Polars

```rust
use polars::prelude::*;

fn load_and_process_data() -> Result<(), Box<dyn std::error::Error>> {
    // Load CSV data
    let df = CsvReader::from_path("data.csv")?
        .finish()?;
    
    println!("Original data shape: {:?}", df.shape());
    
    // Filter data
    let filtered = df
        .lazy()
        .filter(col("age").gt(lit(18)))
        .filter(col("salary").gt(lit(50000)))
        .collect()?;
    
    // Group and aggregate
    let aggregated = filtered
        .lazy()
        .groupby(["department"])
        .agg([
            col("salary").mean().alias("avg_salary"),
            col("age").mean().alias("avg_age"),
            col("salary").count().alias("employee_count"),
        ])
        .collect()?;
    
    println!("Aggregated data:");
    println!("{}", aggregated);
    
    Ok(())
}
```

### Feature Engineering

```rust
use ndarray::Array2;
use std::collections::HashMap;

struct FeatureEngineer {
    categorical_mappings: HashMap<String, HashMap<String, f64>>,
}

impl FeatureEngineer {
    fn new() -> Self {
        FeatureEngineer {
            categorical_mappings: HashMap::new(),
        }
    }
    
    fn fit_categorical(&mut self, data: &[String], feature_name: &str) {
        let mut mapping = HashMap::new();
        let unique_values: std::collections::HashSet<_> = data.iter().cloned().collect();
        
        for (i, value) in unique_values.iter().enumerate() {
            mapping.insert(value.clone(), i as f64);
        }
        
        self.categorical_mappings.insert(feature_name.to_string(), mapping);
    }
    
    fn transform_categorical(&self, value: &str, feature_name: &str) -> f64 {
        if let Some(mapping) = self.categorical_mappings.get(feature_name) {
            mapping.get(value).copied().unwrap_or(-1.0)
        } else {
            -1.0
        }
    }
    
    fn normalize_features(&self, features: &mut Array2<f64>) {
        for mut column in features.axis_iter_mut(ndarray::Axis(1)) {
            let mean = column.iter().sum::<f64>() / column.len() as f64;
            let variance = column.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / column.len() as f64;
            let std_dev = variance.sqrt();
            
            for value in column.iter_mut() {
                *value = (value - mean) / std_dev;
            }
        }
    }
}
```

---

## Traditional Machine Learning

### Linear Regression

```rust
use ndarray::Array2;
use std::f64;

struct LinearRegression {
    weights: Vec<f64>,
    bias: f64,
    learning_rate: f64,
}

impl LinearRegression {
    fn new(n_features: usize, learning_rate: f64) -> Self {
        LinearRegression {
            weights: vec![0.0; n_features],
            bias: 0.0,
            learning_rate,
        }
    }
    
    fn predict(&self, features: &[f64]) -> f64 {
        let mut prediction = self.bias;
        for (i, &weight) in self.weights.iter().enumerate() {
            prediction += weight * features[i];
        }
        prediction
    }
    
    fn fit(&mut self, X: &Array2<f64>, y: &[f64], epochs: usize) {
        let n_samples = X.nrows();
        
        for _epoch in 0..epochs {
            let mut gradients = vec![0.0; self.weights.len()];
            let mut bias_gradient = 0.0;
            
            // Calculate gradients
            for i in 0..n_samples {
                let features = X.row(i);
                let prediction = self.predict(features.as_slice().unwrap());
                let error = prediction - y[i];
                
                for (j, &feature) in features.iter().enumerate() {
                    gradients[j] += error * feature;
                }
                bias_gradient += error;
            }
            
            // Update weights and bias
            for (j, gradient) in gradients.iter().enumerate() {
                self.weights[j] -= self.learning_rate * gradient / n_samples as f64;
            }
            self.bias -= self.learning_rate * bias_gradient / n_samples as f64;
        }
    }
    
    fn mse(&self, X: &Array2<f64>, y: &[f64]) -> f64 {
        let mut total_error = 0.0;
        let n_samples = X.nrows();
        
        for i in 0..n_samples {
            let features = X.row(i);
            let prediction = self.predict(features.as_slice().unwrap());
            let error = prediction - y[i];
            total_error += error * error;
        }
        
        total_error / n_samples as f64
    }
}
```

### Decision Tree

```rust
use std::collections::HashMap;

#[derive(Debug, Clone)]
enum DecisionNode {
    Leaf { value: f64 },
    Split {
        feature_index: usize,
        threshold: f64,
        left: Box<DecisionNode>,
        right: Box<DecisionNode>,
    },
}

struct DecisionTree {
    root: DecisionNode,
    max_depth: usize,
}

impl DecisionTree {
    fn new(max_depth: usize) -> Self {
        DecisionTree {
            root: DecisionNode::Leaf { value: 0.0 },
            max_depth,
        }
    }
    
    fn fit(&mut self, X: &Array2<f64>, y: &[f64]) {
        let indices: Vec<usize> = (0..X.nrows()).collect();
        self.root = self.build_tree(X, y, &indices, 0);
    }
    
    fn build_tree(
        &self,
        X: &Array2<f64>,
        y: &[f64],
        indices: &[usize],
        depth: usize,
    ) -> DecisionNode {
        if depth >= self.max_depth || indices.len() <= 1 {
            let avg_value = indices.iter().map(|&i| y[i]).sum::<f64>() / indices.len() as f64;
            return DecisionNode::Leaf { value: avg_value };
        }
        
        let (best_feature, best_threshold, best_gain) = self.find_best_split(X, y, indices);
        
        if best_gain <= 0.0 {
            let avg_value = indices.iter().map(|&i| y[i]).sum::<f64>() / indices.len() as f64;
            return DecisionNode::Leaf { value: avg_value };
        }
        
        let (left_indices, right_indices) = self.split_indices(X, indices, best_feature, best_threshold);
        
        let left = self.build_tree(X, y, &left_indices, depth + 1);
        let right = self.build_tree(X, y, &right_indices, depth + 1);
        
        DecisionNode::Split {
            feature_index: best_feature,
            threshold: best_threshold,
            left: Box::new(left),
            right: Box::new(right),
        }
    }
    
    fn find_best_split(
        &self,
        X: &Array2<f64>,
        y: &[f64],
        indices: &[usize],
    ) -> (usize, f64, f64) {
        let mut best_feature = 0;
        let mut best_threshold = 0.0;
        let mut best_gain = 0.0;
        
        for feature in 0..X.ncols() {
            let values: Vec<f64> = indices.iter()
                .map(|&i| X[[i, feature]])
                .collect();
            
            let min_val = values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
            let max_val = values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
            
            for threshold in (min_val as i32..max_val as i32).step_by(1) {
                let gain = self.calculate_information_gain(X, y, indices, feature, threshold as f64);
                
                if gain > best_gain {
                    best_gain = gain;
                    best_feature = feature;
                    best_threshold = threshold as f64;
                }
            }
        }
        
        (best_feature, best_threshold, best_gain)
    }
    
    fn calculate_information_gain(
        &self,
        X: &Array2<f64>,
        y: &[f64],
        indices: &[usize],
        feature: usize,
        threshold: f64,
    ) -> f64 {
        let (left_indices, right_indices) = self.split_indices(X, indices, feature, threshold);
        
        if left_indices.is_empty() || right_indices.is_empty() {
            return 0.0;
        }
        
        let parent_entropy = self.calculate_entropy(y, indices);
        let left_entropy = self.calculate_entropy(y, &left_indices);
        let right_entropy = self.calculate_entropy(y, &right_indices);
        
        let left_weight = left_indices.len() as f64 / indices.len() as f64;
        let right_weight = right_indices.len() as f64 / indices.len() as f64;
        
        parent_entropy - (left_weight * left_entropy + right_weight * right_entropy)
    }
    
    fn calculate_entropy(&self, y: &[f64], indices: &[usize]) -> f64 {
        let mut counts = HashMap::new();
        
        for &i in indices {
            *counts.entry(y[i]).or_insert(0) += 1;
        }
        
        let mut entropy = 0.0;
        let total = indices.len() as f64;
        
        for &count in counts.values() {
            let probability = count / total;
            entropy -= probability * probability.ln();
        }
        
        entropy
    }
    
    fn split_indices(
        &self,
        X: &Array2<f64>,
        indices: &[usize],
        feature: usize,
        threshold: f64,
    ) -> (Vec<usize>, Vec<usize>) {
        let mut left = Vec::new();
        let mut right = Vec::new();
        
        for &i in indices {
            if X[[i, feature]] <= threshold {
                left.push(i);
            } else {
                right.push(i);
            }
        }
        
        (left, right)
    }
    
    fn predict(&self, features: &[f64]) -> f64 {
        self.predict_node(&self.root, features)
    }
    
    fn predict_node(&self, node: &DecisionNode, features: &[f64]) -> f64 {
        match node {
            DecisionNode::Leaf { value } => *value,
            DecisionNode::Split { feature_index, threshold, left, right } => {
                if features[*feature_index] <= *threshold {
                    self.predict_node(left, features)
                } else {
                    self.predict_node(right, features)
                }
            }
        }
    }
}
```

---

## Deep Learning with Candle

### Neural Network Basics

```rust
use candle_core::{Tensor, Device, DType};
use candle_nn::{Module, Var, Linear, linear, LayerNorm, layer_norm};

struct SimpleMLP {
    linear1: Linear,
    linear2: Linear,
    dropout: candle_nn::Dropout,
}

impl SimpleMLP {
    fn new(input_dim: usize, hidden_dim: usize, output_dim: usize, device: &Device) -> Result<Self, candle_core::Error> {
        let linear1 = linear(input_dim, hidden_dim, Default::default())?;
        let linear2 = linear(hidden_dim, output_dim, Default::default())?;
        let dropout = candle_nn::Dropout::new(0.1);
        
        Ok(SimpleMLP {
            linear1,
            linear2,
            dropout,
        })
    }
}

impl Module for SimpleMLP {
    fn forward(&self, xs: &Tensor) -> Result<Tensor, candle_core::Error> {
        let xs = xs.apply(&self.linear1)?;
        let xs = candle_nn::ops::relu(&xs)?;
        let xs = self.dropout.forward(&xs, true)?;
        let xs = xs.apply(&self.linear2)?;
        Ok(xs)
    }
}

struct NeuralNetworkTrainer {
    model: SimpleMLP,
    optimizer: candle_nn::Adam,
    device: Device,
}

impl NeuralNetworkTrainer {
    fn new(input_dim: usize, hidden_dim: usize, output_dim: usize) -> Result<Self, candle_core::Error> {
        let device = Device::cuda_if_available(0).unwrap_or(Device::Cpu);
        let model = SimpleMLP::new(input_dim, hidden_dim, output_dim, &device)?;
        let optimizer = candle_nn::Adam::new(&model.vars(), 0.001)?;
        
        Ok(NeuralNetworkTrainer {
            model,
            optimizer,
            device,
        })
    }
    
    fn train_step(&mut self, inputs: &Tensor, targets: &Tensor) -> Result<f64, candle_core::Error> {
        let logits = self.model.forward(inputs)?;
        let loss = candle_nn::loss::mse(&logits, targets)?;
        
        let grads = loss.backward()?;
        self.optimizer.step(&grads)?;
        
        Ok(loss.to_vec0::<f32>()? as f64)
    }
    
    fn predict(&self, inputs: &Tensor) -> Result<Tensor, candle_core::Error> {
        self.model.forward(inputs)
    }
}
```

### Convolutional Neural Network

```rust
use candle_nn::{Conv2d, MaxPool2d, conv2d, max_pool2d};

struct CNN {
    conv1: Conv2d,
    conv2: Conv2d,
    fc: Linear,
    pool: MaxPool2d,
}

impl CNN {
    fn new(device: &Device) -> Result<Self, candle_core::Error> {
        let conv1 = conv2d(1, 32, 3, Default::default())?;
        let conv2 = conv2d(32, 64, 3, Default::default())?;
        let fc = linear(64 * 5 * 5, 10, Default::default())?;
        let pool = max_pool2d(2, 2);
        
        Ok(CNN {
            conv1,
            conv2,
            fc,
            pool,
        })
    }
}

impl Module for CNN {
    fn forward(&self, xs: &Tensor) -> Result<Tensor, candle_core::Error> {
        let xs = xs.apply(&self.conv1)?;
        let xs = candle_nn::ops::relu(&xs)?;
        let xs = xs.apply(&self.pool)?;
        
        let xs = xs.apply(&self.conv2)?;
        let xs = candle_nn::ops::relu(&xs)?;
        let xs = xs.apply(&self.pool)?;
        
        let xs = xs.flatten(1, xs.dims2()?.len() - 1)?;
        let xs = xs.apply(&self.fc)?;
        
        Ok(xs)
    }
}
```

---

## Natural Language Processing

### Text Processing

```rust
use std::collections::HashMap;

struct Tokenizer {
    vocab: HashMap<String, u32>,
    reverse_vocab: HashMap<u32, String>,
}

impl Tokenizer {
    fn new() -> Self {
        Tokenizer {
            vocab: HashMap::new(),
            reverse_vocab: HashMap::new(),
        }
    }
    
    fn build_vocab(&mut self, texts: &[String]) {
        let mut word_counts = HashMap::new();
        
        for text in texts {
            for word in text.split_whitespace() {
                *word_counts.entry(word.to_string()).or_insert(0) += 1;
            }
        }
        
        // Add special tokens
        self.vocab.insert("<pad>".to_string(), 0);
        self.vocab.insert("<unk>".to_string(), 1);
        self.vocab.insert("<sos>".to_string(), 2);
        self.vocab.insert("<eos>".to_string(), 3);
        
        let mut next_id = 4;
        for (word, _) in word_counts {
            if !self.vocab.contains_key(&word) {
                self.vocab.insert(word, next_id);
                next_id += 1;
            }
        }
        
        // Build reverse vocabulary
        for (word, &id) in &self.vocab {
            self.reverse_vocab.insert(id, word.clone());
        }
    }
    
    fn encode(&self, text: &str) -> Vec<u32> {
        text.split_whitespace()
            .map(|word| self.vocab.get(word).copied().unwrap_or(1)) // <unk> token
            .collect()
    }
    
    fn decode(&self, tokens: &[u32]) -> String {
        tokens.iter()
            .filter_map(|&token| self.reverse_vocab.get(&token))
            .collect::<Vec<_>>()
            .join(" ")
    }
    
    fn vocab_size(&self) -> usize {
        self.vocab.len()
    }
}

// Simple word embeddings
struct WordEmbeddings {
    embeddings: Vec<Vec<f64>>,
    vocab: HashMap<String, usize>,
    embedding_dim: usize,
}

impl WordEmbeddings {
    fn new(vocab_size: usize, embedding_dim: usize) -> Self {
        let mut embeddings = Vec::with_capacity(vocab_size);
        
        for _ in 0..vocab_size {
            let mut embedding = Vec::with_capacity(embedding_dim);
            for _ in 0..embedding_dim {
                embedding.push(rand::random::<f64>() * 0.1 - 0.05); // Small random initialization
            }
            embeddings.push(embedding);
        }
        
        WordEmbeddings {
            embeddings,
            vocab: HashMap::new(),
            embedding_dim,
        }
    }
    
    fn get_embedding(&self, word: &str) -> Option<&Vec<f64>> {
        if let Some(&index) = self.vocab.get(word) {
            self.embeddings.get(index)
        } else {
            None
        }
    }
    
    fn cosine_similarity(&self, word1: &str, word2: &str) -> Option<f64> {
        let emb1 = self.get_embedding(word1)?;
        let emb2 = self.get_embedding(word2)?;
        
        let dot_product: f64 = emb1.iter().zip(emb2.iter()).map(|(a, b)| a * b).sum();
        let norm1: f64 = emb1.iter().map(|x| x * x).sum::<f64>().sqrt();
        let norm2: f64 = emb2.iter().map(|x| x * x).sum::<f64>().sqrt();
        
        Some(dot_product / (norm1 * norm2))
    }
}
```

---

## Computer Vision

### Image Processing

```rust
use ndarray::Array3;

struct ImageProcessor;

impl ImageProcessor {
    fn grayscale(image: &Array3<u8>) -> Array3<u8> {
        let (height, width, _) = image.dim();
        let mut gray = Array3::zeros((height, width, 1));
        
        for y in 0..height {
            for x in 0..width {
                let r = image[[y, x, 0]] as f64;
                let g = image[[y, x, 1]] as f64;
                let b = image[[y, x, 2]] as f64;
                
                let gray_value = (0.299 * r + 0.587 * g + 0.114 * b) as u8;
                gray[[y, x, 0]] = gray_value;
            }
        }
        
        gray
    }
    
    fn resize(image: &Array3<u8>, new_width: usize, new_height: usize) -> Array3<u8> {
        let (old_height, old_width, channels) = image.dim();
        let mut resized = Array3::zeros((new_height, new_width, channels));
        
        let x_scale = old_width as f64 / new_width as f64;
        let y_scale = old_height as f64 / new_height as f64;
        
        for y in 0..new_height {
            for x in 0..new_width {
                let src_x = (x as f64 * x_scale) as usize;
                let src_y = (y as f64 * y_scale) as usize;
                
                for c in 0..channels {
                    resized[[y, x, c]] = image[[src_y, src_x, c]];
                }
            }
        }
        
        resized
    }
    
    fn apply_convolution(image: &Array3<u8>, kernel: &Array2<f64>) -> Array3<u8> {
        let (height, width, channels) = image.dim();
        let (k_height, k_width) = kernel.dim();
        let mut result = Array3::zeros((height, width, channels));
        
        let k_center_y = k_height / 2;
        let k_center_x = k_width / 2;
        
        for y in k_center_y..(height - k_center_y) {
            for x in k_center_x..(width - k_center_x) {
                for c in 0..channels {
                    let mut sum = 0.0;
                    
                    for ky in 0..k_height {
                        for kx in 0..k_width {
                            let src_y = y + ky - k_center_y;
                            let src_x = x + kx - k_center_x;
                            sum += image[[src_y, src_x, c]] as f64 * kernel[[ky, kx]];
                        }
                    }
                    
                    result[[y, x, c]] = sum.clamp(0.0, 255.0) as u8;
                }
            }
        }
        
        result
    }
}
```

---

## Reinforcement Learning

### Q-Learning Agent

```rust
use std::collections::HashMap;

struct QLearningAgent {
    q_table: HashMap<(usize, usize), f64>,
    learning_rate: f64,
    discount_factor: f64,
    epsilon: f64,
    n_states: usize,
    n_actions: usize,
}

impl QLearningAgent {
    fn new(n_states: usize, n_actions: usize, learning_rate: f64, discount_factor: f64) -> Self {
        QLearningAgent {
            q_table: HashMap::new(),
            learning_rate,
            discount_factor,
            epsilon: 0.1,
            n_states,
            n_actions,
        }
    }
    
    fn get_q_value(&self, state: usize, action: usize) -> f64 {
        *self.q_table.get(&(state, action)).unwrap_or(&0.0)
    }
    
    fn set_q_value(&mut self, state: usize, action: usize, value: f64) {
        self.q_table.insert((state, action), value);
    }
    
    fn choose_action(&mut self, state: usize) -> usize {
        if rand::random::<f64>() < self.epsilon {
            // Explore: random action
            rand::random::<usize>() % self.n_actions
        } else {
            // Exploit: best action
            let mut best_action = 0;
            let mut best_value = f64::NEG_INFINITY;
            
            for action in 0..self.n_actions {
                let q_value = self.get_q_value(state, action);
                if q_value > best_value {
                    best_value = q_value;
                    best_action = action;
                }
            }
            
            best_action
        }
    }
    
    fn update(&mut self, state: usize, action: usize, reward: f64, next_state: usize) {
        let current_q = self.get_q_value(state, action);
        
        // Find maximum Q-value for next state
        let mut max_next_q = 0.0;
        for next_action in 0..self.n_actions {
            let next_q = self.get_q_value(next_state, next_action);
            if next_q > max_next_q {
                max_next_q = next_q;
            }
        }
        
        // Q-learning update rule
        let new_q = current_q + self.learning_rate * (reward + self.discount_factor * max_next_q - current_q);
        self.set_q_value(state, action, new_q);
    }
    
    fn decay_epsilon(&mut self, decay_rate: f64) {
        self.epsilon = (self.epsilon * decay_rate).max(0.01);
    }
}

// Simple environment for Q-learning
struct GridWorld {
    width: usize,
    height: usize,
    goal: (usize, usize),
    obstacles: Vec<(usize, usize)>,
}

impl GridWorld {
    fn new(width: usize, height: usize) -> Self {
        GridWorld {
            width,
            height,
            goal: (width - 1, height - 1),
            obstacles: vec![(1, 1), (2, 2), (3, 1)],
        }
    }
    
    fn state_to_index(&self, x: usize, y: usize) -> usize {
        y * self.width + x
    }
    
    fn index_to_state(&self, index: usize) -> (usize, usize) {
        (index % self.width, index / self.width)
    }
    
    fn is_valid_position(&self, x: usize, y: usize) -> bool {
        x < self.width && y < self.height && !self.obstacles.contains(&(x, y))
    }
    
    fn step(&self, state: usize, action: usize) -> (usize, f64, bool) {
        let (x, y) = self.index_to_state(state);
        let (mut new_x, mut new_y) = (x, y);
        
        match action {
            0 => new_y = new_y.saturating_sub(1), // Up
            1 => new_y = (new_y + 1).min(self.height - 1), // Down
            2 => new_x = new_x.saturating_sub(1), // Left
            3 => new_x = (new_x + 1).min(self.width - 1), // Right
            _ => {}
        }
        
        if !self.is_valid_position(new_x, new_y) {
            return (state, -1.0, false); // Penalty for hitting obstacle
        }
        
        let new_state = self.state_to_index(new_x, new_y);
        let reward = if (new_x, new_y) == self.goal { 10.0 } else { -0.1 };
        let done = (new_x, new_y) == self.goal;
        
        (new_state, reward, done)
    }
}
```

---

## Model Evaluation and Metrics

### Classification Metrics

```rust
struct ClassificationMetrics;

impl ClassificationMetrics {
    fn accuracy(y_true: &[i32], y_pred: &[i32]) -> f64 {
        let correct = y_true.iter().zip(y_pred.iter())
            .filter(|&(true, pred)| true == pred)
            .count();
        
        correct as f64 / y_true.len() as f64
    }
    
    fn precision(y_true: &[i32], y_pred: &[i32], positive_class: i32) -> f64 {
        let mut true_positives = 0;
        let mut false_positives = 0;
        
        for (&true_val, &pred_val) in y_true.iter().zip(y_pred.iter()) {
            if pred_val == positive_class {
                if true_val == positive_class {
                    true_positives += 1;
                } else {
                    false_positives += 1;
                }
            }
        }
        
        if true_positives + false_positives == 0 {
            0.0
        } else {
            true_positives as f64 / (true_positives + false_positives) as f64
        }
    }
    
    fn recall(y_true: &[i32], y_pred: &[i32], positive_class: i32) -> f64 {
        let mut true_positives = 0;
        let mut false_negatives = 0;
        
        for (&true_val, &pred_val) in y_true.iter().zip(y_pred.iter()) {
            if true_val == positive_class {
                if pred_val == positive_class {
                    true_positives += 1;
                } else {
                    false_negatives += 1;
                }
            }
        }
        
        if true_positives + false_negatives == 0 {
            0.0
        } else {
            true_positives as f64 / (true_positives + false_negatives) as f64
        }
    }
    
    fn f1_score(y_true: &[i32], y_pred: &[i32], positive_class: i32) -> f64 {
        let precision = Self::precision(y_true, y_pred, positive_class);
        let recall = Self::recall(y_true, y_pred, positive_class);
        
        if precision + recall == 0.0 {
            0.0
        } else {
            2.0 * precision * recall / (precision + recall)
        }
    }
    
    fn confusion_matrix(y_true: &[i32], y_pred: &[i32]) -> Vec<Vec<usize>> {
        let classes: std::collections::HashSet<_> = y_true.iter().chain(y_pred).cloned().collect();
        let n_classes = classes.len();
        let mut matrix = vec![vec![0; n_classes]; n_classes];
        
        let class_list: Vec<_> = classes.into_iter().collect();
        
        for (&true_val, &pred_val) in y_true.iter().zip(y_pred.iter()) {
            let true_idx = class_list.iter().position(|&c| c == true_val).unwrap();
            let pred_idx = class_list.iter().position(|&c| c == pred_val).unwrap();
            matrix[true_idx][pred_idx] += 1;
        }
        
        matrix
    }
}
```

---

## Key Takeaways

- **Performance** makes Rust excellent for ML workloads
- **Memory safety** prevents common ML bugs
- **Ecosystem** is growing with pure Rust ML libraries
- **Interoperability** with Python ML libraries is possible
- **Type safety** catches errors at compile time
- **Parallelism** is built into the language
- **GPU support** is available through bindings

---

## ML/AI Best Practices

| Practice | Description | Implementation |
|----------|-------------|----------------|
| **Data preprocessing** | Clean and normalize data | Use polars and custom pipelines |
| **Cross-validation** | Prevent overfitting | Implement k-fold validation |
| **Hyperparameter tuning** | Optimize model parameters | Use grid search or random search |
| **Model evaluation** | Measure performance | Use appropriate metrics |
| **Feature engineering** | Create meaningful features | Domain-specific transformations |
| **Regularization** | Prevent overfitting | L1/L2 regularization, dropout |
| **Ensemble methods** | Combine models | Voting, bagging, boosting |
