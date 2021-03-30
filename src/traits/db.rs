// Copyright (c) 2021 ruarango developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

//! `ruarango` database impl

use super::Database;
use crate::{
    conn::Connection,
    model::{
        db::{Create, Current},
        Response,
    },
    utils::handle_response,
};
use anyhow::{Context, Result};
use async_trait::async_trait;
use futures::future::FutureExt;

macro_rules! api_get {
    ($self:ident, $url:ident, $suffix:literal) => {{
        let current_url = $self
            .$url()
            .join($suffix)
            .with_context(|| format!("Unable to build '{}' url", $suffix))?;
        Ok($self
            .client()
            .get(current_url)
            .send()
            .then(handle_response)
            .await?)
    }};
}

#[async_trait]
impl Database for Connection {
    async fn current(&self) -> Result<Response<Current>> {
        api_get!(self, db_url, "_api/database/current")
    }

    async fn user(&self) -> Result<Response<Vec<String>>> {
        api_get!(self, db_url, "_api/database/user")
    }

    async fn list(&self) -> Result<Response<Vec<String>>> {
        api_get!(self, base_url, "_api/database")
    }

    async fn create(&self, create: &Create) -> Result<Response<bool>> {
        let current_url = self
            .base_url()
            .join("_api/database")
            .with_context(|| "Unable to build '_api/database' url")?;
        Ok(self
            .client()
            .post(current_url)
            .json(create)
            .send()
            .then(handle_response)
            .await?)
    }
    async fn drop(&self, name: &str) -> Result<Response<bool>> {
        let current_url = self
            .base_url()
            .join(&format!("_api/database/{}", name))
            .with_context(|| "Unable to build '_api/database' url")?;
        Ok(self
            .client()
            .delete(current_url)
            .send()
            .then(handle_response)
            .await?)
    }
}

#[cfg(test)]
mod test {
    use crate::{
        db::{CreateBuilder, Current, OptionsBuilder, UserBuilder},
        error::RuarangoError::{self, TestError},
        model::Response,
        traits::Database,
        utils::{default_conn, mock_auth, no_db_conn},
    };
    use anyhow::Result;
    use wiremock::{
        matchers::{body_string_contains, method, path},
        Mock, MockServer, ResponseTemplate,
    };

    async fn mock_current(mock_server: &MockServer) {
        let body = Response::<Current>::default();
        let mock_response = ResponseTemplate::new(200).set_body_json(body);

        Mock::given(method("GET"))
            .and(path("/_db/keti/_api/database/current"))
            .respond_with(mock_response)
            .mount(&mock_server)
            .await;
    }

    async fn mock_user(mock_server: &MockServer) {
        let body = Response::<Vec<String>>::default();
        let mock_response = ResponseTemplate::new(200).set_body_json(body);

        Mock::given(method("GET"))
            .and(path("/_db/keti/_api/database/user"))
            .respond_with(mock_response)
            .mount(&mock_server)
            .await;
    }

    async fn mock_list(mock_server: &MockServer) {
        let body = Response::<Vec<String>>::default();
        let mock_response = ResponseTemplate::new(200).set_body_json(body);

        Mock::given(method("GET"))
            .and(path("_api/database"))
            .respond_with(mock_response)
            .mount(&mock_server)
            .await;
    }

    async fn mock_create(mock_server: &MockServer) {
        let mut body = Response::<bool>::default();
        let _ = body.set_code(201);
        let mock_response = ResponseTemplate::new(201).set_body_json(body);

        Mock::given(method("POST"))
            .and(path("_api/database"))
            .and(body_string_contains("test_db"))
            .respond_with(mock_response)
            .mount(&mock_server)
            .await;
    }

    async fn mock_drop(mock_server: &MockServer) {
        let body = Response::<bool>::default();
        let mock_response = ResponseTemplate::new(200).set_body_json(body);

        Mock::given(method("DELETE"))
            .and(path("_api/database/test_db"))
            .respond_with(mock_response)
            .mount(&mock_server)
            .await;
    }

    #[tokio::test]
    async fn test_current() -> Result<()> {
        let mock_server = MockServer::start().await;
        mock_auth(&mock_server).await;
        mock_current(&mock_server).await;

        let conn = default_conn(mock_server.uri()).await?;

        match conn.current().await {
            Ok(res) => {
                assert_eq!(*res.code(), 200);
                assert!(!res.error());
                assert_eq!(res.result().name(), "test");
                assert_eq!(res.result().id(), "123");
                assert!(!res.result().is_system());
                assert_eq!(res.result().path(), "abcdef");
                assert!(res.result().sharding().is_none());
                assert!(res.result().replication_factor().is_none());
                assert!(res.result().write_concern().is_none());
                Ok(())
            }
            Err(e) => Err(e),
        }
    }

    #[tokio::test]
    async fn test_user() -> Result<()> {
        let mock_server = MockServer::start().await;
        mock_auth(&mock_server).await;
        mock_user(&mock_server).await;

        let conn = default_conn(mock_server.uri()).await?;

        match conn.user().await {
            Ok(res) => {
                assert_eq!(*res.code(), 200);
                assert!(!res.error());
                assert!(res.result().len() > 0);
                Ok(())
            }
            Err(e) => Err(e),
        }
    }

    #[tokio::test]
    async fn test_list() -> Result<()> {
        let mock_server = MockServer::start().await;
        mock_auth(&mock_server).await;
        mock_list(&mock_server).await;

        let conn = no_db_conn(mock_server.uri()).await?;

        match conn.list().await {
            Ok(res) => {
                assert_eq!(*res.code(), 200);
                assert!(!res.error());
                assert!(res.result().len() > 0);
                assert!(res.result().contains(&"_system".to_string()));
                Ok(())
            }
            Err(e) => Err(e),
        }
    }

    fn to_test_error(val: String) -> RuarangoError {
        TestError { val }
    }

    #[tokio::test]
    async fn test_create_drop() -> Result<()> {
        let mock_server = MockServer::start().await;
        mock_auth(&mock_server).await;
        mock_create(&mock_server).await;
        mock_drop(&&mock_server).await;

        let conn = no_db_conn(mock_server.uri()).await?;
        let options = OptionsBuilder::default().build().map_err(to_test_error)?;
        let users = UserBuilder::default()
            .username("test")
            .password("test")
            .active(true)
            .build()
            .map_err(to_test_error)?;
        let create = CreateBuilder::default()
            .name("test_db")
            .options(options)
            .users(vec![users])
            .build()
            .map_err(to_test_error)?;

        match conn.create(&create).await {
            Ok(res) => {
                assert_eq!(*res.code(), 201);
                assert!(!res.error());
                assert!(res.result());
            }
            Err(e) => return Err(e),
        }

        match conn.drop("test_db").await {
            Ok(res) => {
                assert_eq!(*res.code(), 200);
                assert!(!res.error());
                assert!(res.result());
                Ok(())
            }
            Err(e) => Err(e),
        }
    }
}
