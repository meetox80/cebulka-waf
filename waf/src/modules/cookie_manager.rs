//waf/src/modules/cookie_manager.rs
#![allow(non_snake_case)]

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use lazy_static::lazy_static;
use sha2::{Sha256, Digest};
use uuid::Uuid;

lazy_static! {
    static ref COOKIE_STORAGE: Arc<Mutex<CookieStorage>> = Arc::new(Mutex::new(CookieStorage::new()));
}

pub struct CookieStorage {
    Cookies: HashMap<String, Instant>,
    ExpirationTime: Duration,
}

impl CookieStorage {
    fn new() -> Self {
        Self {
            Cookies: HashMap::new(),
            ExpirationTime: Duration::from_secs(600), // 10 minutes
        }
    }

    fn add_cookie(&mut self, Cookie: String) {
        self.Cookies.insert(Cookie, Instant::now());
    }

    fn is_valid(&mut self, Cookie: &str) -> bool {
        if let Some(Created) = self.Cookies.get(Cookie) {
            if Created.elapsed() < self.ExpirationTime {
                return true;
            }
        }
        false
    }

    fn cleanup_expired(&mut self) {
        let Now = Instant::now();
        self.Cookies.retain(|_, Created| {
            Now.duration_since(*Created) < self.ExpirationTime
        });
    }
}

pub fn generate_cookie() -> String {
    let _Uuid = Uuid::new_v4().to_string();
    let mut _Hasher = Sha256::new();
    _Hasher.update(_Uuid);
    let _Result = _Hasher.finalize();
    format!("{:x}", _Result)
}

pub fn validate_cookie(Cookie: &str) -> bool {
    let mut _Storage = COOKIE_STORAGE.lock().unwrap();
    _Storage.cleanup_expired();
    _Storage.is_valid(Cookie)
}

pub fn store_cookie(Cookie: String) {
    let mut _Storage = COOKIE_STORAGE.lock().unwrap();
    _Storage.add_cookie(Cookie);
}

pub fn is_valid_format(Cookie: &str) -> bool {
    Cookie.len() == 64 && Cookie.chars().all(|c| c.is_ascii_hexdigit())
}

pub fn get_active_user_count() -> usize {
    let mut _Storage = COOKIE_STORAGE.lock().unwrap();
    _Storage.cleanup_expired();
    _Storage.Cookies.len()
}

pub fn register() {
    tokio::spawn(async {
        let _CleanupInterval = Duration::from_secs(60);
        loop {
            tokio::time::sleep(_CleanupInterval).await;
            let mut _Storage = COOKIE_STORAGE.lock().unwrap();
            _Storage.cleanup_expired();
        }
    });
} 