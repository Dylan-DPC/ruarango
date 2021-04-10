// Copyright (c) 2021 ruarango developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

//! Common functionality for Integration Tests

use anyhow::{anyhow, Result};
use ruarango::{AsyncKind, Connection, ConnectionBuilder, Either, Job};
use serde::{de::DeserializeOwned, Serialize};

macro_rules! int_test_sync {
    () => {};
    ($res:ident; $conn:ident; $code:literal; $name:ident, $conn_ty:ident, $api:ident($($args:expr),*) => $asserts: block) => {
        #[tokio::test]
        async fn $name() -> Result<()> {
            let $conn = $conn_ty().await?;
            let res = $conn.$api($($args),*).await?;
            let $res = common_db::process_sync_result(res)?;
            $asserts

            Ok(())
        }
    };
    ($res:ident; $conn:ident; $code:literal; $($tail:tt)*) => {
        int_test_sync!($res; $conn; $code; $($tail)*);
    };
    ($res:ident; $conn:ident; $($tail:tt)*) => {
        int_test_sync!($res; $conn; 200; $($tail)*);
    };
    ($res:ident; $($tail:tt)*) => {
        int_test_sync!($res; conn; 200; $($tail)*);
    };
}

macro_rules! int_test_async {
    () => {};
    ($res:ident; $conn:ident; $kind:ty; $name:ident, $conn_ty:ident, $api:ident($($args:expr),*) => $asserts: block) => {
        #[tokio::test]
        async fn $name() -> Result<()> {
            let $conn = $conn_ty().await?;
            let res = $conn.$api($($args),*).await?;
            let $res: $kind = common_db::process_async_result(res, &$conn).await?;
            $asserts

            Ok(())
        }
    };
    ($res:ident; $conn:ident; $kind:ty; $($tail:tt)*) => {
        int_test_async!($res; $conn; $kind; $($tail)*);
    };
    ($res:ident; $kind:ty; $($tail:tt)*) => {
        int_test_async!($res; conn; $kind; $($tail)*);
    };
}

pub(crate) async fn conn_ruarango_async() -> Result<Connection> {
    ConnectionBuilder::default()
        .url(env!("ARANGODB_URL"))
        .username("ruarango")
        .password(env!("ARANGODB_RUARANGO_PASSWORD"))
        .database("ruarango")
        .async_kind(AsyncKind::Store)
        .build()
        .await
}

pub(crate) async fn conn_root_system_async() -> Result<Connection> {
    ConnectionBuilder::default()
        .url(env!("ARANGODB_URL"))
        .username("root")
        .password(env!("ARANGODB_ROOT_PASSWORD"))
        .async_kind(AsyncKind::Store)
        .build()
        .await
}

pub(crate) async fn process_async_result<T>(res: Either<T>, conn: &Connection) -> Result<T>
where
    T: DeserializeOwned + Serialize + Send + Sync,
{
    assert!(res.is_left());
    let job_info = res.left_safe()?;
    assert_eq!(*job_info.code(), 202);
    let id = job_info
        .id()
        .as_ref()
        .ok_or_else(|| anyhow!("invalid job id"))?;

    let mut status = conn.status(id).await?;
    assert!(status == 200 || status == 204);

    while status != 200 {
        std::thread::sleep(std::time::Duration::from_millis(500));
        status = conn.status(id).await?;
    }

    Ok(conn.fetch(id).await?)
}

pub(crate) fn process_sync_result<T>(res: Either<T>) -> Result<T>
where
    T: DeserializeOwned + Serialize + Send + Sync,
{
    assert!(res.is_right());
    Ok(res.right_safe()?)
}
