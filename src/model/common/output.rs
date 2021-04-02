// Copyright (c) 2021 ruarango developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

//! Common Output Structs

use crate::db::output::Current;
use getset::Getters;
use serde_derive::{Deserialize, Serialize};
#[cfg(test)]
use {crate::coll::output::Collections, getset::Setters};

/// A base response
#[derive(Clone, Debug, Deserialize, Getters, Serialize)]
#[cfg_attr(test, derive(Setters))]
#[getset(get = "pub")]
#[cfg_attr(test, getset(set = "pub(crate)"))]
pub struct Response<T> {
    /// Is this respone an error?
    error: bool,
    /// The response code, i.e. 200, 404
    code: usize,
    /// The response content
    result: T,
}

impl Default for Response<Current> {
    fn default() -> Self {
        Response {
            error: false,
            code: 200,
            result: Current::default(),
        }
    }
}

#[cfg(test)]
impl Default for Response<Vec<String>> {
    fn default() -> Self {
        Response {
            error: false,
            code: 200,
            result: vec!["_system".to_string(), "test".to_string()],
        }
    }
}

#[cfg(test)]
impl Default for Response<bool> {
    fn default() -> Self {
        Response {
            error: false,
            code: 200,
            result: true,
        }
    }
}

#[cfg(test)]
impl Default for Response<Vec<Collections>> {
    fn default() -> Self {
        Response {
            error: false,
            code: 200,
            result: vec![Collections::default()],
        }
    }
}
