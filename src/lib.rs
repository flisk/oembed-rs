//! This crate provides a simple generic implementation of the [oEmbed
//! specification][1] version 1.0.
//!
//! ## Notes and Caveats
//!
//! * No HTTP client mechanism is included; users of this library must provide an
//!   implementation of [`client::Http`] to functions requiring remote resources.
//! * [Discovery](https://oembed.com/#section4) is not currently supported.
//! * XML responses are not currently supported.
//! * *Some endpoints* — not naming names — will return data that doesn't conform with the
//!   specification; such data can't currently be parsed by this library. No decision
//!   on how to address this has been made yet, and suggestions are welcome.
//!
//! # Examples
//!
//! ```
//! use std::error::Error;
//! use std::borrow::Cow;
//! use oembed::client::*;
//!
//! struct DummyHttp;
//!
//! impl Http for DummyHttp {
//!     fn url_encode<'a>(&mut self, s: &'a str) -> HttpResult<Cow<'a, str>> {
//!         Ok(s.into())
//!     }
//!
//!     fn get(&mut self, _url: &str) -> HttpResult<String> {
//!         Ok("{
//!             \"version\": \"1.0\",
//!             \"type\": \"photo\",
//!             \"width\": 240,
//!             \"height\": 160,
//!             \"title\": \"ZB8T0193\",
//!             \"url\": \"http://farm4.static.flickr.com/3123/2341623661_7c99f48bbf_m.jpg\",
//!             \"author_name\": \"Bees\",
//!             \"author_url\": \"http://www.flickr.com/photos/bees/\",
//!             \"provider_name\": \"Flickr\",
//!             \"provider_url\": \"http://www.flickr.com/\"
//!         }".to_string())
//!     }
//! }
//!
//! let schema = Schema::load_included();
//! let some_url = "http://www.flickr.com/photos/bees/2341623661/";
//! let mut http = DummyHttp {};
//!
//! let response = schema.fetch(&mut http, some_url)
//!     .expect("Missing provider")
//!     .expect("Failed to fetch server response");
//!
//! assert_eq!("ZB8T0193", response.title.unwrap());
//! ```
//!
//! [1]: https://oembed.com/

#[macro_use]
extern crate serde;
extern crate serde_json;

pub mod client;

/// Crate-wide error type
#[derive(Debug)]
pub enum Error {
    /// Returned if [`client::Http::url_encode`] failed.
    HttpUrlEncode(Box<dyn std::error::Error>),

    /// Returned if [`client::Http::get`] failed.
    HttpGet(Box<dyn std::error::Error>),

    /// Returned if parsing a response failed.
    ParseError(serde_json::Error),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

type Result<T> = std::result::Result<T, Error>;

/// oEmbed provider
#[derive(Deserialize, Serialize, Clone, PartialEq, PartialOrd, Hash, Debug, Default)]
pub struct Provider {
    #[serde(rename = "provider_name")]
    pub name: String,

    #[serde(rename = "provider_url")]
    pub url: String,

    pub endpoints: Vec<Endpoint>,
}

/// oEmbed endpoint
#[derive(Deserialize, Serialize, Clone, PartialEq, PartialOrd, Hash, Debug, Default)]
pub struct Endpoint {
    pub url: String,
    pub schemes: Option<Vec<String>>,
    pub formats: Option<Vec<String>>,

    /// Not currently supported
    pub discovery: Option<bool>,
}

/// Endpoint response to an oEmbed request
///
/// See section 2.3.4 of the [oEmbed specification][1].
///
/// [1]: https://oembed.com/
#[derive(Deserialize, Serialize, Clone, PartialEq, PartialOrd, Hash, Debug)]
pub struct Response {
    #[serde(flatten, rename(deserialize = "type"))]
    pub response_type: ResponseType,
    pub version: String,
    pub title: Option<String>,
    pub author_name: Option<String>,
    pub author_url: Option<String>,
    pub provider_name: Option<String>,
    pub provider_url: Option<String>,
    pub cache_age: Option<String>,
    pub thumbnail_url: Option<String>,
    pub thumbnail_width: Option<i32>,
    pub thumbnail_height: Option<i32>,
}

/// Type-specific oEmbed response data
///
/// See section 2.3.4 of the [oEmbed specification][1].
///
/// [1]: https://oembed.com/
#[derive(Deserialize, Serialize, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
#[serde(tag = "type", rename_all(deserialize = "lowercase"))]
pub enum ResponseType {
    Photo {
        url: String,
        width: Option<i32>,
        height: Option<i32>,
    },

    Video {
        html: String,
        width: Option<i32>,
        height: Option<i32>,
    },

    Rich {
        html: String,
        width: Option<i32>,
        height: Option<i32>,
    },

    Link,
}
