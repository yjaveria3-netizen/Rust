// 33_machine_learning_and_ai.rs
// Comprehensive examples of machine learning and AI in Rust

// Note: This file demonstrates ML concepts but requires proper
// ML libraries and data for production use

use std::collections::{HashMap, HashSet};
use std::f64;

// =========================================
// DATA STRUCTURES AND UTILITIES
// =========================================

#[derive(Debug, Clone)]
pub struct DataPoint {
    pub features: Vec<f64>,
    pub label: Option<f64>,
}

impl DataPoint {
    pub fn new(features: Vec<f64>, label: Option<f64>) -> Self {
        DataPoint { features, label }
    }
    
    pub fn unlabeled(features: Vec<f64>) -> Self {
        DataPoint { features, label: None }
    }
}

pub struct Dataset {
    pub data: Vec<DataPoint>,
    pub n_features: usize,
}

impl Dataset {
    pub fn new(data: Vec<DataPoint>) -> Self {
        let n_features = data.first().map(|d| d.features.len()).unwrap_or(0);
        Dataset { data, n_features }
    }
    
    pub fn split(&self, train_ratio: f64) -> (Dataset, Dataset) {
        let split_index = (self.data.len() as f64 * train_ratio) as usize;
        let train_data = self.data[..split_index].to_vec();
        let test_data = self.data[split_index..].to_vec();
        
        (Dataset::new(train_data), Dataset::new(test_data))
    }
    
    pub fn to_features(&self) -> Vec<Vec<f64>> {
        self.data.iter().map(|d| d.features.clone()).collect()
    }
    
    pub fn to_labels(&self) -> Vec<f64> {
        self.data.iter()
            .filter_map(|d| d.label)
            .collect()
    }
}

// =========================================
// LINEAR REGRESSION
// =========================================

pub struct LinearRegression {
    weights: Vec<f64>,
    bias: f64,
    learning_rate: f64,
}

impl LinearRegression {
    pub fn new(n_features: usize, learning_rate: f64) -> Self {
        LinearRegression {
            weights: vec![0.0; n_features],
            bias: 0.0,
            learning_rate,
        }
    }
    
    pub fn predict(&self, features: &[f64]) -> f64 {
        let mut prediction = self.bias;
        for (i, &weight) in self.weights.iter().enumerate() {
            if i < features.len() {
                prediction += weight * features[i];
            }
        }
        prediction
    }
    
    pub fn fit(&mut self, dataset: &Dataset, epochs: usize) {
        let features = dataset.to_features();
        let labels = dataset.to_labels();
        let n_samples = features.len();
        
        for _epoch in 0..epochs {
            let mut gradients = vec![0.0; self.weights.len()];
            let mut bias_gradient = 0.0;
            
            // Calculate gradients
            for i in 0..n_samples {
                let prediction = self.predict(&features[i]);
                let error = prediction - labels[i];
                
                for (j, &feature) in features[i].iter().enumerate() {
                    if j < gradients.len() {
                        gradients[j] += error * feature;
                    }
                }
                bias_gradient += error;
            }
            
            // Update weights and bias
            for (j, gradient) in gradients.iter().enumerate() {
                if j < self.weights.len() {
                    self.weights[j] -= self.learning_rate * gradient / n_samples as f64;
                }
            }
            self.bias -= self.learning_rate * bias_gradient / n_samples as f64;
        }
    }
    
    pub fn mse(&self, dataset: &Dataset) -> f64 {
        let features = dataset.to_features();
        let labels = dataset.to_labels();
        let mut total_error = 0.0;
        let n_samples = features.len();
        
        for i in 0..n_samples {
            let prediction = self.predict(&features[i]);
            let error = prediction - labels[i];
            total_error += error * error;
        }
        
        total_error / n_samples as f64
    }
}

// =========================================
// DECISION TREE
// =========================================

#[derive(Debug, Clone)]
pub enum DecisionNode {
    Leaf { value: f64 },
    Split {
        feature_index: usize,
        threshold: f64,
        left: Box<DecisionNode>,
        right: Box<DecisionNode>,
    },
}

pub struct DecisionTree {
    root: DecisionNode,
    max_depth: usize,
}

impl DecisionTree {
    pub fn new(max_depth: usize) -> Self {
        DecisionTree {
            root: DecisionNode::Leaf { value: 0.0 },
            max_depth,
        }
    }
    
