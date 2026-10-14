use cargo_api_core::api::crates::{AddOwners, Crate, Yank};
use cargo_api_core::api::{ApiError, AsyncQuery, JsonResult};
use cargo_api_core::client::{CratesIoConnection, ReqwestAsyncClient, Token};
use http::StatusCode;
use mockito::{Server, ServerGuard};
use semver::Version;
use std::borrow::Cow;
use url::Url;

const USER_AGENT: &str = "cargo-api-test";
const TOKEN: &str = "cio_secret";

fn client(server: &ServerGuard, token: Option<&str>) -> ReqwestAsyncClient {
    let mut connection = CratesIoConnection::new(Url::parse(&server.url()).unwrap());

    if let Some(token) = token {
        connection = connection.with_token(Token::new(token).unwrap());
    }

    ReqwestAsyncClient::new(connection, USER_AGENT).unwrap()
}

fn yank() -> Yank<'static> {
    Yank::new("serde".into(), Cow::Owned(Version::new(1, 0, 0)))
}

#[tokio::test]
async fn read_crate() {
    let mut server = Server::new_async().await;
    let mock = server
        .mock("GET", "/v1/crates/serde")
        .match_header("user-agent", USER_AGENT)
        .match_header("accept", "application/json")
        .match_header("authorization", mockito::Matcher::Missing)
        .with_body(r#"{"crate":{"name":"serde"}}"#)
        .create_async()
        .await;

    let value: serde_json::Value = Crate::new("serde".into())
        .query_async(&client(&server, Some(TOKEN)))
        .await
        .unwrap();

    mock.assert_async().await;
    assert_eq!(value["crate"]["name"], "serde");
}

#[tokio::test]
async fn add_owners() {
    let mut server = Server::new_async().await;
    let mock = server
        .mock("PUT", "/v1/crates/serde/owners")
        .match_header("authorization", TOKEN)
        .match_header("content-type", "application/json")
        .match_body(r#"{"owners":["ghost"]}"#)
        .with_body(r#"{"ok":true}"#)
        .create_async()
        .await;

    let value: serde_json::Value = AddOwners::new("serde".into(), vec!["ghost".into()])
        .query_async(&client(&server, Some(TOKEN)))
        .await
        .unwrap();

    mock.assert_async().await;
    assert_eq!(value, serde_json::json!({ "ok": true }));
}

#[tokio::test]
async fn missing_token() {
    let mut server = Server::new_async().await;
    let mock = server
        .mock("DELETE", "/v1/crates/serde/1.0.0/yank")
        .expect(0)
        .create_async()
        .await;

    let result: Result<serde_json::Value, _> = yank().query_async(&client(&server, None)).await;

    mock.assert_async().await;
    assert!(matches!(result, Err(ApiError::MissingToken)));
}

#[tokio::test]
async fn unsuccessful_status() {
    let mut server = Server::new_async().await;
    let mock = server
        .mock("DELETE", "/v1/crates/serde/1.0.0/yank")
        .with_status(403)
        .with_body(r#"{"errors":[{"detail":"must be logged in"}]}"#)
        .create_async()
        .await;

    let result: Result<serde_json::Value, _> =
        yank().query_async(&client(&server, Some(TOKEN))).await;

    mock.assert_async().await;
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

#[tokio::test]
async fn invalid_user_agent() {
    let result = ReqwestAsyncClient::new(CratesIoConnection::default(), "cargo-api\ntest");

    assert!(result.is_err());
}
