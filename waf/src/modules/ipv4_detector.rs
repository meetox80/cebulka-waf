//waf/src/modules/ipv4_detector.rs
#![allow(non_snake_case)]

use regex::Regex;
use std::sync::OnceLock;

use crate::module::{ModuleInfo, register_module};

pub const MODULE_NAME: &str = "IPv4Detector";
pub const MODULE_VERSION: &str = "1.0.0";

pub struct IPv4Detector {
    _Pattern: Regex,
}

fn get_ipv4_regex() -> &'static Regex {
    static _IPV4_PATTERN: OnceLock<Regex> = OnceLock::new();
    _IPV4_PATTERN.get_or_init(|| {
        Regex::new(r"(25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\.(25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\.(25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\.(25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)").unwrap()
    })
}

impl IPv4Detector {
    pub fn new() -> Self {
        IPv4Detector {
            _Pattern: get_ipv4_regex().clone(),
        }
    }

    pub fn redact_ipv4(&self, Content: &mut String) {
        *Content = self._Pattern.replace_all(Content, "[REDACTED]").to_string();
    }
}

pub fn redact_ipv4_in_content(Content: &mut String) {
    let Detector = IPv4Detector::new();
    Detector.redact_ipv4(Content);
}

pub fn register() {
    register_module(ModuleInfo {
        Name: MODULE_NAME.to_string(),
        Version: MODULE_VERSION.to_string(),
        ProcessContent: process_content,
    });
}

fn process_content(Content: &mut String) {
    redact_ipv4_in_content(Content);
} 