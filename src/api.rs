mod crates;

use bytes::Bytes;
use http::StatusCode;
use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt;
use url::Url;

#[derive(fmt::Debug, Default)]
pub struct QueryParams {
    inner: HashMap<Cow<'static, str>, Cow<'static, str>>,
}

impl QueryParams {
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

    fn body(&self) -> Result<Vec<u8>, BodyError> {
        Ok(Vec::with_capacity(0))
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

    fn body(&self) -> Result<Vec<u8>, BodyError> {
        (*self).body()
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
        let request = http::Request::builder()
            .method(self.method())
            .uri(url.as_ref());

        // -- send
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
