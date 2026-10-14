use crate::api::ApiError::HttpRequest;
use crate::api::{ApiError, Connection, MissingToken};
use crate::client::CratesIoConnection;
use bytes::Bytes;
use http::request::Builder;
use reqwest::ResponseBuilderExt;
use url::Url;

#[cfg(feature = "blocking_reqwest")]
pub struct ReqwestClient {
    client: reqwest::blocking::Client,
    connection: CratesIoConnection,
}

#[cfg(feature = "blocking_reqwest")]
impl ReqwestClient {
    pub fn new(connection: CratesIoConnection, user_agent: &str) -> Result<Self, NewClientError> {
        Ok(Self {
            client: reqwest::blocking::ClientBuilder::default()
                .user_agent(user_agent)
                .build()?,
            connection,
        })
    }
}

#[cfg(feature = "blocking_reqwest")]
impl Connection for ReqwestClient {
    fn base_url(&self) -> &Url {
        self.connection.base_url()
    }

    fn authorize(&self, request_builder: Builder) -> Result<Builder, MissingToken> {
        self.connection.authorize(request_builder)
    }
}

#[cfg(feature = "blocking_reqwest")]
impl crate::api::Client for ReqwestClient {
    type Error = Error;

    fn send(
        &self,
        request_builder: Builder,
        body: Vec<u8>,
    ) -> Result<http::Response<Bytes>, ApiError<Self::Error>> {
        let request = request_builder
            .body(body)
            .map_err(|error| HttpRequest { error })?;

        send_request(&self.client, request).map_err(|error| ApiError::Client { error })
    }
}

#[cfg(feature = "async_reqwest")]
pub struct ReqwestAsyncClient {
    client: reqwest::Client,
    connection: CratesIoConnection,
}

#[cfg(feature = "async_reqwest")]
impl ReqwestAsyncClient {
    pub fn new(connection: CratesIoConnection, user_agent: &str) -> Result<Self, NewClientError> {
        Ok(Self {
            client: reqwest::ClientBuilder::default()
                .user_agent(user_agent)
                .build()?,
            connection,
        })
    }
}

#[cfg(feature = "async_reqwest")]
impl Connection for ReqwestAsyncClient {
    fn base_url(&self) -> &Url {
        self.connection.base_url()
    }

    fn authorize(&self, request_builder: Builder) -> Result<Builder, MissingToken> {
        self.connection.authorize(request_builder)
    }
}

#[cfg(feature = "async_reqwest")]
impl crate::api::AsyncClient for ReqwestAsyncClient {
    type Error = Error;

    async fn send_async(
        &self,
        request_builder: Builder,
        body: Vec<u8>,
    ) -> Result<http::Response<Bytes>, ApiError<Self::Error>> {
        let request = request_builder
            .body(body)
            .map_err(|error| HttpRequest { error })?;

        send_request_async(&self.client, request)
            .await
            .map_err(|error| ApiError::Client { error })
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

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum NewClientError {
    #[error("Unable to create the HTTP client: {}", error)]
    Reqwest {
        #[from]
        error: reqwest::Error,
    },
}

#[cfg(feature = "blocking_reqwest")]
fn send_request(
    client: &reqwest::blocking::Client,
    request: http::Request<Vec<u8>>,
) -> Result<http::Response<Bytes>, Error> {
    // https://docs.rs/reqwest/latest/reqwest/blocking/struct.Request.html#method.try_from
    let req = request.try_into()?;

    // execute the query
    let res = client.execute(req)?;

    // no try_into from reqwest::blocking::Response for http::Response
    let builder = response_builder(res.status(), res.version(), res.url(), res.headers());
    let body = res.bytes()?;
    let response = builder.body(body).unwrap();

    Ok(response)
}

#[cfg(feature = "async_reqwest")]
async fn send_request_async(
    client: &reqwest::Client,
    request: http::Request<Vec<u8>>,
) -> Result<http::Response<Bytes>, Error> {
    let req = request.try_into()?;

    let res = client.execute(req).await?;

    let builder = response_builder(res.status(), res.version(), res.url(), res.headers());
    let body = res.bytes().await?;
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
