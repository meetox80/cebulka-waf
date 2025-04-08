//waf/src/modules/mod.rs
#![allow(non_snake_case)]

pub mod ipv4_detector;
pub mod ipv6_detector;
pub mod brotli_compressor;
pub mod cookie_manager;
pub mod dashboard;

pub fn init_all() {
    ipv4_detector::register();
    ipv6_detector::register();
    brotli_compressor::register();
    cookie_manager::register();
    dashboard::register();
} 