// 32_security_and_cryptography.rs
// Comprehensive examples of security and cryptography in Rust

// Note: This file demonstrates security concepts but requires proper
// cryptographic libraries and security practices for production use

use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fmt;

// =========================================
// CRYPTOGRAPHIC PRIMITIVES
// =========================================

// Simulated SHA-256 hash function
pub struct Sha256Hasher {
    data: Vec<u8>,
}

impl Sha256Hasher {
    pub fn new() -> Self {
        Sha256Hasher {
            data: Vec::new(),
        }
    }
    
    pub fn update(&mut self, data: &[u8]) {
        self.data.extend_from_slice(data);
    }
    
    pub fn finalize(&self) -> [u8; 32] {
        // Simulated SHA-256 (in real implementation, use proper crypto library)
        let mut result = [0u8; 32];
        let mut hash = self.data.len() as u32;
        
        for (i, &byte) in self.data.iter().enumerate() {
            hash = hash.wrapping_mul(31).wrapping_add(byte as u32);
            result[i % 32] = (hash >> (i % 8)) as u8;
        }
        
        result
    }
    
    pub fn hash(data: &[u8]) -> [u8; 32] {
        let mut hasher = Sha256Hasher::new();
        hasher.update(data);
        hasher.finalize()
    }
}

// Simulated HMAC implementation
pub struct HmacSha256 {
    key: Vec<u8>,
}

impl HmacSha256 {
    pub fn new(key: &[u8]) -> Self {
        HmacSha256 {
            key: key.to_vec(),
        }
    }
    
    pub fn sign(&self, data: &[u8]) -> [u8; 32] {
        // Simulated HMAC (in real implementation, use proper crypto library)
        let mut combined = Vec::new();
        combined.extend_from_slice(&self.key);
        combined.extend_from_slice(data);
        Sha256Hasher::hash(&combined)
    }
    
    pub fn verify(&self, data: &[u8], signature: &[u8]) -> bool {
        let computed = self.sign(data);
        &computed[..] == signature
    }
}

// Simulated AES-GCM encryption
pub struct Aes256Gcm {
    key: [u8; 32],
}

impl Aes256Gcm {
    pub fn new(key: [u8; 32]) -> Self {
        Aes256Gcm { key }
    }
    
    pub fn encrypt(&self, plaintext: &[u8]) -> Result<(Vec<u8>, [u8; 12]), CryptoError> {
        // Simulated AES-GCM encryption (in real implementation, use proper crypto library)
        let mut ciphertext = Vec::with_capacity(plaintext.len());
        let mut nonce = [0u8; 12];
        
        // Simple XOR-based "encryption" for demonstration
        for (i, &byte) in plaintext.iter().enumerate() {
            ciphertext.push(byte ^ self.key[i % 32]);
        }
        
        // Generate pseudo-random nonce
        for i in 0..12 {
            nonce[i] = (self.key[i] ^ self.key[i + 16]) ^ (i as u8);
        }
        
        Ok((ciphertext, nonce))
    }
    
    pub fn decrypt(&self, ciphertext: &[u8], nonce: &[u8; 12]) -> Result<Vec<u8>, CryptoError> {
        // Simulated AES-GCM decryption
        let mut plaintext = Vec::with_capacity(ciphertext.len());
        
        for (i, &byte) in ciphertext.iter().enumerate() {
            plaintext.push(byte ^ self.key[i % 32]);
        }
        
        Ok(plaintext)
    }
}

// Simulated Ed25519 keypair
#[derive(Debug, Clone)]
pub struct Ed25519Keypair {
    pub private_key: [u8; 32],
    pub public_key: [u8; 32],
}

impl Ed25519Keypair {
    pub fn generate() -> Self {
        // Simulated key generation (in real implementation, use proper crypto library)
        let mut private_key = [0u8; 32];
        let mut public_key = [0u8; 32];
        
        // Generate pseudo-random private key
        for i in 0..32 {
            private_key[i] = ((i * 7 + 13) % 256) as u8;
        }
        
        // Derive public key (simplified)
        for i in 0..32 {
            public_key[i] = private_key[i].wrapping_mul(3).wrapping_add(7);
        }
        
        Ed25519Keypair {
            private_key,
            public_key,
        }
    }
    