    pub fn fit(&mut self, dataset: &Dataset) {
        let indices: Vec<usize> = (0..dataset.data.len()).collect();
        self.root = self.build_tree(dataset, &indices, 0);
    }
    
    fn build_tree(
        &self,
        dataset: &Dataset,
        indices: &[usize],
        depth: usize,
    ) -> DecisionNode {
        if depth >= self.max_depth || indices.len() <= 1 {
            let avg_value = indices.iter()
                .filter_map(|&i| dataset.data[i].label)
                .sum::<f64>() / indices.len() as f64;
            return DecisionNode::Leaf { value: avg_value };
        }
        
        let (best_feature, best_threshold, best_gain) = self.find_best_split(dataset, indices);
        
        if best_gain <= 0.0 {
            let avg_value = indices.iter()
                .filter_map(|&i| dataset.data[i].label)
                .sum::<f64>() / indices.len() as f64;
            return DecisionNode::Leaf { value: avg_value };
        }
        
        let (left_indices, right_indices) = self.split_indices(dataset, indices, best_feature, best_threshold);
        
        let left = self.build_tree(dataset, &left_indices, depth + 1);
        let right = self.build_tree(dataset, &right_indices, depth + 1);
        
        DecisionNode::Split {
            feature_index: best_feature,
            threshold: best_threshold,
            left: Box::new(left),
            right: Box::new(right),
        }
    }
    
    fn find_best_split(
        &self,
        dataset: &Dataset,
        indices: &[usize],
    ) -> (usize, f64, f64) {
        let mut best_feature = 0;
        let mut best_threshold = 0.0;
        let mut best_gain = 0.0;
        
        for feature in 0..dataset.n_features {
            let values: Vec<f64> = indices.iter()
                .filter_map(|&i| dataset.data[i].features.get(feature).copied())
                .collect();
            
            if values.is_empty() {
                continue;
            }
            
            let min_val = values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
            let max_val = values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
            
            // Try a few threshold values
            let n_thresholds = 10.min(values.len());
            for i in 0..n_thresholds {
                let threshold = min_val + (max_val - min_val) * (i as f64 / n_thresholds as f64);
                let gain = self.calculate_information_gain(dataset, indices, feature, threshold);
                
                if gain > best_gain {
                    best_gain = gain;
                    best_feature = feature;
                    best_threshold = threshold;
                }
            }
        }
        
        (best_feature, best_threshold, best_gain)
    }
    
    fn calculate_information_gain(
        &self,
        dataset: &Dataset,
        indices: &[usize],
        feature: usize,
        threshold: f64,
    ) -> f64 {
        let (left_indices, right_indices) = self.split_indices(dataset, indices, feature, threshold);
        
        if left_indices.is_empty() || right_indices.is_empty() {
            return 0.0;
        }
        
        let parent_entropy = self.calculate_entropy(dataset, indices);
        let left_entropy = self.calculate_entropy(dataset, &left_indices);
        let right_entropy = self.calculate_entropy(dataset, &right_indices);
        
        let left_weight = left_indices.len() as f64 / indices.len() as f64;
        let right_weight = right_indices.len() as f64 / indices.len() as f64;
        
        parent_entropy - (left_weight * left_entropy + right_weight * right_entropy)
    }
    
    fn calculate_entropy(&self, dataset: &Dataset, indices: &[usize]) -> f64 {
        let mut counts = HashMap::new();
        
        for &i in indices {
            if let Some(label) = dataset.data[i].label {
                *counts.entry(label).or_insert(0) += 1;
            }
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
        dataset: &Dataset,
        indices: &[usize],
        feature: usize,
        threshold: f64,
    ) -> (Vec<usize>, Vec<usize>) {
        let mut left = Vec::new();
        let mut right = Vec::new();
        
        for &i in indices {
            if let Some(feature_value) = dataset.data[i].features.get(feature) {
                if *feature_value <= threshold {
                    left.push(i);
                } else {
                    right.push(i);
                }
            }
        }
        
        (left, right)
    }
    
    pub fn predict(&self, features: &[f64]) -> f64 {
        self.predict_node(&self.root, features)
    }
    
    fn predict_node(&self, node: &DecisionNode, features: &[f64]) -> f64 {
        match node {
            DecisionNode::Leaf { value } => *value,
            DecisionNode::Split { feature_index, threshold, left, right } => {
                if let Some(feature_value) = features.get(*feature_index) {
                    if *feature_value <= *threshold {
                        self.predict_node(left, features)
                    } else {
                        self.predict_node(right, features)
                    }
                } else {
                    0.0 // Default value if feature is missing
                }
            }
        }
    }
}

// =========================================
// K-NEAREST NEIGHBORS
// =========================================

pub struct KNN {
    k: usize,
    training_data: Vec<DataPoint>,
}

impl KNN {
    pub fn new(k: usize) -> Self {
        KNN {
            k,
            training_data: Vec::new(),
        }
    }
    
