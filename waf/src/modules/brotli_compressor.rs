//waf/src/modules/brotli_compressor.rs
#![allow(non_snake_case)]

use brotli::CompressorWriter;
use std::io::Write;
use bytes::Bytes;

use crate::module::{ModuleInfo, register_module, process_content};
use hyper::{HeaderMap, header};

pub const MODULE_NAME: &str = "BrotliCompressor";
pub const MODULE_VERSION: &str = "1.0.0";

thread_local! {
    static _COMPRESSED_CONTENT: std::cell::RefCell<Option<Vec<u8>>> = std::cell::RefCell::new(None);
    static _ORIGINAL_CONTENT_TYPE: std::cell::RefCell<Option<String>> = std::cell::RefCell::new(None);
}

pub struct BrotliCompressor {
    _CompressionLevel: u32,
}

impl BrotliCompressor {
    pub fn new(CompressionLevel: u32) -> Self {
        let Level = if CompressionLevel > 11 { 11 } else { CompressionLevel };
        BrotliCompressor {
            _CompressionLevel: Level,
        }
    }

    pub fn compress(&self, Content: &mut String, ContentType: Option<&str>) -> Vec<u8> {
        if let Some(OriginalType) = ContentType {
            _ORIGINAL_CONTENT_TYPE.with(|Cell| {
                *Cell.borrow_mut() = Some(OriginalType.to_string());
            });
        }
        
        let OriginalBytes = Content.as_bytes();
        let mut CompressedBuffer = Vec::new();
        
        {
            let mut Compressor = CompressorWriter::new(
                &mut CompressedBuffer, 
                4096, 
                self._CompressionLevel, 
                22
            );
            
            if let Err(_) = Compressor.write_all(OriginalBytes) {
                return Vec::new();
            }
            
            if let Err(_) = Compressor.flush() {
                return Vec::new();
            }
        }
        
        CompressedBuffer
    }
    
    pub fn compress_bytes(&self, InputBytes: &[u8], ContentType: Option<&str>) -> Vec<u8> {
        if let Some(OriginalType) = ContentType {
            _ORIGINAL_CONTENT_TYPE.with(|Cell| {
                *Cell.borrow_mut() = Some(OriginalType.to_string());
            });
        }
        
        let mut CompressedBuffer = Vec::new();
        let mut Success = true;
        
        {
            let mut Compressor = CompressorWriter::new(
                &mut CompressedBuffer, 
                4096, 
                self._CompressionLevel, 
                22
            );
            
            if Compressor.write_all(InputBytes).is_err() {
                Success = false;
            } else if Compressor.flush().is_err() {
                Success = false;
            }
        }
        
        if !Success {
            Vec::new()
        } else {
            CompressedBuffer
        }
    }
}

pub fn is_compressible_content(ContentType: &str) -> bool {
    let LowerType = ContentType.to_lowercase();
    
    LowerType.contains("text/") || 
    LowerType.contains("application/json") || 
    LowerType.contains("application/javascript") || 
    LowerType.contains("application/xml") || 
    LowerType.contains("application/xhtml+xml") ||
    LowerType.contains("image/svg+xml") ||
    LowerType.contains("application/x-www-form-urlencoded")
}

pub fn compress_content(Content: &mut String) -> Bytes {
    let Compressor = BrotliCompressor::new(4);
    let CompressedData = Compressor.compress(Content, None);
    Bytes::from(CompressedData)
}

pub fn compress_content_with_type(Content: &mut String, ContentType: &str) -> Bytes {
    let Compressor = BrotliCompressor::new(4);
    let CompressedData = Compressor.compress(Content, Some(ContentType));
    Bytes::from(CompressedData)
}

pub fn compress_bytes_with_type(Data: &[u8], ContentType: &str) -> Bytes {
    if let Ok(TextContent) = std::str::from_utf8(Data) {
        let mut ContentString = TextContent.to_string();
        
        process_content(&mut ContentString);
        
        if !is_compressible_content(ContentType) {
            return Bytes::from(ContentString);
        }
        
        let Compressor = BrotliCompressor::new(4);
        let CompressedData = Compressor.compress(&mut ContentString, Some(ContentType));
        return Bytes::from(CompressedData);
    } else {
        if !is_compressible_content(ContentType) {
            return Bytes::from(Data.to_vec());
        }
        
        let Compressor = BrotliCompressor::new(4);
        let CompressedData = Compressor.compress_bytes(Data, Some(ContentType));
        Bytes::from(CompressedData)
    }
}

pub fn register() {
    register_module(ModuleInfo {
        Name: MODULE_NAME.to_string(),
        Version: MODULE_VERSION.to_string(),
        ProcessContent: process_content_brotli,
    });
}

fn process_content_brotli(Content: &mut String) {
    let CompressedBytes = compress_content(Content);
    _COMPRESSED_CONTENT.with(|Cell| {
        *Cell.borrow_mut() = Some(CompressedBytes.to_vec());
    });
}

pub fn get_compressed_content() -> Option<Bytes> {
    _COMPRESSED_CONTENT.with(|Cell| {
        Cell.borrow_mut().take().map(Bytes::from)
    })
}

pub fn get_original_content_type() -> Option<String> {
    _ORIGINAL_CONTENT_TYPE.with(|Cell| {
        Cell.borrow().clone()
    })
}

pub fn update_headers_for_brotli(Headers: &mut HeaderMap) -> bool {
    let HasCompressedContent = _ORIGINAL_CONTENT_TYPE.with(|Cell| Cell.borrow().is_some());
    
    if HasCompressedContent {
        Headers.insert(
            header::CONTENT_ENCODING,
            header::HeaderValue::from_static("br")
        );
        
        Headers.insert(
            header::VARY,
            header::HeaderValue::from_static("Accept-Encoding")
        );
        
        if let Some(OriginalType) = get_original_content_type() {
            if let Ok(TypeValue) = header::HeaderValue::from_str(&OriginalType) {
                Headers.insert(header::CONTENT_TYPE, TypeValue);
            }
        }
    }
    
    HasCompressedContent
}