    pub fn sign(&self, message: &[u8]) -> [u8; 64] {
        // Simulated signature (in real implementation, use proper crypto library)
        let mut signature = [0u8; 64];
        let hash = Sha256Hasher::hash(message);
        
        // Simple signature simulation
        for i in 0..32 {
            signature[i] = self.private_key[i] ^ hash[i];
            signature[i + 32] = hash[i] ^ (i as u8);
        }
        
        signature
    }
    
    pub fn verify(&self, message: &[u8], signature: &[u8]) -> bool {
        let hash = Sha256Hasher::hash(message);
        
        // Simple verification
        for i in 0..32 {
            if signature[i] != (self.private_key[i] ^ hash[i]) {
                return false;
            }
            if signature[i + 32] != (hash[i] ^ (i as u8)) {
                return false;
            }
        }
        
        true
    }
}

// =========================================
// PASSWORD HASHING
// =========================================

pub struct Argon2Hasher {
    memory_cost: u32,
    time_cost: u32,
    parallelism: u32,
}

impl Argon2Hasher {
    pub fn new() -> Self {
        Argon2Hasher {
            memory_cost: 65536,
            time_cost: 3,
            parallelism: 4,
        }
    }
    
    pub fn with_params(memory_cost: u32, time_cost: u32, parallelism: u32) -> Self {
        Argon2Hasher {
            memory_cost,
            time_cost,
            parallelism,
        }
    }
    
    pub fn hash_password(&self, password: &str, salt: &[u8]) -> Result<String, CryptoError> {
        // Simulated Argon2 hashing (in real implementation, use proper crypto library)
        let mut hasher = Sha256Hasher::new();
        hasher.update(password.as_bytes());
        hasher.update(salt);
        
        // Apply multiple rounds to simulate memory-hard function
        let mut hash = hasher.finalize();
        for _ in 0..self.time_cost {
            hash = Sha256Hasher::hash(&hash);
        }
        
        // Format as $argon2id$v=19$m=65536,t=3,p=4$salt$hash
        let salt_hex = hex::encode(salt);
        let hash_hex = hex::encode(&hash);
        
        Ok(format!(
            "$argon2id$v=19$m={},t={},p=${}${}${}",
            self.memory_cost, self.time_cost, self.parallelism, salt_hex, hash_hex
        ))
    }
    
    pub fn verify_password(&self, password: &str, hash: &str) -> Result<bool, CryptoError> {
        // Parse the hash format
        let parts: Vec<&str> = hash.split('$').collect();
        if parts.len() != 6 {
            return Err(CryptoError::InvalidFormat);
        }
        
        let salt_hex = parts[4];
        let salt = hex::decode(salt_hex).map_err(|_| CryptoError::InvalidFormat)?;
        
        // Rehash with same parameters and compare
        let computed_hash = self.hash_password(password, &salt)?;
        Ok(computed_hash == hash)
    }
}

// =========================================
// SECURE RANDOM NUMBER GENERATION
// =========================================

pub struct SecureRng {
    state: u64,
}

impl SecureRng {
    pub fn new() -> Self {
        // In real implementation, use OS entropy
        SecureRng {
            state: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos() as u64,
        }
    }
    
    pub fn with_seed(seed: u64) -> Self {
        SecureRng { state: seed }
    }
    
    pub fn fill_bytes(&mut self, bytes: &mut [u8]) {
        for byte in bytes.iter_mut() {
            self.state = self.state.wrapping_mul(1103515245).wrapping_add(12345);
            *byte = (self.state >> 8) as u8;
        }
    }
    
    pub fn gen_range(&mut self, min: u64, max: u64) -> u64 {
        let range = max - min;
        if range == 0 {
            return min;
        }
        
        self.state = self.state.wrapping_mul(1103515245).wrapping_add(12345);
        min + (self.state % range)
    }
}

// =========================================
// AUTHENTICATION AND AUTHORIZATION
// =========================================

