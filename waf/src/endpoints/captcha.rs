//waf/src/endpoints/captcha.rs
#![allow(non_snake_case)]

use hyper::{Request, Response, StatusCode};
use hyper::body::Incoming;
use hyper::header::{SET_COOKIE, COOKIE};
use http_body_util::Full;
use http_body_util::combinators::BoxBody;
use hyper::body::Bytes;
use std::convert::Infallible;

use crate::modules::cookie_manager::{generate_cookie, validate_cookie, store_cookie, is_valid_format};
use crate::modules::brotli_compressor::{compress_bytes_with_type, update_headers_for_brotli};

type ResponseBody = BoxBody<Bytes, Infallible>;

#[derive(Clone)]
pub struct CaptchaEndpoint;

impl CaptchaEndpoint {
    pub fn new() -> Self {
        Self
    }
    
    pub fn extract_cookie(&self, Request: &Request<Incoming>) -> Option<String> {
        let _Headers = Request.headers();
        _Headers.get(COOKIE).and_then(|cookie_value| {
            let _CookieStr = cookie_value.to_str().ok()?;
            _CookieStr.split(';')
                .map(|s| s.trim())
                .find(|s| s.starts_with("access="))
                .map(|s| s[7..].to_string())
        })
    }
    
    pub fn create_captcha_response(&self) -> Response<ResponseBody> {
        let _NewCookie = generate_cookie();
        store_cookie(_NewCookie.clone());
        
        let _ContentType = "text/html; charset=utf-8";
        let _HtmlContent = "<html><body><h1>captcha 192.168.0.1</h1></body></html>";
        let _CompressedBytes = compress_bytes_with_type(_HtmlContent.as_bytes(), _ContentType);
        
        let mut _Response = Response::builder()
            .status(StatusCode::OK)
            .header(SET_COOKIE, format!("access={}; Max-Age=600; Path=/; HttpOnly", _NewCookie))
            .header(hyper::header::CONTENT_TYPE, _ContentType)
            .body(BoxBody::new(Full::new(_CompressedBytes)))
            .unwrap();
            
        update_headers_for_brotli(_Response.headers_mut());
        _Response
    }
    
    pub fn create_redirect_response(&self) -> Response<ResponseBody> {
        Response::builder()
            .status(StatusCode::FOUND)
            .header(hyper::header::LOCATION, "/captcha")
            .body(BoxBody::new(Full::new(Bytes::from(""))))
            .unwrap()
    }
    
    pub fn handle_request(&self, Request: &Request<Incoming>) -> Option<Response<ResponseBody>> {
        let _Path = Request.uri().path();
        
        if _Path == "/captcha" {
            return Some(self.create_captcha_response());
        }
        
        let _CookieValid = self.extract_cookie(Request)
            .map(|cookie| is_valid_format(&cookie) && validate_cookie(&cookie))
            .unwrap_or(false);
        
        if !_CookieValid {
            return Some(self.create_redirect_response());
        }
        
        None
    }
}