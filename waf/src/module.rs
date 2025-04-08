//waf/src/module.rs
#![allow(non_snake_case)]

use std::collections::HashMap;
use std::sync::Mutex;
use once_cell::sync::Lazy;

#[derive(Clone)]
pub struct ModuleInfo {
    pub Name: String,
    pub Version: String,
    pub ProcessContent: fn(&mut String),
}

static _MODULE_REGISTRY: Lazy<Mutex<HashMap<String, ModuleInfo>>> = Lazy::new(|| 
    Mutex::new(HashMap::with_capacity(16))
);

pub fn register_module(Info: ModuleInfo) {
    let _ModuleIdentifier = Info.Name.clone();
    
    if let Ok(mut ModuleMap) = _MODULE_REGISTRY.lock() {
        ModuleMap.insert(_ModuleIdentifier.clone(), Info);
    }
}

pub fn process_content(Content: &mut String) {
    if let Ok(ModuleMap) = _MODULE_REGISTRY.lock() {
        for (_ModuleKey, ModuleData) in ModuleMap.iter() {
            (ModuleData.ProcessContent)(Content);
        }
    }
}

pub fn print_modules_performance_report() {
    if let Ok(_ModuleMap) = _MODULE_REGISTRY.lock() {
        // Performance report functionality without logging
    }
}

pub fn get_registered_module_count() -> usize {
    if let Ok(ModuleMap) = _MODULE_REGISTRY.lock() {
        return ModuleMap.len();
    }
    0
}