    pub fn fit(&mut self, dataset: &Dataset) {
        self.training_data = dataset.data.clone();
    }
    
    pub fn predict(&self, features: &[f64]) -> Option<f64> {
        if self.training_data.is_empty() {
            return None;
        }
        
        let mut distances: Vec<(f64, f64)> = Vec::new();
        
        for point in &self.training_data {
            if let Some(label) = point.label {
                let distance = self.euclidean_distance(features, &point.features);
                distances.push((distance, label));
            }
        }
        
        // Sort by distance
        distances.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
        
        // Take k nearest neighbors
        let k_nearest = distances.iter().take(self.k);
        
        // Average the labels
        let sum: f64 = k_nearest.map(|(_, label)| label).sum();
        Some(sum / self.k as f64)
    }
    
    fn euclidean_distance(&self, a: &[f64], b: &[f64]) -> f64 {
        let mut sum = 0.0;
        let min_len = a.len().min(b.len());
        
        for i in 0..min_len {
            sum += (a[i] - b[i]).powi(2);
        }
        
        sum.sqrt()
    }
}

// =========================================
// K-MEANS CLUSTERING
// =========================================

pub struct KMeans {
    k: usize,
    centroids: Vec<Vec<f64>>,
    max_iterations: usize,
}

impl KMeans {
    pub fn new(k: usize, max_iterations: usize) -> Self {
        KMeans {
            k,
            centroids: Vec::new(),
            max_iterations,
        }
    }
    
    pub fn fit(&mut self, dataset: &Dataset) {
        let features = dataset.to_features();
        let n_features = dataset.n_features;
        
        // Initialize centroids randomly
        self.centroids = self.initialize_centroids(&features, n_features);
        
        for _iteration in 0..self.max_iterations {
            // Assign points to clusters
            let clusters = self.assign_to_clusters(&features);
            
            // Update centroids
            let new_centroids = self.update_centroids(&features, &clusters, n_features);
            
            // Check for convergence
            if self.centroids_converged(&new_centroids) {
                break;
            }
            
            self.centroids = new_centroids;
        }
    }
    
    fn initialize_centroids(&self, features: &[Vec<f64>], n_features: usize) -> Vec<Vec<f64>> {
        let mut centroids = Vec::new();
        let n_samples = features.len();
        
        for i in 0..self.k {
            let sample_index = (i * n_samples / self.k) % n_samples;
            centroids.push(features[sample_index].clone());
        }
        
        centroids
    }
    
    fn assign_to_clusters(&self, features: &[Vec<f64>]) -> Vec<usize> {
        features.iter()
            .map(|point| {
                let mut min_distance = f64::INFINITY;
                let mut best_cluster = 0;
                
                for (cluster_idx, centroid) in self.centroids.iter().enumerate() {
                    let distance = self.euclidean_distance(point, centroid);
                    if distance < min_distance {
                        min_distance = distance;
                        best_cluster = cluster_idx;
                    }
                }
                
                best_cluster
            })
            .collect()
    }
    
    fn update_centroids(&self, features: &[Vec<f64>], clusters: &[usize], n_features: usize) -> Vec<Vec<f64>> {
        let mut new_centroids = vec![vec![0.0; n_features]; self.k];
        let mut cluster_counts = vec![0; self.k];
        
        // Sum up points in each cluster
        for (point_idx, point) in features.iter().enumerate() {
            let cluster = clusters[point_idx];
            cluster_counts[cluster] += 1;
            
            for (feature_idx, &value) in point.iter().enumerate() {
                new_centroids[cluster][feature_idx] += value;
            }
        }
        
        // Average the points
        for cluster in 0..self.k {
            if cluster_counts[cluster] > 0 {
                for feature_idx in 0..n_features {
                    new_centroids[cluster][feature_idx] /= cluster_counts[cluster] as f64;
                }
            }
        }
        
        new_centroids
    }
    
