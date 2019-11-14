# oembed-rs

A generic implementation of [oEmbed](https://oembed.com) 1.0.

[![Latest Version](https://img.shields.io/crates/v/oembed.svg)](https://crates.io/crates/oembed)
[![Documentation](https://docs.rs/oembed/badge.svg)](https://docs.rs/oembed)
[![License](https://img.shields.io/github/license/Flisk/oembed-rs.svg)](LICENSE.txt)

## Currently Unsupported

* Endpoint discovery
* XML responses

## Quick Start

```rust
use std::error::Error;
use std::borrow::Cow;
use oembed::client::*;

struct DummyHttp;

impl Http for DummyHttp {
    fn url_encode<'a>(&mut self, s: &'a str) -> HttpResult<Cow<'a, str>> {
        Ok(s.into())
    }

    fn get(&mut self, _url: &str) -> HttpResult<String> {
        Ok("{
            \"version\": \"1.0\",
            \"type\": \"photo\",
            \"width\": 240,
            \"height\": 160,
            \"title\": \"ZB8T0193\",
            \"url\": \"http://farm4.static.flickr.com/3123/2341623661_7c99f48bbf_m.jpg\",
            \"author_name\": \"Bees\",
            \"author_url\": \"http://www.flickr.com/photos/bees/\",
            \"provider_name\": \"Flickr\",
            \"provider_url\": \"http://www.flickr.com/\"
        }".to_string())
    }
}

let schema = Schema::load_included();
let some_url = "http://www.flickr.com/photos/bees/2341623661/";
let mut http = DummyHttp {};

let response = schema.fetch(&mut http, some_url)
    .expect("Missing provider")
    .expect("Failed to fetch server response");

println!("{:?}", response);
```

## License

The `oembed-rs` crate is under the MIT license, which is included in
[`LICENSE.txt`].

The included provider list ([`src/providers.json`]) is under the MIT
license, which is included in [`LICENSE.providers.txt`].

[`LICENSE.txt`]: LICENSE.txt
[`LICENSE.providers.txt`]: LICENSE.providers.txt
[`src/providers.json`]: src/providers.json
