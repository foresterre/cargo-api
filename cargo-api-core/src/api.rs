pub mod crates;

use bytes::Bytes;
use http::StatusCode;
use std::borrow::Cow;
use std::fmt;
use url::Url;

// A list rather than a map, because some parameters (e.g. `ids[]`) may be repeated.
#[derive(fmt::Debug, Default)]
pub struct QueryParams {
    inner: Vec<(Cow<'static, str>, Cow<'static, str>)>,
}

impl QueryParams {
    pub fn push(
        &mut self,
        key: impl Into<Cow<'static, str>>,
        value: impl Into<Cow<'static, str>>,
    ) -> &mut Self {
        self.inner.push((key.into(), value.into()));
        self
    }

    pub fn append_to_url(&self, url: &mut Url) {
        url.query_pairs_mut().extend_pairs(self.iter_tuples());
    }

    fn iter_tuples(&self) -> impl Iterator<Item = (&str, &str)> {
        self.inner.iter().map(|(k, v)| (k.as_ref(), v.as_ref()))
    }
}

#[derive(fmt::Debug, thiserror::Error)]
#[error(transparent)]
#[non_exhaustive]
pub struct BodyError {
    pub error: serde_json::Error,
}

/// The body of a request, together with its content type.
#[derive(Clone, fmt::Debug, PartialEq, Eq)]
pub struct Body {
    content_type: &'static str,
    bytes: Vec<u8>,
}

impl Body {
    pub fn new(content_type: &'static str, bytes: Vec<u8>) -> Self {
        Self {
            content_type,
            bytes,
        }
    }

    pub fn json<T: serde::Serialize + ?Sized>(value: &T) -> Result<Self, BodyError> {
        serde_json::to_vec(value)
            .map(|bytes| Self::new("application/json", bytes))
            .map_err(|error| BodyError { error })
    }

    pub fn content_type(&self) -> &'static str {
        self.content_type
    }

    pub fn into_bytes(self) -> Vec<u8> {
        self.bytes
    }
}

/// Whether an endpoint must be called with an API token.
#[derive(Clone, Copy, fmt::Debug, Default, PartialEq, Eq)]
pub enum Authentication {
    #[default]
    None,
    Required,
}

#[derive(fmt::Debug, thiserror::Error)]
#[non_exhaustive]
pub enum JsonResult {
    #[error("{0}")]
    Json(serde_json::Value),
    #[error(transparent)]
    Error(serde_json::Error),
}

impl From<Result<serde_json::Value, serde_json::Error>> for JsonResult {
    fn from(value: Result<serde_json::Value, serde_json::Error>) -> Self {
        match value {
            Ok(v) => JsonResult::Json(v),
            Err(e) => JsonResult::Error(e),
        }
    }
}

#[derive(fmt::Debug, thiserror::Error)]
#[non_exhaustive]
pub enum ApiError<C: std::error::Error + Send + Sync + 'static> {
    #[error("Body error: {}", error)]
    Body {
        #[from]
        error: BodyError,
    },
    #[error("Client error: {}", error)]
    Client { error: C },
    #[error("Missing required API token")]
    MissingToken,
    #[error("Unable to build HTTP request: {}", error)]
    HttpRequest { error: http::Error },
    #[error("HTTP request failed with status code '{}': {}", status_code, body)]
    HttpResponse {
        status_code: StatusCode,
        body: JsonResult,
    },
    #[error("Unable to parse JSON response into type '{}': {}", r#type, error)]
    ParseType {
        error: serde_json::Error,
        r#type: &'static str,
    },
    #[error("Unable to parse url '{}' (path '{}'): {}", url, path, error)]
    Url {
        error: url::ParseError,
        url: Cow<'static, str>,
        path: Cow<'static, str>,
    },
}

impl<C: std::error::Error + Send + Sync + 'static> ApiError<C> {
    pub fn parse_type_error<T>(error: serde_json::Error) -> Self {
        ApiError::ParseType {
            error,
            r#type: std::any::type_name::<T>(),
        }
    }
}

/// Http endpoint trait for cargo-api.
///
/// # Credits
///
/// Inspired by Ben Boeckel's blog [post](https://plume.benboeckel.net/~/JustAnotherBlog/designing-rust-bindings-for-rest-ap-is)
/// titled "Designing Rust bindings for REST APIs".
pub trait Endpoint {
    fn method(&self) -> http::Method;

    fn endpoint(&self) -> Cow<'static, str>;

