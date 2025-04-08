//waf/src/modules/ipv6_detector.rs
#![allow(non_snake_case)]

use regex::Regex;
use std::sync::OnceLock;

use crate::module::{ModuleInfo, register_module};

pub const MODULE_NAME: &str = "IPv6Detector";
pub const MODULE_VERSION: &str = "1.0.0";

pub struct IPv6Detector {
    _Pattern: Regex,
}

fn get_ipv6_regex() -> &'static Regex {
    static _IPV6_PATTERN: OnceLock<Regex> = OnceLock::new();
    _IPV6_PATTERN.get_or_init(|| {
        Regex::new(r"(?i)(([0-9a-f]{1,4}:){7}[0-9a-f]{1,4}|::([0-9a-f]{1,4}:){0,6}[0-9a-f]{1,4}|([0-9a-f]{1,4}:){1,6}:[0-9a-f]{1,4}|([0-9a-f]{1,4}:){1,5}(:[0-9a-f]{1,4}){1,2}|([0-9a-f]{1,4}:){1,4}(:[0-9a-f]{1,4}){1,3}|([0-9a-f]{1,4}:){1,3}(:[0-9a-f]{1,4}){1,4}|([0-9a-f]{1,4}:){1,2}(:[0-9a-f]{1,4}){1,5}|[0-9a-f]{1,4}:((:[0-9a-f]{1,4}){1,6})|:((:[0-9a-f]{1,4}){1,7}|:))").unwrap()
    })
}

impl IPv6Detector {
    pub fn new() -> Self {
        IPv6Detector {
            _Pattern: get_ipv6_regex().clone(),
        }
    }

    pub fn redact_ipv6(&self, Content: &mut String) {
        *Content = self._Pattern.replace_all(Content, "[REDACTED]").to_string();
    }
}

pub fn redact_ipv6_in_content(Content: &mut String) {
    let Detector = IPv6Detector::new();
    Detector.redact_ipv6(Content);
}

pub fn register() {
    register_module(ModuleInfo {
        Name: MODULE_NAME.to_string(),
        Version: MODULE_VERSION.to_string(),
        ProcessContent: process_content,
    });
}

fn process_content(Content: &mut String) {
    redact_ipv6_in_content(Content);
} 