#[derive(Debug, Clone)]
pub struct JwtClaims {
    pub sub: String,        // Subject (user ID)
    pub exp: u64,          // Expiration time
    pub iat: u64,          // Issued at
    pub role: String,       // User role
}

pub struct JwtManager {
    secret: Vec<u8>,
}

impl JwtManager {
    pub fn new(secret: &str) -> Self {
        JwtManager {
            secret: secret.as_bytes().to_vec(),
        }
    }
    
    pub fn create_token(&self, claims: &JwtClaims) -> Result<String, CryptoError> {
        // Simulated JWT creation (in real implementation, use proper JWT library)
        let header = b"eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9"; // {"alg":"HS256","typ":"JWT"}
        let payload = format!(
            r#"{{"sub":"{}","exp":{},"iat":{},"role":"{}"}}"#,
            claims.sub, claims.exp, claims.iat, claims.role
        );
        
        let mut data = Vec::new();
        data.extend_from_slice(header);
        data.push(b'.');
        data.extend_from_slice(payload.as_bytes());
        
        let signature = HmacSha256::new(&self.secret).sign(&data);
        
        let token = format!(
            "{}.{}.{}",
            String::from_utf8_lossy(header),
            payload,
            hex::encode(&signature)
        );
        
        Ok(token)
    }
    
    pub fn verify_token(&self, token: &str) -> Result<JwtClaims, CryptoError> {
        let parts: Vec<&str> = token.split('.').collect();
        if parts.len() != 3 {
            return Err(CryptoError::InvalidFormat);
        }
        
        // Verify signature
        let header_and_payload = format!("{}.{}", parts[0], parts[1]);
        let signature = hex::decode(parts[2]).map_err(|_| CryptoError::InvalidFormat)?;
        
        if !HmacSha256::new(&self.secret).verify(header_and_payload.as_bytes(), &signature) {
            return Err(CryptoError::InvalidSignature);
        }
        
        // Parse payload (simplified)
        let payload = parts[1];
        // In real implementation, properly parse JSON
        Ok(JwtClaims {
            sub: "user123".to_string(), // Simplified
            exp: 1234567890,
            iat: 1234567800,
            role: "user".to_string(),
        })
    }
}

#[derive(Debug, Clone)]
pub struct ApiKeyInfo {
    pub user_id: String,
    pub permissions: Vec<String>,
    pub created_at: u64,
    pub expires_at: Option<u64>,
}

pub struct ApiKeyManager {
    keys: HashMap<String, ApiKeyInfo>,
}

impl ApiKeyManager {
    pub fn new() -> Self {
        ApiKeyManager {
            keys: HashMap::new(),
        }
    }
    
    pub fn generate_key(&mut self, user_id: &str, permissions: Vec<String>) -> String {
        let key = self.generate_secure_key();
        let info = ApiKeyInfo {
            user_id: user_id.to_string(),
            permissions,
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            expires_at: None,
        };
        
        self.keys.insert(key.clone(), info);
        key
    }
    
    pub fn validate_key(&self, key: &str, required_permission: &str) -> Option<&ApiKeyInfo> {
        if let Some(info) = self.keys.get(key) {
            // Check if key is expired
            if let Some(expires_at) = info.expires_at {
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs();
                if now > expires_at {
                    return None;
                }
            }
            
            // Check permissions
            if info.permissions.contains(&required_permission.to_string()) {
                return Some(info);
            }
        }
        
        None
    }
    
    fn generate_secure_key(&self) -> String {
        let mut rng = SecureRng::new();
        let mut bytes = [0u8; 32];
        rng.fill_bytes(&mut bytes);
        hex::encode(&bytes)
    }
}

// =========================================
// INPUT VALIDATION
// =========================================

#[derive(Debug)]
pub enum ValidationError {
    TooLong(usize),
    TooShort(usize),
    InvalidCharacter(char),
    PatternMatch(String),
    Empty,
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ValidationError::TooLong(max) => write!(f, "Input exceeds maximum length of {}", max),
            ValidationError::TooShort(min) => write!(f, "Input below minimum length of {}", min),
            ValidationError::InvalidCharacter(ch) => write!(f, "Invalid character: {}", ch),
            ValidationError::PatternMatch(pattern) => write!(f, "Input matches forbidden pattern: {}", pattern),
            ValidationError::Empty => write!(f, "Input cannot be empty"),
        }
    }
}

