//! HTTP security-headers middleware (P7.1).
//!
//! Adds the following response headers to every HTTP response:
//!
//! | Header | Value |
//! |--------|-------|
//! | `X-Content-Type-Options` | `nosniff` |
//! | `X-Frame-Options` | `DENY` |
//! | `Referrer-Policy` | `strict-origin-when-cross-origin` |
//! | `Permissions-Policy` | camera=(), microphone=(), geolocation=() |
//! | `X-XSS-Protection` | `0` (disables legacy IE filter; CSP is the modern solution) |
//!
//! The Content-Security-Policy (CSP) header is intentionally omitted here
//! because it requires tuning per deployment (inline scripts, external CDNs,
//! etc.). It should be configured at the reverse-proxy layer (nginx / Caddy).

use axum::body::Body;
use axum::http::Request;
use axum::middleware::Next;
use axum::response::Response;

/// Axum middleware that injects a standard set of security response headers.
///
/// Mount this as the **outermost** layer so headers are present on every
/// response, including error responses.
pub async fn security_headers_middleware(
    req: Request<Body>,
    next: Next,
) -> Response {
    let mut response = next.run(req).await;

    let headers = response.headers_mut();

    // Prevent MIME-type sniffing.
    headers.insert(
        axum::http::header::HeaderName::from_static("x-content-type-options"),
        axum::http::HeaderValue::from_static("nosniff"),
    );

    // Prevent clickjacking via iframe embedding.
    headers.insert(
        axum::http::header::HeaderName::from_static("x-frame-options"),
        axum::http::HeaderValue::from_static("DENY"),
    );

    // Control Referer information sent to other origins.
    headers.insert(
        axum::http::header::HeaderName::from_static("referrer-policy"),
        axum::http::HeaderValue::from_static("strict-origin-when-cross-origin"),
    );

    // Restrict access to browser features not used by this app.
    headers.insert(
        axum::http::header::HeaderName::from_static("permissions-policy"),
        axum::http::HeaderValue::from_static(
            "camera=(), microphone=(), geolocation=(), payment=()",
        ),
    );

    // Disable legacy XSS filter (superseded by CSP; leaving it enabled can
    // introduce vulnerabilities in older browsers).
    headers.insert(
        axum::http::header::HeaderName::from_static("x-xss-protection"),
        axum::http::HeaderValue::from_static("0"),
    );

    response
}
