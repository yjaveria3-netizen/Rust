# Security and Cryptography in Rust

## Overview

Rust's memory safety and type system make it an excellent choice for security-critical applications. This guide covers cryptographic operations, secure coding practices, authentication, authorization, and building secure systems in Rust.

---

## Cryptographic Foundations

### Core Cryptography Crates

| Crate | Purpose | Features |
|-------|---------|----------|
| `ring` | Cryptographic primitives | Hashing, HMAC, AEAD, signatures |
| `rustcrypto` | Crypto algorithms collection | Comprehensive algorithm support |
| `argon2` | Password hashing | Memory-hard password hashing |
| `rsa` | RSA cryptography | Key generation, encryption, signatures |
| `ed25519` | Ed25519 signatures | Modern elliptic curve cryptography |
| `aes-gcm` | AES-GCM encryption | Authenticated encryption |
| `sha2` | SHA-2 hashing | SHA-256, SHA-512 |
| `hmac` | HMAC implementation | Message authentication |

### Choosing the Right Crypto Library

- **ring** - Modern, safe, and well-maintained
- **rustcrypto** - Comprehensive, modular approach
- **argon2** - Best for password hashing
- **ed25519** - Modern signatures with better security

---

## Hashing and Message Authentication

### SHA-256 Hashing

```rust
use sha2::{Sha256, Digest};
use hex;

fn sha256_hash(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    let result = hasher.finalize();
    hex::encode(result)
}

fn verify_sha256(data: &[u8], expected_hash: &str) -> bool {
    let computed_hash = sha256_hash(data);
    computed_hash == expected_hash
}

// Usage
let data = b"Hello, secure world!";
let hash = sha256_hash(data);
println!("SHA-256: {}", hash);

let is_valid = verify_sha256(data, &hash);
assert!(is_valid);
```

### HMAC for Message Authentication

```rust
use hmac::{Hmac, Mac};
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

fn create_hmac(key: &[u8], data: &[u8]) -> Vec<u8> {
    let mut mac = HmacSha256::new_from_slice(key).expect("HMAC can take key of any size");
    mac.update(data);
    mac.finalize().into_bytes().to_vec()
}

fn verify_hmac(key: &[u8], data: &[u8], mac: &[u8]) -> bool {
    match HmacSha256::new_from_slice(key) {
        Ok(mut hmac) => {
            hmac.update(data);
            hmac.verify(mac).is_ok()
        }
        Err(_) => false,
    }
}

// Usage
let key = b"super-secret-key";
let message = b"Authenticated message";
let mac = create_hmac(key, message);
println!("HMAC: {}", hex::encode(&mac));

let is_valid = verify_hmac(key, message, &mac);
assert!(is_valid);
```

### Password Hashing with Argon2

```rust
use argon2::{self, Config, ThreadMode, Variant, Version};

fn hash_password(password: &str, salt: &[u8]) -> Result<String, argon2::Error> {
    let config = Config {
        variant: Variant::Argon2id,
        version: Version::Version13,
        mem_cost: 65536,
        time_cost: 3,
        lanes: 4,
        thread_mode: ThreadMode::Parallel,
        secret: &[],
        ad: &[],
        hash_length: 32,
    };
    
    argon2::hash_encoded(password.as_bytes(), salt, &config)
}

fn verify_password(password: &str, hash: &str) -> Result<bool, argon2::Error> {
    argon2::verify_encoded(hash, password.as_bytes())
}

// Usage
let password = "my-secure-password";
let salt = b"random-salt-value";
let hash = hash_password(password, salt)?;

println!("Password hash: {}", hash);

let is_valid = verify_password(password, &hash)?;
assert!(is_valid);
```

---

## Symmetric Encryption

### AES-GCM Encryption

```rust
use aes_gcm::{Aes256Gcm, Key, Nonce, aead::{Aead, NewAead}};
use rand::{RngCore, thread_rng};

fn encrypt_aes256_gcm(key: &[u8; 32], plaintext: &[u8]) -> Result<(Vec<u8>, Vec<u8>), aes_gcm::Error> {
    let cipher = Aes256Gcm::new(Key::from_slice(key));
    let mut nonce_bytes = [0u8; 12];
    thread_rng().fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);
    
    cipher.encrypt(nonce, plaintext)
        .map(|ciphertext| (ciphertext, nonce_bytes.to_vec()))
}

fn decrypt_aes256_gcm(key: &[u8; 32], nonce: &[u8], ciphertext: &[u8]) -> Result<Vec<u8>, aes_gcm::Error> {
    let cipher = Aes256Gcm::new(Key::from_slice(key));
    let nonce = Nonce::from_slice(nonce);
    
    cipher.decrypt(nonce, ciphertext)
}

// Usage
let key = [0u8; 32]; // In production, use a proper key
let plaintext = b"Secret message to encrypt";

let (ciphertext, nonce) = encrypt_aes256_gcm(&key, plaintext)?;
println!("Ciphertext: {}", hex::encode(&ciphertext));
println!("Nonce: {}", hex::encode(&nonce));

let decrypted = decrypt_aes256_gcm(&key, &nonce, &ciphertext)?;
assert_eq!(decrypted, plaintext);
```