impl Error for ValidationError {}

pub struct InputValidator {
    allowed_chars: HashSet<char>,
    min_length: usize,
    max_length: usize,
    patterns: Vec<String>,
}

impl InputValidator {
    pub fn new(min_length: usize, max_length: usize) -> Self {
        InputValidator {
            allowed_chars: HashSet::new(),
            min_length,
            max_length,
            patterns: Vec::new(),
        }
    }
    
    pub fn with_allowed_chars(mut self, chars: &str) -> Self {
        self.allowed_chars = chars.chars().collect();
        self
    }
    
    pub fn with_patterns(mut self, patterns: Vec<String>) -> Self {
        self.patterns = patterns;
        self
    }
    
    pub fn validate(&self, input: &str) -> Result<String, ValidationError> {
        // Check empty
        if input.is_empty() {
            return Err(ValidationError::Empty);
        }
        
        // Check length
        if input.len() < self.min_length {
            return Err(ValidationError::TooShort(self.min_length));
        }
        
        if input.len() > self.max_length {
            return Err(ValidationError::TooLong(self.max_length));
        }
        
        // Check allowed characters
        if !self.allowed_chars.is_empty() {
            for ch in input.chars() {
                if !self.allowed_chars.contains(&ch) {
                    return Err(ValidationError::InvalidCharacter(ch));
                }
            }
        }
        
        // Check patterns (for SQL injection, XSS, etc.)
        for pattern in &self.patterns {
            if input.to_lowercase().contains(&pattern.to_lowercase()) {
                return Err(ValidationError::PatternMatch(pattern.clone()));
            }
        }
        
        Ok(input.to_string())
    }
}

// =========================================
// SECURE CONFIGURATION
// =========================================

#[derive(Debug, Clone)]
pub struct SecurityConfig {
    pub jwt_secret: String,
    pub database_url: String,
    pub api_keys: Vec<String>,
    pub allowed_origins: Vec<String>,
    pub max_request_size: usize,
    pub rate_limit: u32,
}

impl SecurityConfig {
    pub fn load_from_env() -> Self {
        SecurityConfig {
            jwt_secret: std::env::var("JWT_SECRET").unwrap_or_else(|_| "default-secret-change-in-production".to_string()),
            database_url: std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite://app.db".to_string()),
            api_keys: std::env::var("API_KEYS")
                .unwrap_or_else(|_| "".to_string())
                .split(',')
                .map(|s| s.trim().to_string())
                .collect(),
            allowed_origins: std::env::var("ALLOWED_ORIGINS")
                .unwrap_or_else(|_| "http://localhost:3000".to_string())
                .split(',')
                .map(|s| s.trim().to_string())
                .collect(),
            max_request_size: std::env::var("MAX_REQUEST_SIZE")
                .unwrap_or_else(|_| "1048576".to_string())
                .parse()
                .unwrap_or(1024 * 1024),
            rate_limit: std::env::var("RATE_LIMIT")
                .unwrap_or_else(|_| "100".to_string())
                .parse()
                .unwrap_or(100),
        }
    }
    
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();
        
        if self.jwt_secret.len() < 32 {
            errors.push("JWT secret should be at least 32 characters".to_string());
        }
        
        if self.jwt_secret == "default-secret-change-in-production" {
            errors.push("JWT secret should be changed in production".to_string());
        }
        
        if self.max_request_size > 10 * 1024 * 1024 {
            errors.push("Maximum request size seems too large".to_string());
        }
        