    fn centroids_converged(&self, new_centroids: &[Vec<f64>]) -> bool {
        if self.centroids.len() != new_centroids.len() {
            return false;
        }
        
        for (old_centroid, new_centroid) in self.centroids.iter().zip(new_centroids.iter()) {
            for (old_val, new_val) in old_centroid.iter().zip(new_centroid.iter()) {
                if (old_val - new_val).abs() > 1e-6 {
                    return false;
                }
            }
        }
        
        true
    }
    
    pub fn predict(&self, features: &[f64]) -> usize {
        let mut min_distance = f64::INFINITY;
        let mut best_cluster = 0;
        
        for (cluster_idx, centroid) in self.centroids.iter().enumerate() {
            let distance = self.euclidean_distance(features, centroid);
            if distance < min_distance {
                min_distance = distance;
                best_cluster = cluster_idx;
            }
        }
        
        best_cluster
    }
    
    fn euclidean_distance(&self, a: &[f64], b: &[f64]) -> f64 {
        let mut sum = 0.0;
        let min_len = a.len().min(b.len());
        
        for i in 0..min_len {
            sum += (a[i] - b[i]).powi(2);
        }
        
        sum.sqrt()
    }
}

// =========================================
// NEURAL NETWORK (SIMPLIFIED)
// =========================================

pub struct SimpleNeuralNetwork {
    weights_input_hidden: Vec<Vec<f64>>,
    weights_hidden_output: Vec<f64>,
    bias_hidden: Vec<f64>,
    bias_output: f64,
    learning_rate: f64,
}

impl SimpleNeuralNetwork {
    pub fn new(input_size: usize, hidden_size: usize, learning_rate: f64) -> Self {
        SimpleNeuralNetwork {
            weights_input_hidden: vec![vec![0.0; hidden_size]; input_size],
            weights_hidden_output: vec![0.0; hidden_size],
            bias_hidden: vec![0.0; hidden_size],
            bias_output: 0.0,
            learning_rate,
        }
    }
    
    fn sigmoid(x: f64) -> f64 {
        1.0 / (1.0 + (-x).exp())
    }
    
    fn sigmoid_derivative(x: f64) -> f64 {
        let s = Self::sigmoid(x);
        s * (1.0 - s)
    }
    
    fn forward(&self, inputs: &[f64]) -> (Vec<f64>, Vec<f64>, f64) {
        // Hidden layer
        let mut hidden = Vec::with_capacity(self.weights_input_hidden[0].len());
        for (i, weights) in self.weights_input_hidden.iter().enumerate() {
            let mut sum = self.bias_hidden[i];
            for (j, &input) in inputs.iter().enumerate() {
                if j < weights.len() {
                    sum += weights[j] * input;
                }
            }
            hidden.push(Self::sigmoid(sum));
        }
        
        // Output layer
        let mut output = self.bias_output;
        for (i, &hidden_val) in hidden.iter().enumerate() {
            if i < self.weights_hidden_output.len() {
                output += self.weights_hidden_output[i] * hidden_val;
            }
        }
        
        (hidden, Self::sigmoid(output), output)
    }
    
    pub fn predict(&self, inputs: &[f64]) -> f64 {
        let (_, output, _) = self.forward(inputs);
        output
    }
    
