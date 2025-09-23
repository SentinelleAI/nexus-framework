use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::middleware::Next;
use colored::Colorize;
use http_body_util::BodyExt;
use std::time::Instant;
use crate::prelude::{IntoResponse, Response};

fn log_debug_details(
    method: &hyper::Method,
    uri: &hyper::Uri,
    status: StatusCode,
    duration: std::time::Duration,
    req_body: &str,
    res_body: &str,
    res_headers: &hyper::HeaderMap,
) {
    // Indent every line of the request and response bodies
    let indented_req_body = req_body.lines().map(|line| format!("\n│   {}", line)).collect::<String>();
    let indented_res_body = res_body.lines().map(|line| format!("\n│   {}", line)).collect::<String>();

    tracing::debug!(
        "\n╭────────── Request Summary ───\n│ Request: {} {}\n│ Request Body: {}\n│ Response: {}\n│ Duration: {:?}\n│ Headers: {:?}\n│ Response Body: {}\n╰────────────",
        method,
        uri,
        if indented_req_body.is_empty() { " (empty)".to_string() } else { indented_req_body },
        status,
        duration,
        res_headers,
        if indented_res_body.is_empty() { " (empty)".to_string() } else { indented_res_body },
    );
}

pub async fn request_logging(mut req: Request<Body>, next: Next) -> impl IntoResponse {
    let start = Instant::now();
    let method = req.method().clone();
    let uri = req.uri().clone();

    let req_body_bytes = if tracing::enabled!(tracing::Level::DEBUG) {
        let body = std::mem::replace(req.body_mut(), Body::empty());
        let bytes = body.collect().await.map_or(bytes::Bytes::new(), |c| c.to_bytes());
        *req.body_mut() = Body::from(bytes.clone());
        Some(bytes)
    } else {
        None
    };

    // --- Run the handler ---
    let response = next.run(req).await;

    // --- Prepare summary log info ---
    let span = tracing::info_span!(
        "nfw-core"
    );
    let _enter = span.enter();
    let span = tracing::info_span!(
        "request"
    );
    let _enter = span.enter();
    let duration = start.elapsed();
    let status = response.status();
    let status_colored = match status.as_u16() {
        200..=299 => status.to_string().green(),
        400..=599 => status.to_string().red(),
        _ => status.to_string().yellow(),
    };
    let method_colored = match method.as_str() {
        "GET" => "GET".green().bold(),
        "POST" => "POST".yellow().bold(),
        "PUT" => "PUT".blue().bold(),
        "PATCH" => "PATCH".magenta().bold(),
        "DELETE" => "DELETE".red().bold(),
        "HEAD" => "HEAD".green().bold(),
        "OPTIONS" => "OPTIONS".bright_black().bold(),
        _ => "UNKNWN".normal(),
    };

    // Log the summary for every request
    tracing::info!(
        "{} {} -> {} ({:?})",
        method_colored,
        uri,
        status_colored,
        duration
    );

    // --- Buffer and log the response body only in DEBUG mode ---
    if let Some(req_bytes) = req_body_bytes {
        let (parts, body) = response.into_parts();
        let res_bytes = body.collect().await.map_or(bytes::Bytes::new(), |c| c.to_bytes());

        let req_body_str = String::from_utf8_lossy(&req_bytes);
        let res_body_str = String::from_utf8_lossy(&res_bytes);

        log_debug_details(&method, &uri, status, duration, &req_body_str, &res_body_str, &parts.headers);

        return Response::from_parts(parts, Body::from(res_bytes));
    }

    response
}