        if self.rate_limit > 10000 {
            errors.push("Rate limit seems too high".to_string());
        }
        
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

// =========================================
// ERROR TYPES
// =========================================

#[derive(Debug)]
pub enum CryptoError {
    InvalidFormat,
    InvalidSignature,
    InvalidKey,
    EncryptionFailed,
    DecryptionFailed,
}

impl fmt::Display for CryptoError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CryptoError::InvalidFormat => write!(f, "Invalid format"),
            CryptoError::InvalidSignature => write!(f, "Invalid signature"),
            CryptoError::InvalidKey => write!(f, "Invalid key"),
            CryptoError::EncryptionFailed => write!(f, "Encryption failed"),
            CryptoError::DecryptionFailed => write!(f, "Decryption failed"),
        }
    }
}

impl Error for CryptoError {}

// =========================================
// DEMONSTRATION FUNCTIONS
// =========================================

pub fn demonstrate_hashing() {
    println!("=== HASHING DEMONSTRATION ===");
    
    let data = b"Hello, secure world!";
    let hash = Sha256Hasher::hash(data);
    println!("SHA-256: {}", hex::encode(&hash));
    
    // Verify hash
    let is_valid = Sha256Hasher::hash(data) == hash;
    println!("Hash verification: {}", is_valid);
    
    // HMAC
    let key = b"super-secret-key";
    let message = b"Authenticated message";
    let hmac = HmacSha256::new(key).sign(message);
    println!("HMAC: {}", hex::encode(&hmac));
    
    let is_hmac_valid = HmacSha256::new(key).verify(message, &hmac);
    println!("HMAC verification: {}", is_hmac_valid);
    
    println!();
}

pub fn demonstrate_encryption() {
    println!("=== ENCRYPTION DEMONSTRATION ===");
    
    let key = [0u8; 32]; // In production, use a proper key
    let plaintext = b"Secret message to encrypt";
    
    let aes = Aes256Gcm::new(key);
    
    match aes.encrypt(plaintext) {
        Ok((ciphertext, nonce)) => {
            println!("Ciphertext: {}", hex::encode(&ciphertext));
            println!("Nonce: {}", hex::encode(&nonce));
            
            match aes.decrypt(&ciphertext, &nonce) {
                Ok(decrypted) => {
                    println!("Decrypted: {}", String::from_utf8_lossy(&decrypted));
                    assert_eq!(decrypted, plaintext);
                }
                Err(e) => println!("Decryption failed: {}", e),
            }
        }
        Err(e) => println!("Encryption failed: {}", e),
    }
    
    println!();
}

pub fn demonstrate_signatures() {
    println!("=== DIGITAL SIGNATURES DEMONSTRATION ===");
    
    let keypair = Ed25519Keypair::generate();
    println!("Public key: {}", hex::encode(&keypair.public_key));
    
    let message = b"Important message to sign";
    let signature = keypair.sign(message);
    println!("Signature: {}", hex::encode(&signature));
    
    let is_valid = keypair.verify(message, &signature);
    println!("Signature verification: {}", is_valid);
    
    // Tampered message should fail
    let tampered_message = b"Tampered message";
    let is_tampered_valid = keypair.verify(tampered_message, &signature);
    println!("Tampered message verification: {}", is_tampered_valid);
    
    println!();
}

pub fn demonstrate_password_hashing() {
    println!("=== PASSWORD HASHING DEMONSTRATION ===");
    
    let password = "my-secure-password";
    let salt = b"random-salt-value";
    
    let hasher = Argon2Hasher::new();
    match hasher.hash_password(password, salt) {
        Ok(hash) => {
            println!("Password hash: {}", hash);
            
            match hasher.verify_password(password, &hash) {
                Ok(is_valid) => {
                    println!("Password verification: {}", is_valid);
                    assert!(is_valid);
                }
                Err(e) => println!("Verification error: {}", e),
            }
        }
        Err(e) => println!("Hashing error: {}", e),
    }
    
    println!();
}

pub fn demonstrate_jwt() {
    println!("=== JWT DEMONSTRATION ===");
    
    let jwt_manager = JwtManager::new("your-256-bit-secret");
    
    let claims = JwtClaims {
        sub: "user123".to_string(),
        exp: 1234567890,
        iat: 1234567800,
        role: "admin".to_string(),
    };
    
    match jwt_manager.create_token(&claims) {
        Ok(token) => {
            println!("JWT Token: {}", token);
            
            match jwt_manager.verify_token(&token) {
                Ok(verified_claims) => {
                    println!("Verified claims: {:?}", verified_claims);
                }
                Err(e) => println!("JWT verification failed: {}", e),
            }
        }
        Err(e) => println!("JWT creation failed: {}", e),
    }
    
    println!();
}

