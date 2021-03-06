use crate::error::BrokerError;
use crate::utils::http::ResponseExt;
use crate::web::{empty_response, Context, HandlerResult};
use headers::ContentType;
use http::{Response, StatusCode};
use hyper::Body;
use hyper_staticfile::{resolve_path, ResponseBuilder};
use std::env;

/// Handler for the root path, redirects to the Portier homepage.
pub async fn index(_: &mut Context) -> HandlerResult {
    let mut res = empty_response(StatusCode::SEE_OTHER);
    res.header(
        hyper::header::LOCATION,
        "https://portier.github.io".to_owned(),
    );
    Ok(res)
}

/// Version information for the broker.
pub async fn version(_: &mut Context) -> HandlerResult {
    // TODO: Find a more robust way of detecting the git commit.
    // Maybe check/set it in build.rs? Fall back to HEROKU_SLUG_COMMIT?
    let version = env!("CARGO_PKG_VERSION");
    let sha = match env::var("HEROKU_SLUG_COMMIT") {
        Ok(sha) => sha,
        Err(_) => "unknown".to_owned(),
    };
    let body = format!("Portier {} (git commit {})", version, sha);

    let mut res = Response::new(Body::from(body));
    res.typed_header(ContentType::text_utf8());
    Ok(res)
}

/// Static serving of resources.
pub async fn static_(ctx: &mut Context) -> HandlerResult {
    let result = resolve_path(&ctx.app.res_dir, ctx.uri.path())
        .await
        .map_err(|e| BrokerError::Internal(format!("static serving failed: {}", e)))?;
    let res = ResponseBuilder::new()
        .request_parts(&ctx.method, &ctx.uri, &ctx.headers)
        .cache_headers(Some(ctx.app.static_ttl.as_secs() as u32))
        .build(result)
        .expect("could not build static serving response");
    Ok(res)
}
