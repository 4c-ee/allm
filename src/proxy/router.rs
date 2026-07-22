//! Per-request dispatch. `route` is the body of the `service_fn`
//! closure each hyper connection runs — a flat `match` over
//! `(method, path)` for the routes the proxy answers, mirroring the
//! style of [`crate::ipc::methods::dispatch_request`].
//!
//! The route table covers the OpenAI compat surface (`/v1/...`):
//! `/v1/models`, `/v1/chat/completions`, `/v1/completions`,
//! `/v1/embeddings`, `/v1/rerank`. This is the only surface — the
//! single URL any OpenAI-compatible client attaches through.

use std::error::Error as StdError;
use std::sync::Arc;

use http_body_util::{combinators::BoxBody, BodyExt, Full};
use hyper::body::{Bytes, Incoming};
use hyper::{Method, Request, Response, StatusCode};

use super::forward;
use super::openai::{ErrorObject, ErrorResponse, ModelList, ModelObject};
use super::route::{self, RouteDecision};
use super::state::ProxyState;
use crate::discovery::DiscoveredModel;

/// The error type our `BoxBody` carries. Forwarding streams upstream
/// `reqwest::Response::bytes_stream()` chunks through `StreamBody`,
/// so the body alias must accept *some* error type at frame time.
pub type BodyError = Box<dyn StdError + Send + Sync>;

pub type ProxyResponse = Result<Response<BoxBody<Bytes, BodyError>>, hyper::Error>;

/// Entry point invoked by the `service_fn` closure. Returns a fully
/// constructed `Response`; the caller hands it back to hyper.
pub async fn route(state: Arc<ProxyState>, req: Request<Incoming>) -> ProxyResponse {
  let method = req.method().clone();
  let path = req.uri().path().to_string();

  let auth_exempt = matches!(
    (&method, path.as_str()),
    (&Method::GET | &Method::HEAD, "/")
  );
  if !auth_exempt && state.auth.enforced() && !state.auth.check(req.headers()) {
    return unauthorized();
  }

  match (&method, path.as_str()) {
    (&Method::GET | &Method::HEAD, "/") => root_identity(),
    (&Method::GET, "/v1/models") => list_models(state).await,
    (&Method::POST, "/v1/chat/completions") => forward_request(state, req).await,
    (&Method::POST, "/v1/completions") => forward_request(state, req).await,
    (&Method::POST, "/v1/embeddings") => forward_request(state, req).await,
    (&Method::POST, "/v1/rerank") => forward_request(state, req).await,
    _ => not_found(),
  }
}

fn root_identity() -> ProxyResponse {
  Ok(text_response(StatusCode::OK, "LlamaStash is running\n"))
}

fn text_response(status: StatusCode, body: &'static str) -> Response<BoxBody<Bytes, BodyError>> {
  Response::builder()
    .status(status)
    .header(hyper::header::CONTENT_TYPE, "text/plain; charset=utf-8")
    .body(full_body(Bytes::from_static(body.as_bytes())))
    .expect("static text response must build")
}

async fn forward_request(state: Arc<ProxyState>, req: Request<Incoming>) -> ProxyResponse {
  let (method, uri, headers, body) = forward::deconstruct(req);

  let parsed = match route::buffer_and_extract(body).await {
    Ok(p) => p,
    Err(e) => return route::body_error_response(e),
  };

  let decision = route::decide(&state, parsed.model).await;
  match decision {
    RouteDecision::ReadyAt {
      port,
      served_model_id,
      served_model_key,
      upstream_path_prefix,
      fallback,
      fallback_reason,
    } => {
      state.mru.touch(&served_model_key).await;
      forward::forward_to_upstream(
        &state,
        forward::InboundRequest {
          method,
          uri,
          headers,
          body_bytes: parsed.bytes,
        },
        forward::Target {
          port,
          served_model_id: &served_model_id,
          served_model_key: &served_model_key,
          upstream_path_prefix: upstream_path_prefix.as_deref(),
          fallback,
          fallback_reason: fallback_reason.as_deref(),
        },
      )
      .await
    }
    RouteDecision::NotRunning {
      requested_model,
      resolved_row,
      arch,
    } => {
      let inbound = forward::InboundRequest {
        method,
        uri,
        headers,
        body_bytes: parsed.bytes,
      };
      route::handle_not_running(&state, inbound, requested_model, *resolved_row, arch).await
    }
    RouteDecision::NotFound { requested_model } => error_with_matches(
      StatusCode::NOT_FOUND,
      "model_not_found",
      &format!("{requested_model} not found"),
      Vec::<String>::new(),
    ),
    RouteDecision::Ambiguous {
      requested_model,
      candidates,
    } => {
      let message = format!(
        "`{requested_model}` matched {n} models; refine the reference (full path or unique substring)",
        n = candidates.len()
      );
      error_with_matches(
        StatusCode::BAD_REQUEST,
        "ambiguous_model",
        &message,
        candidates,
      )
    }
    RouteDecision::ModelRequired => error_with_code(
      StatusCode::BAD_REQUEST,
      "invalid_request",
      "the `model` field is required",
      "model_required",
      Some("model"),
    ),
    RouteDecision::BackendUnavailable { .. } => error_response(
      StatusCode::SERVICE_UNAVAILABLE,
      "backend_unavailable",
      "the requested backend is not running; start it first",
    ),
  }
}