pub fn demonstrate_api_keys() {
    println!("=== API KEY DEMONSTRATION ===");
    
    let mut key_manager = ApiKeyManager::new();
    
    let api_key = key_manager.generate_key("user123", vec!["read".to_string(), "write".to_string()]);
    println!("Generated API Key: {}", api_key);
    
    match key_manager.validate_key(&api_key, "read") {
        Some(info) => {
            println!("Access granted for user: {}", info.user_id);
            println!("Permissions: {:?}", info.permissions);
        }
        None => println!("Access denied"),
    }
    
    // Test invalid permission
    match key_manager.validate_key(&api_key, "admin") {
        Some(_) => println!("Unexpected access granted"),
        None => println!("Access denied for admin permission"),
    }
    
    println!();
}

pub fn demonstrate_input_validation() {
    println!("=== INPUT VALIDATION DEMONSTRATION ===");
    
    let validator = InputValidator::new(3, 100)
        .with_allowed_chars("abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789 .,!?")
        .with_patterns(vec![
            "union".to_string(),
            "select".to_string(),
            "drop".to_string(),
            "<script".to_string(),
        ]);
    
    let test_inputs = vec![
        "Hello, world!",
        "A",
        "This is a very long input that exceeds the maximum length limit",
        "Hello <script>alert('xss')</script>",
        "SELECT * FROM users",
    ];
    
    for input in test_inputs {
        match validator.validate(input) {
            Ok(validated) => println!("✓ '{}'", validated),
            Err(e) => println!("✗ '{}': {}", input, e),
        }
    }
    
    println!();
}

pub fn demonstrate_secure_rng() {
    println!("=== SECURE RNG DEMONSTRATION ===");
    
    let mut rng = SecureRng::new();
    
    let mut bytes = [0u8; 32];
    rng.fill_bytes(&mut bytes);
    println!("Secure random bytes: {}", hex::encode(&bytes));
    
    let number = rng.gen_range(1000, 9999);
    println!("Secure random number: {}", number);
    
    // Deterministic RNG with seed
    let mut deterministic_rng = SecureRng::with_seed(42);
    let deterministic_number = deterministic_rng.gen_range(1000, 9999);
    println!("Deterministic number (seed=42): {}", deterministic_number);
    
    println!();
}

pub fn demonstrate_security_config() {
    println!("=== SECURITY CONFIGURATION DEMONSTRATION ===");
    
    let config = SecurityConfig::load_from_env();
    println!("Security config: {:?}", config);
    
    match config.validate() {
        Ok(()) => println!("✓ Security configuration is valid"),
        Err(errors) => {
            println!("✗ Security configuration errors:");
            for error in errors {
                println!("  - {}", error);
            }
        }
    }
    
    println!();
}

// =========================================
// MAIN DEMONSTRATION
// =========================================

fn main() {
    println!("=== SECURITY AND CRYPTOGRAPHY DEMONSTRATIONS ===\n");
    
    demonstrate_hashing();
    demonstrate_encryption();
    demonstrate_signatures();
    demonstrate_password_hashing();
    demonstrate_jwt();
    demonstrate_api_keys();
    demonstrate_input_validation();
    demonstrate_secure_rng();
    demonstrate_security_config();
    
    println!("=== SECURITY AND CRYPTOGRAPHY DEMONSTRATIONS COMPLETE ===");
    println!("Note: This uses simulated cryptographic functions. Real implementations should:");
    println!("- Use established cryptographic libraries like ring or rustcrypto");
    println!("- Follow proper key management practices");
    println!("- Use hardware security modules when available");
    println!("- Implement proper random number generation");
    println!("- Follow security best practices and guidelines");
}

