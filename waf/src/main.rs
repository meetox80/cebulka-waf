//waf/src/main.rs
#![allow(non_snake_case)]

use dotenv::dotenv;
use hyper::server::conn::http1;
use hyper::{Request, Response};
use hyper_util::client::legacy::Client;
use hyper_util::client::legacy::connect::HttpConnector;
use hyper_util::rt::TokioExecutor;
use std::env;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use hyper::service::service_fn;
use hyper_util::rt::TokioIo;
use hyper::body::Bytes;
use http_body_util::BodyExt;
use http_body_util::Full;
use hyper::body::Incoming;
use hyper_util::client::legacy::Error as HyperError;
use http_body_util::combinators::BoxBody;
use std::convert::Infallible;
use tokio::time::{interval, Duration};
use std::sync::atomic::{AtomicU64, Ordering};

mod module;
mod modules;
mod endpoints;

use module::{process_content, print_modules_performance_report, get_registered_module_count};
use modules::brotli_compressor::{update_headers_for_brotli, compress_bytes_with_type, compress_content_with_type};
use endpoints::captcha::CaptchaEndpoint;
use modules::dashboard::{increment_request_counter, increment_response_counter};

type ResponseBody = BoxBody<Bytes, Infallible>;

static _REQUEST_COUNTER: AtomicU64 = AtomicU64::new(0);
static _RESPONSE_COUNTER: AtomicU64 = AtomicU64::new(0);

#[tokio::main]
async fn main() {
    dotenv().ok();
    
    modules::init_all();
    
    let _SourcePort: u16 = env::var("SOURCE_PORT")
        .unwrap_or_else(|_| String::from("2025"))
        .parse()
        .unwrap();
    
    let _DestinationPort: u16 = env::var("DESTINATION_PORT")
        .unwrap_or_else(|_| String::from("1337"))
        .parse()
        .unwrap();

    let _ServerAddress: SocketAddr = SocketAddr::from(([127, 0, 0, 1], _SourcePort));
    
    let _ServerListener: TcpListener = match TcpListener::bind(_ServerAddress).await {
        Ok(listener) => listener,
        Err(_e) => {
            std::process::exit(1);
        }
    };
    
    let _HttpClient: Client<HttpConnector, Incoming> = Client::builder(TokioExecutor::new()).build_http();
    let _CaptchaEndpoint = CaptchaEndpoint::new();

    let _ModuleCount = get_registered_module_count();
    
    // Start performance monitoring task
    tokio::spawn(async {
        let mut _ReportInterval = interval(Duration::from_secs(60));
        loop {
            _ReportInterval.tick().await;
            
            let _TotalRequests = _REQUEST_COUNTER.load(Ordering::Relaxed);
            let _TotalResponses = _RESPONSE_COUNTER.load(Ordering::Relaxed);
                              
            print_modules_performance_report();
        }
    });

    loop {
        let (Stream, _ClientAddr) = match _ServerListener.accept().await {
            Ok(connection) => connection,
            Err(_e) => {
                continue;
            }
        };
        
        let ClientClone = _HttpClient.clone();
        let DestPort = _DestinationPort;
        let CaptchaEndpointClone = _CaptchaEndpoint.clone();

        tokio::task::spawn(async move {
            http1::Builder::new()
                .serve_connection(
                    TokioIo::new(Stream),
                    service_fn(move |req| proxy_service(req, ClientClone.clone(), DestPort, CaptchaEndpointClone.clone())),
                )
                .await
                .unwrap_or_else(|_| {});
        });
    }
}

async fn proxy_service(
    Request: Request<Incoming>,
    Client: Client<HttpConnector, Incoming>,
    DestPort: u16,
    CaptchaEndpoint: CaptchaEndpoint,
) -> Result<Response<ResponseBody>, HyperError> {
    _REQUEST_COUNTER.fetch_add(1, Ordering::Relaxed);
    increment_request_counter();
    
    if let Some(Response) = CaptchaEndpoint.handle_request(&Request) {
        _RESPONSE_COUNTER.fetch_add(1, Ordering::Relaxed);
        increment_response_counter();
        return Ok(Response);
    }
    
    let _RequestMethod = Request.method().clone();
    let _RequestUri = Request.uri().clone();
    
    // Forward the request to the destination
    let mut RequestToForward = Request;
    let _DestinationUri: String = format!(
        "http://127.0.0.1:{}{}",
        DestPort,
        RequestToForward.uri().path_and_query().map(|x| x.as_str()).unwrap_or("")
    );
    
    *RequestToForward.uri_mut() = _DestinationUri.parse().unwrap();
    
    let ServerResponse = match Client.request(RequestToForward).await {
        Ok(response) => response,
        Err(e) => {
            let ErrorResponse = Response::builder()
                .status(hyper::StatusCode::BAD_GATEWAY)
                .body(BoxBody::new(Full::new(Bytes::from(format!("Proxy error: {}", e)))))
                .unwrap();
            _RESPONSE_COUNTER.fetch_add(1, Ordering::Relaxed);
            increment_response_counter();
            return Ok(ErrorResponse);
        }
    };
    
    let _ResponseStatus = ServerResponse.status();
    
    let (ResponseParts, ResponseBody) = ServerResponse.into_parts();
    let BodyBytes = match ResponseBody.collect().await {
        Ok(collected) => collected.to_bytes(),
        Err(_) => {
            let ErrorResponse = Response::builder()
                .status(hyper::StatusCode::INTERNAL_SERVER_ERROR)
                .body(BoxBody::new(Full::new(Bytes::from("Error processing response"))))
                .unwrap();
            _RESPONSE_COUNTER.fetch_add(1, Ordering::Relaxed);
            increment_response_counter();
            return Ok(ErrorResponse);
        }
    };
    
    let mut ProcessedResponseParts = ResponseParts;
    let ContentBytes;
    
    let ContentType = ProcessedResponseParts.headers.get(hyper::header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok());
        
    if let Some(TypeValue) = ContentType {
        if let Ok(BodyText) = std::str::from_utf8(&BodyBytes) {
            let mut Content: String = BodyText.to_string();
            process_content(&mut Content);
            ContentBytes = compress_content_with_type(&mut Content, TypeValue);
        } else {
            ContentBytes = compress_bytes_with_type(&BodyBytes, TypeValue);
        }
        update_headers_for_brotli(&mut ProcessedResponseParts.headers);
    } else {
        let mut Content: String = String::from_utf8_lossy(&BodyBytes).into_owned();
        process_content(&mut Content);
        ContentBytes = Bytes::from(Content);
    }
    
    if let Some(Header) = ProcessedResponseParts.headers.get_mut(hyper::header::CONTENT_LENGTH) {
        *Header = hyper::header::HeaderValue::from_str(&ContentBytes.len().to_string()).unwrap();
    }
    
    let NewResponseBody = BoxBody::new(Full::new(ContentBytes));
    let FinalResponse = Response::from_parts(ProcessedResponseParts, NewResponseBody);
    
    _RESPONSE_COUNTER.fetch_add(1, Ordering::Relaxed);
    increment_response_counter();
    Ok(FinalResponse)
}