    fn parameters(&self) -> QueryParams {
        QueryParams::default()
    }

    fn body(&self) -> Result<Option<Body>, BodyError> {
        Ok(None)
    }

    fn authentication(&self) -> Authentication {
        Authentication::None
    }
}

/// Http api trait for cargo-api.
///
/// # Credits
///
/// Inspired by Ben Boeckel's blog [post](https://plume.benboeckel.net/~/JustAnotherBlog/designing-rust-bindings-for-rest-ap-is)
/// titled "Designing Rust bindings for REST APIs".
pub trait Client {
    type Error: std::error::Error + Send + Sync + 'static;

    fn base_endpoint(&self, path: &str) -> Result<Url, ApiError<Self::Error>>;

    /// Adds the API token to a request of an endpoint which requires authentication.
    ///
    /// Returns [`ApiError::MissingToken`] when the client has no API token.
    fn authorize(
        &self,
        request_builder: http::request::Builder,
    ) -> Result<http::request::Builder, ApiError<Self::Error>>;

    // By separating the request builder and the body, additional items may be added
    // to the request, such as authentication.
    fn send(
        &self,
        request_builder: http::request::Builder,
        body: Vec<u8>,
    ) -> Result<http::Response<Bytes>, ApiError<Self::Error>>;
}

/// Query trait for 'cargo-api'
///
/// # Credits
///
/// Inspired by Ben Boeckel's blog [post](https://plume.benboeckel.net/~/JustAnotherBlog/designing-rust-bindings-for-rest-ap-is)
/// titled "Designing Rust bindings for REST APIs".
pub trait Query<T, C: Client> {
    fn query(&self, client: &C) -> Result<T, ApiError<C::Error>>;
}

impl<E> Endpoint for &E
where
    E: Endpoint,
{
    fn method(&self) -> http::Method {
        (*self).method()
    }

    fn endpoint(&self) -> Cow<'static, str> {
        (*self).endpoint()
    }

    fn parameters(&self) -> QueryParams {
        (*self).parameters()
    }

    fn body(&self) -> Result<Option<Body>, BodyError> {
        (*self).body()
    }

    fn authentication(&self) -> Authentication {
        (*self).authentication()
    }
}

impl<E, T, C> Query<T, C> for E
where
    E: Endpoint,
    T: serde::de::DeserializeOwned,
    C: Client,
{
    fn query(&self, client: &C) -> Result<T, ApiError<C::Error>> {
        // -- compute the URL
        // this is the base url with the path, but excluding any query parameters
        let mut url = client.base_endpoint(self.endpoint().as_ref())?;
        // add query parameters to the url
        self.parameters().append_to_url(&mut url);

        // -- build the request
        let body = self.body()?;
        // responses are always parsed as JSON, and some endpoints (e.g. download) only
        // respond with JSON when it is explicitly asked for
        let mut request = http::Request::builder()
            .method(self.method())
            .uri(url.as_ref())
            .header(http::header::ACCEPT, "application/json");

        if let Some(body) = &body {
            request = request.header(http::header::CONTENT_TYPE, body.content_type());
        }

        let request = match self.authentication() {
            Authentication::None => request,
            Authentication::Required => client.authorize(request)?,
        };

        // -- send
        let body = body.map(Body::into_bytes).unwrap_or_default();
        let response = client.send(request, body)?;

        // -- handle response errors
        if !response.status().is_success() {
            // request failed, can be any non-2xx for now
            return Err(ApiError::HttpResponse {
                status_code: response.status(),
                body: serde_json::from_slice(response.body()).into(),
            });
        }

        // -- parse type
        serde_json::from_slice::<T>(response.body()).map_err(ApiError::parse_type_error::<T>)
    }
}

pub struct Json<E> {
    endpoint: E,
}

impl<E> Json<E> {
    pub fn new(endpoint: E) -> Self {
        Self { endpoint }
    }
}