// =========================================
// UNIT TESTS
// =========================================

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_sha256_hash() {
        let data = b"Hello, world!";
        let hash1 = Sha256Hasher::hash(data);
        let hash2 = Sha256Hasher::hash(data);
        
        assert_eq!(hash1, hash2);
        assert_eq!(hash1.len(), 32);
    }
    
    #[test]
    fn test_hmac() {
        let key = b"test-key";
        let message = b"test-message";
        let hmac = HmacSha256::new(key).sign(message);
        
        assert!(HmacSha256::new(key).verify(message, &hmac));
        assert!(!HmacSha256::new(key).verify(b"different-message", &hmac));
    }
    
    #[test]
    fn test_aes_encryption() {
        let key = [0u8; 32];
        let plaintext = b"test message";
        let aes = Aes256Gcm::new(key);
        
        let (ciphertext, nonce) = aes.encrypt(plaintext).unwrap();
        let decrypted = aes.decrypt(&ciphertext, &nonce).unwrap();
        
        assert_eq!(decrypted, plaintext);
    }
    
    #[test]
    fn test_ed25519_signatures() {
        let keypair = Ed25519Keypair::generate();
        let message = b"test message";
        let signature = keypair.sign(message);
        
        assert!(keypair.verify(message, &signature));
        assert!(!keypair.verify(b"different message", &signature));
    }
    
    #[test]
    fn test_password_hashing() {
        let password = "test-password";
        let salt = b"test-salt";
        let hasher = Argon2Hasher::new();
        
        let hash = hasher.hash_password(password, salt).unwrap();
        let is_valid = hasher.verify_password(password, &hash).unwrap();
        
        assert!(is_valid);
        assert!(!hasher.verify_password("wrong-password", &hash).unwrap());
    }
    
    #[test]
    fn test_jwt() {
        let jwt_manager = JwtManager::new("test-secret");
        let claims = JwtClaims {
            sub: "user123".to_string(),
            exp: 1234567890,
            iat: 1234567800,
            role: "user".to_string(),
        };
        
        let token = jwt_manager.create_token(&claims).unwrap();
        let verified_claims = jwt_manager.verify_token(&token).unwrap();
        
        assert_eq!(verified_claims.sub, claims.sub);
        assert_eq!(verified_claims.role, claims.role);
    }
    
    #[test]
    fn test_api_key_manager() {
        let mut manager = ApiKeyManager::new();
        let key = manager.generate_key("user123", vec!["read".to_string()]);
        
        assert!(manager.validate_key(&key, "read").is_some());
        assert!(manager.validate_key(&key, "write").is_none());
        assert!(manager.validate_key("invalid-key", "read").is_none());
    }
    
    #[test]
    fn test_input_validation() {
        let validator = InputValidator::new(3, 10)
            .with_allowed_chars("abc");
        
        assert!(validator.validate("abc").is_ok());
        assert!(validator.validate("ab").is_err()); // Too short
        assert!(validator.validate("abcd").is_err()); // Too long
        assert!(validator.validate("abd").is_err()); // Invalid character
    }
    
    #[test]
    fn test_secure_rng() {
        let mut rng1 = SecureRng::with_seed(42);
        let mut rng2 = SecureRng::with_seed(42);
        
        let num1 = rng1.gen_range(0, 1000);
        let num2 = rng2.gen_range(0, 1000);
        
        assert_eq!(num1, num2); // Same seed should produce same number
        
        let mut rng3 = SecureRng::new();
        let num3 = rng3.gen_range(0, 1000);
        let num4 = rng3.gen_range(0, 1000);
        
        assert_ne!(num3, num4); // Different calls should produce different numbers
    }
    
    #[test]
    fn test_security_config() {
        let config = SecurityConfig {
            jwt_secret: "weak".to_string(),
            database_url: "sqlite://test.db".to_string(),
            api_keys: vec![],
            allowed_origins: vec!["http://localhost:3000".to_string()],
            max_request_size: 1024,
            rate_limit: 100,
        };
        
        let errors = config.validate().unwrap_err();
        assert!(!errors.is_empty()); // Should have validation errors
    }
}
