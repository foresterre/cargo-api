use crate::api::ApiError::HttpRequest;
use crate::api::{ApiError, Client};
use bytes::Bytes;
use http::request::Builder;
use reqwest::ResponseBuilderExt;
use std::borrow::Cow;
use url::Url;

const CRATES_API: &'static str = "https://crates.io/api/";

pub struct ReqwestClient {
    client: reqwest::blocking::Client,
    // Doesn't include the version and beyond.
    // A full path would be: https://crates.io/api/v1/crates/cargo-api
    url: Url,
}

impl ReqwestClient {
    pub fn new(user_agent: &str) -> Self {
        Self {
            client: reqwest::blocking::ClientBuilder::default()
                .user_agent(user_agent)
                .build()
                .unwrap(),
            url: Url::parse(CRATES_API).unwrap(),
        }
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

impl Client for ReqwestClient {
    type Error = Error;

    fn base_endpoint(&self, path: &str) -> Result<Url, ApiError<Self::Error>> {
        self.url.join(path).map_err(|e| ApiError::Url {
            error: e,
            url: Cow::Borrowed(CRATES_API),
            path: Cow::Owned(path.to_string()),
        })
    }

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

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Reqwest {
        #[from]
        error: reqwest::Error,
    },
}

fn into_http_response(res: reqwest::blocking::Response) -> Result<http::Response<Bytes>, Error> {
    let mut builder = http::response::Builder::new()
        .status(res.status())
        .version(res.version())
        .url(res.url().clone());

    for (name, value) in res.headers() {
        builder = builder.header(name, value);
    }

    let body = res.bytes()?;
    let response = builder.body(body).unwrap();

    Ok(response)
}