### Key Derivation

```rust
use hkdf::Hkdf;
use sha2::Sha256;
use rand::{RngCore, thread_rng};

fn derive_keys(master_key: &[u8], salt: &[u8], info: &[u8]) -> (Vec<u8>, Vec<u8>) {
    let hk = Hkdf::<Sha256>::new(Some(salt), master_key);
    let mut okm = [0u8; 64]; // Two 32-byte keys
    
    hk.expand(info, &mut okm)
        .expect("HKDF should expand correctly");
    
    let key1 = okm[0..32].to_vec();
    let key2 = okm[32..64].to_vec();
    
    (key1, key2)
}

// Usage
let master_key = b"master-key-material";
let salt = b"salt-value";
let info = b"key-derivation-info";

let (encryption_key, mac_key) = derive_keys(master_key, salt, info);
println!("Encryption key: {}", hex::encode(&encryption_key));
println!("MAC key: {}", hex::encode(&mac_key));
```

---

## Asymmetric Cryptography

### Ed25519 Digital Signatures

```rust
use ed25519_dalek::{Keypair, PublicKey, Signature, Signer, Verifier};
use rand::thread_rng;

fn generate_keypair() -> Keypair {
    let mut csprng = thread_rng();
    Keypair::generate(&mut csprng)
}

fn sign_message(private_key: &Keypair, message: &[u8]) -> Signature {
    private_key.sign(message)
}

fn verify_signature(public_key: &PublicKey, message: &[u8], signature: &Signature) -> bool {
    public_key.verify(message, signature).is_ok()
}

// Usage
let keypair = generate_keypair();
let message = b"Important message to sign";

let signature = sign_message(&keypair, message);
println!("Signature: {}", hex::encode(signature.to_bytes()));

let is_valid = verify_signature(&keypair.public, message, &signature);
assert!(is_valid);

// Tampered message should fail verification
let tampered_message = b"Tampered message";
let is_tampered_valid = verify_signature(&keypair.public, tampered_message, &signature);
assert!(!is_tampered_valid);
```

### RSA Encryption and Signatures

```rust
use rsa::{RsaPrivateKey, RsaPublicKey, PaddingScheme, PublicKey};
use sha2::Sha256;
use rand::thread_rng;

fn generate_rsa_keypair() -> Result<(RsaPrivateKey, RsaPublicKey), rsa::errors::Error> {
    let mut rng = thread_rng();
    let bits = 2048;
    RsaPrivateKey::new(&mut rng, bits)
        .map(|private_key| {
            let public_key = private_key.to_public_key();
            (private_key, public_key)
        })
}

fn rsa_encrypt(public_key: &RsaPublicKey, data: &[u8]) -> Result<Vec<u8>, rsa::errors::Error> {
    let mut rng = thread_rng();
    let padding = PaddingScheme::new_oaep::<Sha256>();
    public_key.encrypt(&mut rng, padding, data)
}

fn rsa_decrypt(private_key: &RsaPrivateKey, ciphertext: &[u8]) -> Result<Vec<u8>, rsa::errors::Error> {
    let padding = PaddingScheme::new_oaep::<Sha256>();
    private_key.decrypt(padding, ciphertext)
}

// Usage
let (private_key, public_key) = generate_rsa_keypair()?;
let message = b"RSA encrypted message";

let ciphertext = rsa_encrypt(&public_key, message)?;
println!("RSA ciphertext: {}", hex::encode(&ciphertext));

let decrypted = rsa_decrypt(&private_key, &ciphertext)?;
assert_eq!(decrypted, message);
```

---

## Secure Random Number Generation

### Cryptographically Secure RNG