    pub fn train(&mut self, dataset: &Dataset, epochs: usize) {
        let features = dataset.to_features();
        let labels = dataset.to_labels();
        
        for _epoch in 0..epochs {
            for (sample_idx, inputs) in features.iter().enumerate() {
                let target = labels[sample_idx];
                
                // Forward pass
                let (hidden, output, raw_output) = self.forward(inputs);
                
                // Calculate error
                let output_error = target - output;
                let output_delta = output_error * Self::sigmoid_derivative(raw_output);
                
                // Backpropagate to hidden layer
                let mut hidden_errors = Vec::with_capacity(hidden.len());
                for (i, &hidden_val) in hidden.iter().enumerate() {
                    if i < self.weights_hidden_output.len() {
                        hidden_errors.push(output_delta * self.weights_hidden_output[i]);
                    }
                }
                
                // Update weights and biases
                // Update hidden to output weights
                for (i, &hidden_val) in hidden.iter().enumerate() {
                    if i < self.weights_hidden_output.len() {
                        self.weights_hidden_output[i] += self.learning_rate * output_delta * hidden_val;
                    }
                }
                self.bias_output += self.learning_rate * output_delta;
                
                // Update input to hidden weights
                for (i, weights) in self.weights_input_hidden.iter_mut().enumerate() {
                    if i < hidden_errors.len() {
                        let hidden_delta = hidden_errors[i] * Self::sigmoid_derivative(hidden[i]);
                        for (j, &input) in inputs.iter().enumerate() {
                            if j < weights.len() {
                                weights[j] += self.learning_rate * hidden_delta * input;
                            }
                        }
                        self.bias_hidden[i] += self.learning_rate * hidden_delta;
                    }
                }
            }
        }
    }
}

// =========================================
// MODEL EVALUATION
// =========================================

pub struct ModelEvaluator;

impl ModelEvaluator {
    pub fn accuracy(y_true: &[f64], y_pred: &[f64]) -> f64 {
        let correct = y_true.iter().zip(y_pred.iter())
            .filter(|&(true, pred)| (true - pred).abs() < 0.5)
            .count();
        
        correct as f64 / y_true.len() as f64
    }
    
    pub fn mean_squared_error(y_true: &[f64], y_pred: &[f64]) -> f64 {
        let total_error: f64 = y_true.iter().zip(y_pred.iter())
            .map(|(true_val, pred_val)| (true_val - pred_val).powi(2))
            .sum();
        
        total_error / y_true.len() as f64
    }
    
    pub fn mean_absolute_error(y_true: &[f64], y_pred: &[f64]) -> f64 {
        let total_error: f64 = y_true.iter().zip(y_pred.iter())
            .map(|(true_val, pred_val)| (true_val - pred_val).abs())
            .sum();
        
        total_error / y_true.len() as f64
    }
    
    pub fn r_squared(y_true: &[f64], y_pred: &[f64]) -> f64 {
        let mean_true = y_true.iter().sum::<f64>() / y_true.len() as f64;
        
        let total_sum_squares: f64 = y_true.iter()
            .map(|y| (y - mean_true).powi(2))
            .sum();
        
        let residual_sum_squares: f64 = y_true.iter().zip(y_pred.iter())
            .map(|(true_val, pred_val)| (true_val - pred_val).powi(2))
            .sum();
        
        1.0 - (residual_sum_squares / total_sum_squares)
    }
}

// =========================================
// FEATURE ENGINEERING
// =========================================

pub struct FeatureEngineer {
    categorical_mappings: HashMap<String, HashMap<String, f64>>,
}

impl FeatureEngineer {
    pub fn new() -> Self {
        FeatureEngineer {
            categorical_mappings: HashMap::new(),
        }
    }
    
    pub fn fit_categorical(&mut self, data: &[String], feature_name: &str) {
        let mut mapping = HashMap::new();
        let unique_values: HashSet<_> = data.iter().cloned().collect();
        
        for (i, value) in unique_values.iter().enumerate() {
            mapping.insert(value.clone(), i as f64);
        }
        
        self.categorical_mappings.insert(feature_name.to_string(), mapping);
    }
    
    pub fn transform_categorical(&self, value: &str, feature_name: &str) -> f64 {
        if let Some(mapping) = self.categorical_mappings.get(feature_name) {
            mapping.get(value).copied().unwrap_or(-1.0)
        } else {
            -1.0
        }
    }
    
    pub fn normalize_features(&self, features: &mut [Vec<f64>]) {
        if features.is_empty() {
            return;
        }
        
        let n_features = features[0].len();
        
        for j in 0..n_features {
            let mut sum = 0.0;
            let mut sum_sq = 0.0;
            let n = features.len();
            
            for i in 0..n {
                if j < features[i].len() {
                    let val = features[i][j];
                    sum += val;
                    sum_sq += val * val;
                }
            }
            
            let mean = sum / n as f64;
            let variance = (sum_sq / n as f64) - (mean * mean);
            let std_dev = variance.sqrt();
            
            for i in 0..n {
                if j < features[i].len() {
                    if std_dev > 0.0 {
                        features[i][j] = (features[i][j] - mean) / std_dev;
                    }
                }
            }
        }
    }
    