impl<E, C> Query<serde_json::Value, C> for Json<E>
where
    E: Endpoint,
    C: Client,
{
    fn query(&self, client: &C) -> Result<serde_json::Value, ApiError<C::Error>> {
        self.endpoint.query(client)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::crates::{AddOwners, Crate, Yank};
    use std::cell::RefCell;

    #[derive(fmt::Debug, thiserror::Error)]
    #[error("fake client error")]
    struct FakeError;

    /// Records the request it is asked to send, and responds with a fixed response.
    struct FakeClient {
        token: Option<&'static str>,
        status: StatusCode,
        response: &'static str,
        sent: RefCell<Option<http::Request<Vec<u8>>>>,
    }

    impl FakeClient {
        fn new(token: Option<&'static str>) -> Self {
            Self {
                token,
                status: StatusCode::OK,
                response: r#"{"ok":true}"#,
                sent: RefCell::new(None),
            }
        }

        fn sent(&self) -> http::Request<Vec<u8>> {
            self.sent.take().expect("expected a request to be sent")
        }
    }

    impl Client for FakeClient {
        type Error = FakeError;

        fn base_endpoint(&self, path: &str) -> Result<Url, ApiError<Self::Error>> {
            Ok(Url::parse("https://crates.test/api/")
                .unwrap()
                .join(path)
                .unwrap())
        }

        fn authorize(
            &self,
            request_builder: http::request::Builder,
        ) -> Result<http::request::Builder, ApiError<Self::Error>> {
            match self.token {
                Some(token) => Ok(request_builder.header(http::header::AUTHORIZATION, token)),
                None => Err(ApiError::MissingToken),
            }
        }

        fn send(
            &self,
            request_builder: http::request::Builder,
            body: Vec<u8>,
        ) -> Result<http::Response<Bytes>, ApiError<Self::Error>> {
            let request = request_builder.body(body).unwrap();
            self.sent.replace(Some(request));

            Ok(http::Response::builder()
                .status(self.status)
                .body(Bytes::from_static(self.response.as_bytes()))
                .unwrap())
        }
    }

    fn yank() -> Yank<'static> {
        Yank::new("serde".into(), Cow::Owned(semver::Version::new(1, 0, 0)))
    }

    #[test]
    fn asks_for_json_without_authentication() {
        let client = FakeClient::new(Some("token"));

        let _: serde_json::Value = Crate::new("serde".into()).query(&client).unwrap();

        let request = client.sent();
        assert_eq!(request.method(), http::Method::GET);
        assert_eq!(request.uri(), "https://crates.test/api/v1/crates/serde");
        assert_eq!(request.headers()[http::header::ACCEPT], "application/json");
        assert!(!request.headers().contains_key(http::header::AUTHORIZATION));
        assert!(!request.headers().contains_key(http::header::CONTENT_TYPE));
        assert!(request.body().is_empty());
    }

    #[test]
    fn authorizes_endpoints_which_require_it() {
        let client = FakeClient::new(Some("token"));

        let _: serde_json::Value = yank().query(&client).unwrap();

        assert_eq!(
            client.sent().headers()[http::header::AUTHORIZATION],
            "token"
        );
    }

    #[test]
    fn missing_token_is_an_error_before_sending() {
        let client = FakeClient::new(None);

        let result: Result<serde_json::Value, _> = yank().query(&client);

        assert!(matches!(result, Err(ApiError::MissingToken)));
        assert!(client.sent.borrow().is_none());
    }

    #[test]
    fn sends_the_body_with_its_content_type() {
        let client = FakeClient::new(Some("token"));
        let endpoint = AddOwners::new("serde".into(), vec!["ghost".into()]);

        let _: serde_json::Value = endpoint.query(&client).unwrap();

        let request = client.sent();
        assert_eq!(
            request.headers()[http::header::CONTENT_TYPE],
            "application/json"
        );
        assert_eq!(request.body(), br#"{"owners":["ghost"]}"#);
    }

    #[test]
    fn repeats_query_parameters() {
        let client = FakeClient::new(None);
        let endpoint =
            crates::Search::new().with_filter(crates::Filter::Ids(vec!["a".into(), "b".into()]));

        let _: serde_json::Value = endpoint.query(&client).unwrap();

        assert_eq!(
            client.sent().uri(),
            "https://crates.test/api/v1/crates?ids%5B%5D=a&ids%5B%5D=b"
        );
    }

    #[test]
    fn unsuccessful_status_is_an_error() {
        let client = FakeClient {
            status: StatusCode::FORBIDDEN,
            response: r#"{"errors":[{"detail":"must be logged in"}]}"#,
            ..FakeClient::new(Some("token"))
        };

        let result: Result<serde_json::Value, _> = yank().query(&client);

        match result {
            Err(ApiError::HttpResponse {
                status_code,
                body: JsonResult::Json(body),
            }) => {
                assert_eq!(status_code, StatusCode::FORBIDDEN);
                assert_eq!(body["errors"][0]["detail"], "must be logged in");
            }
            other => panic!("expected an HTTP response error, got {:?}", other),
        }
    }
}