```rust
use rand::{RngCore, thread_rng, SeedableRng};
use rand_chacha::ChaCha20Rng;

fn generate_secure_bytes(size: usize) -> Vec<u8> {
    let mut rng = thread_rng();
    let mut bytes = vec![0u8; size];
    rng.fill_bytes(&mut bytes);
    bytes
}

fn generate_secure_number(min: u64, max: u64) -> u64 {
    let mut rng = thread_rng();
    rng.gen_range(min..max)
}

fn deterministic_secure_rng(seed: [u8; 32]) -> ChaCha20Rng {
    ChaCha20Rng::from_seed(seed)
}

// Usage
let secure_bytes = generate_secure_bytes(32);
println!("Secure random bytes: {}", hex::encode(&secure_bytes));

let secure_number = generate_secure_number(1000, 9999);
println!("Secure random number: {}", secure_number);

let seed = [0u8; 32];
let mut deterministic_rng = deterministic_secure_rng(seed);
let deterministic_number = deterministic_rng.gen_range(1000..9999);
println!("Deterministic number: {}", deterministic_number);
```

---

## Authentication and Authorization

### JWT Token Implementation

```rust
use serde::{Deserialize, Serialize};
use jsonwebtoken::{encode, decode, Header, Algorithm, Validation, EncodingKey, DecodingKey};

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    exp: usize,
    iat: usize,
    role: String,
}

fn create_jwt(user_id: &str, role: &str, secret: &str) -> Result<String, jsonwebtoken::errors::Error> {
    let now = chrono::Utc::now();
    let exp = now + chrono::Duration::hours(24);
    
    let claims = Claims {
        sub: user_id.to_string(),
        exp: exp.timestamp() as usize,
        iat: now.timestamp() as usize,
        role: role.to_string(),
    };
    
    encode(&Header::default(), &claims, &EncodingKey::from_secret(secret.as_ref()))
}

fn verify_jwt(token: &str, secret: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    let validation = Validation::new(Algorithm::HS256);
    let token_data = decode::<Claims>(token, &DecodingKey::from_secret(secret.as_ref()), &validation)?;
    Ok(token_data.claims)
}

// Usage
let secret = "your-256-bit-secret";
let user_id = "user123";
let role = "admin";

let token = create_jwt(user_id, role, secret)?;
println!("JWT Token: {}", token);

let claims = verify_jwt(&token, secret)?;
println!("User ID: {}", claims.sub);
println!("Role: {}", claims.role);
```

### API Key Authentication

```rust
use std::collections::HashMap;
use uuid::Uuid;

struct ApiKeyManager {
    keys: HashMap<String, ApiKeyInfo>,
}

#[derive(Debug, Clone)]
struct ApiKeyInfo {
    user_id: String,
    permissions: Vec<String>,
    created_at: chrono::DateTime<chrono::Utc>,
    expires_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl ApiKeyManager {
    fn new() -> Self {
        ApiKeyManager {
            keys: HashMap::new(),
        }
    }
    
    fn generate_key(&mut self, user_id: &str, permissions: Vec<String>) -> String {
        let key = Uuid::new_v4().to_string();
        let info = ApiKeyInfo {
            user_id: user_id.to_string(),
            permissions,
            created_at: chrono::Utc::now(),
            expires_at: None,
        };
        
        self.keys.insert(key.clone(), info);
        key
    }
    
    fn validate_key(&self, key: &str, required_permission: &str) -> Option<&ApiKeyInfo> {
        if let Some(info) = self.keys.get(key) {
            // Check if key is expired
            if let Some(expires_at) = info.expires_at {
                if chrono::Utc::now() > expires_at {
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
}

// Usage
let mut key_manager = ApiKeyManager::new();

let api_key = key_manager.generate_key("user123", vec!["read".to_string(), "write".to_string()]);
println!("Generated API Key: {}", api_key);

if let Some(info) = key_manager.validate_key(&api_key, "read") {
    println!("Access granted for user: {}", info.user_id);
} else {
    println!("Access denied");
}
```

---

## Secure Network Communication

### TLS Configuration

```rust
use rustls::{Certificate, PrivateKey, ServerConfig, ClientConfig};
use rustls_pemfile::{certs, pkcs8_private_keys};
use std::fs::File;
use std::io::BufReader;

fn load_server_config(cert_file: &str, key_file: &str) -> Result<ServerConfig, Box<dyn std::error::Error>> {
    let cert_file = File::open(cert_file)?;
    let mut cert_reader = BufReader::new(cert_file);
    let certs = certs(&mut cert_reader)?
        .into_iter()
        .map(Certificate)
        .collect();
    
    let key_file = File::open(key_file)?;
    let mut key_reader = BufReader::new(key_file);
    let keys = pkcs8_private_keys(&mut key_reader)?
        .into_iter()
        .map(PrivateKey)
        .collect();
    
    let config = ServerConfig::builder()
        .with_safe_defaults()
        .with_no_client_auth()
        .with_single_cert(certs, keys.remove(0))?;
    
    Ok(config)
}

fn load_client_config() -> Result<ClientConfig, Box<dyn std::error::Error>> {
    let config = ClientConfig::builder()
        .with_safe_defaults()
        .with_root_certificates(rustls_native_certs::load_native_certs()?)
        .with_no_client_auth();
    
    Ok(config)
}
```