    pub fn polynomial_features(&self, features: &[f64], degree: usize) -> Vec<f64> {
        let mut poly_features = Vec::new();
        
        for d in 0..=degree {
            for &feature in features {
                poly_features.push(feature.powi(d as i32));
            }
        }
        
        poly_features
    }
}

// =========================================
// DEMONSTRATION FUNCTIONS
// =========================================

pub fn demonstrate_linear_regression() {
    println!("=== LINEAR REGRESSION DEMONSTRATION ===");
    
    // Create sample data
    let data = vec![
        DataPoint::new(vec![1.0, 2.0], Some(3.0)),
        DataPoint::new(vec![2.0, 3.0], Some(5.0)),
        DataPoint::new(vec![3.0, 4.0], Some(7.0)),
        DataPoint::new(vec![4.0, 5.0], Some(9.0)),
        DataPoint::new(vec![5.0, 6.0], Some(11.0)),
    ];
    
    let dataset = Dataset::new(data);
    let mut model = LinearRegression::new(2, 0.01);
    
    println!("Training linear regression...");
    model.fit(&dataset, 1000);
    
    // Test predictions
    let test_features = vec![vec![6.0, 7.0], vec![10.0, 11.0]];
    for features in test_features {
        let prediction = model.predict(&features);
        println!("Features: {:?} -> Prediction: {:.2}", features, prediction);
    }
    
    let mse = model.mse(&dataset);
    println!("MSE: {:.4}", mse);
    
    println!();
}

pub fn demonstrate_decision_tree() {
    println!("=== DECISION TREE DEMONSTRATION ===");
    
    // Create sample data
    let data = vec![
        DataPoint::new(vec![1.0, 1.0], Some(0.0)),
        DataPoint::new(vec![1.0, 2.0], Some(0.0)),
        DataPoint::new(vec![2.0, 1.0], Some(0.0)),
        DataPoint::new(vec![2.0, 2.0], Some(1.0)),
        DataPoint::new(vec![3.0, 1.0], Some(1.0)),
        DataPoint::new(vec![3.0, 2.0], Some(1.0)),
    ];
    
    let dataset = Dataset::new(data);
    let mut model = DecisionTree::new(3);
    
    println!("Training decision tree...");
    model.fit(&dataset);
    
    // Test predictions
    let test_features = vec![vec![2.5, 1.5], vec![1.5, 2.5]];
    for features in test_features {
        let prediction = model.predict(&features);
        println!("Features: {:?} -> Prediction: {:.2}", features, prediction);
    }
    
    println!();
}

pub fn demonstrate_knn() {
    println!("=== K-NEAREST NEIGHBORS DEMONSTRATION ===");
    
    // Create sample data
    let data = vec![
        DataPoint::new(vec![1.0, 2.0], Some(1.0)),
        DataPoint::new(vec![2.0, 3.0], Some(1.0)),
        DataPoint::new(vec![3.0, 3.0], Some(2.0)),
        DataPoint::new(vec![6.0, 5.0], Some(2.0)),
        DataPoint::new(vec![7.0, 7.0], Some(2.0)),
    ];
    
    let dataset = Dataset::new(data);
    let mut model = KNN::new(3);
    
    println!("Training KNN...");
    model.fit(&dataset);
    
    // Test predictions
    let test_features = vec![vec![2.5, 2.5], vec![5.0, 6.0]];
    for features in test_features {
        if let Some(prediction) = model.predict(&features) {
            println!("Features: {:?} -> Prediction: {:.2}", features, prediction);
        }
    }
    
    println!();
}

pub fn demonstrate_kmeans() {
    println!("=== K-MEANS CLUSTERING DEMONSTRATION ===");
    
    // Create sample data
    let data = vec![
        DataPoint::unlabeled(vec![1.0, 1.0]),
        DataPoint::unlabeled(vec![1.5, 2.0]),
        DataPoint::unlabeled(vec![3.0, 4.0]),
        DataPoint::unlabeled(vec![5.0, 7.0]),
        DataPoint::unlabeled(vec![3.5, 5.0]),
        DataPoint::unlabeled(vec![4.5, 5.0]),
        DataPoint::unlabeled(vec![3.5, 4.5]),
    ];
    
    let dataset = Dataset::new(data);
    let mut model = KMeans::new(2, 100);
    
    println!("Training K-means...");
    model.fit(&dataset);
    
    // Test clustering
    let test_features = vec![vec![2.0, 2.0], vec![4.0, 5.0], vec![6.0, 6.0]];
    for features in test_features {
        let cluster = model.predict(&features);
        println!("Features: {:?} -> Cluster: {}", features, cluster);
    }
    
    println!();
}

pub fn demonstrate_neural_network() {
    println!("=== NEURAL NETWORK DEMONSTRATION ===");
    
    // Create sample data (XOR problem)
    let data = vec![
        DataPoint::new(vec![0.0, 0.0], Some(0.0)),
        DataPoint::new(vec![0.0, 1.0], Some(1.0)),
        DataPoint::new(vec![1.0, 0.0], Some(1.0)),
        DataPoint::new(vec![1.0, 1.0], Some(0.0)),
    ];
    
    let dataset = Dataset::new(data);
    let mut model = SimpleNeuralNetwork::new(2, 4, 0.1);
    
    println!("Training neural network...");
    model.train(&dataset, 10000);
    
    // Test predictions
    let test_features = vec![vec![0.0, 0.0], vec![0.0, 1.0], vec![1.0, 0.0], vec![1.0, 1.0]];
    for features in test_features {
        let prediction = model.predict(&features);
        println!("Features: {:?} -> Prediction: {:.2}", features, prediction);
    }
    
    println!();
}

pub fn demonstrate_feature_engineering() {
    println!("=== FEATURE ENGINEERING DEMONSTRATION ===");
    
    let mut engineer = FeatureEngineer::new();
    
    // Sample categorical data
    let colors = vec!["red".to_string(), "blue".to_string(), "green".to_string()];
    engineer.fit_categorical(&colors, "color");
    
    let encoded_red = engineer.transform_categorical("red", "color");
    let encoded_blue = engineer.transform_categorical("blue", "color");
    let encoded_green = engineer.transform_categorical("green", "color");
    
    println!("Color encoding:");
    println!("  red -> {}", encoded_red);
    println!("  blue -> {}", encoded_blue);
    println!("  green -> {}", encoded_green);
    
    // Sample numerical data
    let mut features = vec![
        vec![1.0, 2.0, 3.0],
        vec![4.0, 5.0, 6.0],
        vec![7.0, 8.0, 9.0],
    ];
    
    println!("Original features: {:?}", features);
    
    engineer.normalize_features(&mut features);
    println!("Normalized features: {:?}", features);
    
    let poly_features = engineer.polynomial_features(&[2.0, 3.0], 2);
    println!("Polynomial features (degree=2): {:?}", poly_features);
    
    println!();
}

pub fn demonstrate_model_evaluation() {
    println!("=== MODEL EVALUATION DEMONSTRATION ===");
    
    let y_true = vec![1.0, 2.0, 3.0, 4.0, 5.0];
    let y_pred = vec![1.1, 1.9, 3.2, 3.8, 5.1];
    
    let accuracy = ModelEvaluator::accuracy(&y_true, &y_pred);
    let mse = ModelEvaluator::mean_squared_error(&y_true, &y_pred);
    let mae = ModelEvaluator::mean_absolute_error(&y_true, &y_pred);
    let r2 = ModelEvaluator::r_squared(&y_true, &y_pred);
    
    println!("Model evaluation metrics:");
    println!("  Accuracy: {:.4}", accuracy);
    println!("  MSE: {:.4}", mse);
    println!("  MAE: {:.4}", mae);
    println!("  R²: {:.4}", r2);
    
    println!();
}

// =========================================
// MAIN DEMONSTRATION
// =========================================

fn main() {
    println!("=== MACHINE LEARNING AND AI DEMONSTRATIONS ===\n");
    
    demonstrate_linear_regression();
    demonstrate_decision_tree();
    demonstrate_knn();
    demonstrate_kmeans();
    demonstrate_neural_network();
    demonstrate_feature_engineering();
    demonstrate_model_evaluation();
    
    println!("=== MACHINE LEARNING AND AI DEMONSTRATIONS COMPLETE ===");
    println!("Note: This uses simplified ML algorithms. Real implementations should:");
    println!("- Use established ML libraries like linfa, candle, or tch");
    println!("- Include proper data preprocessing and validation");
    println!("- Implement cross-validation and hyperparameter tuning");
    println!("- Handle edge cases and numerical stability");
    println!("- Use GPU acceleration for deep learning");
}

// =========================================
// UNIT TESTS
// =========================================

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_linear_regression() {
        let data = vec![
            DataPoint::new(vec![1.0], Some(2.0)),
            DataPoint::new(vec![2.0], Some(4.0)),
            DataPoint::new(vec![3.0], Some(6.0)),
        ];
        
        let dataset = Dataset::new(data);
        let mut model = LinearRegression::new(1, 0.1);
        model.fit(&dataset, 100);
        
        let prediction = model.predict(&[4.0]);
        assert!((prediction - 8.0).abs() < 0.1);
    }
    
