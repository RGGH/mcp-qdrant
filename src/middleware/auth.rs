use axum::{
    body::Body,
    extract::Request,
    http::{header, StatusCode},
    middleware::Next,
    response::Response,
};

pub async fn auth_middleware(
    auth_token: Option<String>,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // If no auth token is configured, allow all requests
    let Some(expected_token) = auth_token else {
        return Ok(next.run(request).await);
    };

    // Get Authorization header
    let auth_header = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .ok_or(StatusCode::UNAUTHORIZED)?;

    // Check if it starts with "Bearer "
    if !auth_header.starts_with("Bearer ") {
        return Err(StatusCode::UNAUTHORIZED);
    }

    // Extract token (skip "Bearer " prefix which is 7 chars)
    let provided_token = &auth_header[7..];

    // Use constant-time comparison to prevent timing attacks
    if !constant_time_eq::constant_time_eq(
        provided_token.as_bytes(),
        expected_token.as_bytes(),
    ) {
        return Err(StatusCode::FORBIDDEN);
    }

    // Token is valid, proceed with request
    Ok(next.run(request).await)
}