### Secure WebSocket Server

```rust
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::{accept_async, tungstenite::Message};
use futures_util::{SinkExt, StreamExt};
use std::sync::Arc;
use tokio_rustls::{TlsAcceptor, server::TlsStream};

struct SecureWebSocketServer {
    acceptor: Arc<TlsAcceptor>,
}

impl SecureWebSocketServer {
    fn new(config: ServerConfig) -> Self {
        let acceptor = Arc::new(TlsAcceptor::from(Arc::new(config)));
        SecureWebSocketServer { acceptor }
    }
    
    async fn handle_connection(&self, stream: TcpStream) -> Result<(), Box<dyn std::error::Error>> {
        let tls_stream = self.acceptor.accept(stream).await?;
        let ws_stream = accept_async(tls_stream).await?;
        let (mut ws_sender, mut ws_receiver) = ws_stream.split();
        
        while let Some(msg) = ws_receiver.next().await {
            match msg? {
                Message::Text(text) => {
                    // Process secure message
                    let response = format!("Echo: {}", text);
                    ws_sender.send(Message::Text(response)).await?;
                }
                Message::Binary(data) => {
                    // Process secure binary data
                    println!("Received {} bytes securely", data.len());
                }
                Message::Close(_) => {
                    println!("Client disconnected");
                    break;
                }
                _ => {}
            }
        }
        
        Ok(())
    }
}
```

---

## Security Best Practices

### Input Validation and Sanitization

```rust
use regex::Regex;
use std::collections::HashSet;

struct InputValidator {
    allowed_chars: HashSet<char>,
    max_length: usize,
    patterns: Vec<Regex>,
}

impl InputValidator {
    fn new(max_length: usize) -> Self {
        InputValidator {
            allowed_chars: HashSet::new(),
            max_length,
            patterns: Vec::new(),
        }
    }
    
    fn with_allowed_chars(mut self, chars: &str) -> Self {
        self.allowed_chars = chars.chars().collect();
        self
    }
    
    fn with_patterns(mut self, patterns: Vec<Regex>) -> Self {
        self.patterns = patterns;
        self
    }
    
    fn validate(&self, input: &str) -> Result<String, ValidationError> {
        // Check length
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
            if pattern.is_match(input) {
                return Err(ValidationError::PatternMatch);
            }
        }
        
        Ok(input.to_string())
    }
}

#[derive(Debug)]
enum ValidationError {
    TooLong(usize),
    InvalidCharacter(char),
    PatternMatch,
}

// Usage
let validator = InputValidator::new(100)
    .with_allowed_chars("abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789 .,!?")
    .with_patterns(vec![
        Regex::new(r"(?i)(union|select|drop|insert|update|delete)").unwrap(), // SQL injection
        Regex::new(r"<script.*?>").unwrap(), // XSS
    ]);

let safe_input = validator.validate("Hello, world!")?;
println!("Validated input: {}", safe_input);
```

### Secure Configuration Management

```rust
use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Debug, Deserialize, Serialize)]
struct SecurityConfig {
    jwt_secret: String,
    database_url: String,
    api_keys: Vec<String>,
    allowed_origins: Vec<String>,
    max_request_size: usize,
    rate_limit: u32,
}

impl SecurityConfig {
    fn load_from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let config: SecurityConfig = toml::from_str(&content)?;
        Ok(config)
    }
    
    fn load_from_env() -> Self {
        SecurityConfig {
            jwt_secret: std::env::var("JWT_SECRET").unwrap_or_else(|_| "default-secret".to_string()),
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
}
```

---

## Key Takeaways

- **Memory safety** prevents many security vulnerabilities
- **Constant-time operations** prevent timing attacks
- **Proper key management** is essential for cryptographic security
- **Input validation** prevents injection attacks
- **TLS/SSL** protects network communications
- **Secure defaults** reduce attack surface
- **Regular updates** keep dependencies secure

---

## Security Best Practices

| Practice | Description | Implementation |
|----------|-------------|----------------|
| **Zero-knowledge proofs** | Prove without revealing | Use cryptographic protocols |
| **Secure key storage** | Protect cryptographic keys | Use hardware security modules |
| **Input validation** | Prevent injection attacks | Sanitize all user input |
| **Rate limiting** | Prevent abuse | Implement request throttling |
| **Audit logging** | Track security events | Log all access attempts |
| **Principle of least privilege** | Minimize permissions | Grant minimal necessary access |
| **Regular security audits** | Find vulnerabilities | Use security scanning tools |