async fn list_models(state: Arc<ProxyState>) -> ProxyResponse {
  let snap = state.ctx.catalog.snapshot().await;
  let mut rows: Vec<ModelObject> = snap
    .iter()
    .map(|m| ModelObject::new(model_id_for(m)))
    .collect();
  rows.sort_by(|a, b| a.id.cmp(&b.id));
  let list = ModelList::new(rows);
  let bytes = serde_json::to_vec(&list).expect("json encoding of fixed shape");
  Ok(json_response(StatusCode::OK, bytes))
}

/// Project a [`DiscoveredModel`] onto the `id` field of an OpenAI
/// `model` object. Rule: `display_label` wins when set, otherwise
/// fall back to `path.file_stem()`.
fn model_id_for(m: &DiscoveredModel) -> String {
  if let Some(label) = &m.display_label {
    return label.clone();
  }
  crate::util::paths::model_display_name(&m.path)
}

fn not_found() -> ProxyResponse {
  error_response(StatusCode::NOT_FOUND, "not_found", "no such route")
}

fn unauthorized() -> ProxyResponse {
  let mut resp = json_response(StatusCode::UNAUTHORIZED, unauthorized_body());
  resp.headers_mut().insert(
    hyper::header::WWW_AUTHENTICATE,
    hyper::header::HeaderValue::from_static("Bearer"),
  );
  Ok(resp)
}

fn unauthorized_body() -> Vec<u8> {
  serde_json::to_vec(&ErrorResponse {
    error: ErrorObject::new("invalid_request_error", "missing or invalid API key")
      .with_code("invalid_api_key"),
  })
  .expect("json encoding of fixed shape")
}

pub(crate) fn error_response(status: StatusCode, r#type: &str, message: &str) -> ProxyResponse {
  let body = ErrorResponse {
    error: ErrorObject::new(r#type, message),
  };
  let bytes = serde_json::to_vec(&body).expect("json encoding of fixed shape");
  Ok(json_response(status, bytes))
}

pub(crate) fn error_with_code(
  status: StatusCode,
  r#type: &str,
  message: &str,
  code: &str,
  param: Option<&str>,
) -> ProxyResponse {
  let mut error = ErrorObject::new(r#type, message).with_code(code);
  if let Some(p) = param {
    error = error.with_param(p);
  }
  let bytes = serde_json::to_vec(&ErrorResponse { error }).expect("json encoding of fixed shape");
  Ok(json_response(status, bytes))
}

pub(crate) fn error_with_matches<I, S>(
  status: StatusCode,
  r#type: &str,
  message: &str,
  matches: I,
) -> ProxyResponse
where
  I: IntoIterator<Item = S>,
  S: Into<String>,
{
  let error = ErrorObject::new(r#type, message).with_matches(matches);
  let bytes = serde_json::to_vec(&ErrorResponse { error }).expect("json encoding of fixed shape");
  Ok(json_response(status, bytes))
}

pub(crate) fn json_response(
  status: StatusCode,
  body: Vec<u8>,
) -> Response<BoxBody<Bytes, BodyError>> {
  let body = full_body(Bytes::from(body));
  Response::builder()
    .status(status)
    .header(hyper::header::CONTENT_TYPE, "application/json")
    .body(body)
    .expect("static headers always parse")
}

fn full_body(bytes: Bytes) -> BoxBody<Bytes, BodyError> {
  Full::new(bytes).map_err(|never| match never {}).boxed()
}
