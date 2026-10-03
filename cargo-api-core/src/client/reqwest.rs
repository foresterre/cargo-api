use crate::api::ApiError::HttpRequest;
use crate::api::{ApiError, Client, RestClient};
use crate::client::Token;
use bytes::Bytes;
use http::request::Builder;
use reqwest::ResponseBuilderExt;
use std::borrow::Cow;
use url::Url;

const CRATES_API: &str = "https://crates.io/api/";

pub struct ReqwestClient {
    client: reqwest::blocking::Client,
    // Doesn't include the version and beyond.
    // A full path would be: https://crates.io/api/v1/crates/cargo-api
    url: Url,
    token: Option<Token>,
}

impl ReqwestClient {
    pub fn new(user_agent: &str) -> Self {
        Self {
            client: reqwest::blocking::ClientBuilder::default()
                .user_agent(user_agent)
                .build()
                .unwrap(),
            url: Url::parse(CRATES_API).unwrap(),
            token: None,
        }
    }

    /// API token for endpoints which require authentication.
    pub fn with_token(mut self, token: Token) -> Self {
        self.token = Some(token);
        self
    }

    fn send_request(
        &self,
        request: http::Request<Vec<u8>>,
    ) -> Result<http::Response<Bytes>, Error> {
        // https://docs.rs/reqwest/latest/reqwest/blocking/struct.Request.html#method.try_from
        let req = request.try_into()?;

        // execute the query
        let res = self.client.execute(req)?;

        // no try_into from reqwest::blocking::Response for http::Response
        let http_res = into_http_response(res)?;

        Ok(http_res)
    }
}

impl RestClient for ReqwestClient {
    type Error = Error;

    fn base_endpoint(&self, path: &str) -> Result<Url, ApiError<Self::Error>> {
        base_endpoint(&self.url, path)
    }

    fn authorize(&self, request_builder: Builder) -> Result<Builder, ApiError<Self::Error>> {
        authorize(self.token.as_ref(), request_builder)
    }
}

impl Client for ReqwestClient {
    fn send(
        &self,
        request_builder: Builder,
        body: Vec<u8>,
    ) -> Result<http::Response<Bytes>, ApiError<Self::Error>> {
        let request = request_builder
            .body(body)
            .map_err(|error| HttpRequest { error })?;

        self.send_request(request)
            .map_err(|error| ApiError::Client { error })
    }
}

#[cfg(feature = "async")]
pub struct AsyncReqwestClient {
    client: reqwest::Client,
    // Doesn't include the version and beyond.
    // A full path would be: https://crates.io/api/v1/crates/cargo-api
    url: Url,
    token: Option<Token>,
}

#[cfg(feature = "async")]
impl AsyncReqwestClient {
    pub fn new(user_agent: &str) -> Self {
        Self {
            client: reqwest::ClientBuilder::default()
                .user_agent(user_agent)
                .build()
                .unwrap(),
            url: Url::parse(CRATES_API).unwrap(),
            token: None,
        }
    }

    /// API token for endpoints which require authentication.
    pub fn with_token(mut self, token: Token) -> Self {
        self.token = Some(token);
        self
    }

    async fn send_request(
        &self,
        request: http::Request<Vec<u8>>,
    ) -> Result<http::Response<Bytes>, Error> {
        let req = request.try_into()?;
        let res = self.client.execute(req).await?;

        let builder = response_builder(res.status(), res.version(), res.url(), res.headers());
        let body = res.bytes().await?;

        Ok(builder.body(body).unwrap())
    }
}

#[cfg(feature = "async")]
impl RestClient for AsyncReqwestClient {
    type Error = Error;

    fn base_endpoint(&self, path: &str) -> Result<Url, ApiError<Self::Error>> {
        base_endpoint(&self.url, path)
    }

    fn authorize(&self, request_builder: Builder) -> Result<Builder, ApiError<Self::Error>> {
        authorize(self.token.as_ref(), request_builder)
    }
}

#[cfg(feature = "async")]
impl crate::api::AsyncClient for AsyncReqwestClient {
    async fn send_async(
        &self,
        request_builder: Builder,
        body: Vec<u8>,
    ) -> Result<http::Response<Bytes>, ApiError<Self::Error>> {
        let request = request_builder
            .body(body)
            .map_err(|error| HttpRequest { error })?;

        self.send_request(request)
            .await
            .map_err(|error| ApiError::Client { error })
    }
}

fn base_endpoint(url: &Url, path: &str) -> Result<Url, ApiError<Error>> {
    url.join(path).map_err(|e| ApiError::Url {
        error: e,
        url: Cow::Borrowed(CRATES_API),
        path: Cow::Owned(path.to_string()),
    })
}

fn authorize(token: Option<&Token>, request_builder: Builder) -> Result<Builder, ApiError<Error>> {
    match token {
        Some(token) => Ok(
            request_builder.header(http::header::AUTHORIZATION, token.as_header_value().clone())
        ),
        None => Err(ApiError::MissingToken),
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Reqwest {
        #[from]
        error: reqwest::Error,
    },
}

fn into_http_response(res: reqwest::blocking::Response) -> Result<http::Response<Bytes>, Error> {
    let builder = response_builder(res.status(), res.version(), res.url(), res.headers());

    let body = res.bytes()?;
    let response = builder.body(body).unwrap();

    Ok(response)
}

fn response_builder(
    status: http::StatusCode,
    version: http::Version,
    url: &Url,
    headers: &http::HeaderMap,
) -> http::response::Builder {
    let mut builder = http::response::Builder::new()
        .status(status)
        .version(version)
        .url(url.clone());

    for (name, value) in headers {
        builder = builder.header(name, value);
    }

    builder
}