    #[test]
    fn test_dataset_split() {
        let data = vec![
            DataPoint::new(vec![1.0], Some(1.0)),
            DataPoint::new(vec![2.0], Some(2.0)),
            DataPoint::new(vec![3.0], Some(3.0)),
            DataPoint::new(vec![4.0], Some(4.0)),
        ];
        
        let dataset = Dataset::new(data);
        let (train, test) = dataset.split(0.5);
        
        assert_eq!(train.data.len(), 2);
        assert_eq!(test.data.len(), 2);
    }
    
    #[test]
    fn test_knn() {
        let data = vec![
            DataPoint::new(vec![1.0], Some(1.0)),
            DataPoint::new(vec![2.0], Some(2.0)),
            DataPoint::new(vec![3.0], Some(3.0)),
        ];
        
        let dataset = Dataset::new(data);
        let mut model = KNN::new(1);
        model.fit(&dataset);
        
        let prediction = model.predict(&[2.5]);
        assert!(prediction.is_some());
        assert!((prediction.unwrap() - 2.0).abs() < 0.1);
    }
    
    #[test]
    fn test_kmeans() {
        let data = vec![
            DataPoint::unlabeled(vec![1.0]),
            DataPoint::unlabeled(vec![2.0]),
            DataPoint::unlabeled(vec![8.0]),
            DataPoint::unlabeled(vec![9.0]),
        ];
        
        let dataset = Dataset::new(data);
        let mut model = KMeans::new(2, 10);
        model.fit(&dataset);
        
        // Should cluster [1,2] and [8,9]
        let cluster1 = model.predict(&[1.5]);
        let cluster2 = model.predict(&[8.5]);
        
        assert_ne!(cluster1, cluster2);
    }
    
    #[test]
    fn test_feature_engineering() {
        let mut engineer = FeatureEngineer::new();
        
        let colors = vec!["red".to_string(), "blue".to_string()];
        engineer.fit_categorical(&colors, "color");
        
        let encoded = engineer.transform_categorical("red", "color");
        assert_eq!(encoded, 0.0);
        
        let mut features = vec![vec![1.0, 2.0], vec![3.0, 4.0]];
        engineer.normalize_features(&mut features);
        
        // After normalization, mean should be ~0 and std ~1
        let mean1 = features[0].iter().sum::<f64>() / features[0].len() as f64;
        let mean2 = features[1].iter().sum::<f64>() / features[1].len() as f64;
        
        assert!(mean1.abs() < 1e-10);
        assert!(mean2.abs() < 1e-10);
    }
    
    #[test]
    fn test_model_evaluation() {
        let y_true = vec![1.0, 2.0, 3.0];
        let y_pred = vec![1.0, 2.0, 3.0];
        
        let accuracy = ModelEvaluator::accuracy(&y_true, &y_pred);
        let mse = ModelEvaluator::mean_squared_error(&y_true, &y_pred);
        
        assert_eq!(accuracy, 1.0);
        assert_eq!(mse, 0.0);
    }
    
    #[test]
    fn test_neural_network() {
        let data = vec![
            DataPoint::new(vec![0.0], Some(0.0)),
            DataPoint::new(vec![1.0], Some(1.0)),
        ];
        
        let dataset = Dataset::new(data);
        let mut model = SimpleNeuralNetwork::new(1, 2, 0.5);
        model.train(&dataset, 100);
        
        let prediction = model.predict(&[0.0]);
        assert!(prediction < 0.5);
        
        let prediction = model.predict(&[1.0]);
        assert!(prediction > 0.5);
    }
}
