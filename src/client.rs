use crate::{Endpoint, Error, Provider, Response, Result};
use std::borrow::Cow;

/// Result type for the [`Http`] trait
pub type HttpResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

/// Defines HTTP operations required by client features
///
/// An implementation of this trait is required for any client operationAn implementation
/// of this trait is required for anything involving HTTP.
pub trait Http {
    /// URL-encode a string so it can be used safely as part of a URL.
    fn url_encode<'a>(&mut self, s: &'a str) -> HttpResult<Cow<'a, str>>;

    /// Retrieve the body of a resource located at `url`.
    fn get(&mut self, url: &str) -> HttpResult<String>;
}

/// Schema containing known oEmbed providers and their endpoints
///
/// The list of providers is currently quite small (~400 elements). For this reason, they
/// are stored in a standard [`Vec`] and looked up with a linear scan.
#[derive(Clone, PartialEq, PartialOrd, Hash, Debug, Default)]
pub struct Schema {
    providers: Vec<Provider>,
}

/// Result of an endpoint search
#[derive(Clone, PartialEq, PartialOrd, Hash, Debug)]
pub struct MatchedEndpoint<'a> {
    /// The provider that the matched endpoint belongs to
    pub provider: &'a Provider,

    /// The matched endpoint
    pub endpoint: &'a Endpoint,

    /// The URL scheme that caused this match
    pub matched_scheme: &'a str,
}

impl Schema {
    /// Load providers from the included schema file
    ///
    /// Schema data is included at build time, so your copy may be out of date. If an
    /// up-to-date schema is required, use [`fetch_latest`][1] or [`fetch_from_url`][2]
    /// instead.
    ///
    /// # Panics
    ///
    /// Panics if the included provider file can't be parsed, which should never happen in
    /// practice.
    ///
    /// [1]: Schema::fetch_latest
    /// [2]: Schema::fetch_from_url
    pub fn load_included() -> Self {
        let json = include_str!("providers.json");
        let providers = serde_json::from_str(&json)
            .expect("Failed to load providers.json. This build of oembed is broken!");

        Self { providers }
    }

    /// Load schema from the public list provided at `https://oembed.com/providers.json`
    pub fn fetch_latest(http: &mut impl Http) -> Result<Self> {
        Self::fetch_from_url(http, "https://oembed.com/providers.json")
    }

    /// Load schema from a specific URL
    pub fn fetch_from_url(http: &mut impl Http, url: &str) -> Result<Self> {
        let s = http.get(url).map_err(|e| Error::HttpGet(e.into()))?;

        let providers = serde_json::from_str(&s).map_err(|e| Error::ParseError(e))?;

        Ok(Self { providers })
    }

    /// Search for the first [`Endpoint`] with a scheme matching `url`
    pub fn match_endpoint(&self, url: &str) -> Option<MatchedEndpoint> {
        for provider in &self.providers {
            for endpoint in &provider.endpoints {
                if let Some(matched_scheme) = endpoint.match_url_scheme(url) {
                    return Some(MatchedEndpoint {
                        provider,
                        endpoint,
                        matched_scheme,
                    });
                }
            }
        }

        None
    }

    /// Fetch an oEmbed response for `url`
    ///
    /// Returns `None` if no endpoint with a scheme matching `url` is found.
    pub fn fetch(&self, http: &mut impl Http, url: &str) -> Option<Result<Response>> {
        self.match_endpoint(url)
            .map(|m| m.endpoint.fetch(http, url))
    }
}

impl Endpoint {
    /// Fetch an oEmbed response for `url` from this endpoint
    pub fn fetch(&self, http: &mut impl Http, url: &str) -> Result<Response> {
        let encoded_url = http.url_encode(url).map_err(|e| Error::HttpUrlEncode(e))?;
        let request_url = format!("{}?format=json&url={}", self.url, encoded_url);

        let s = http.get(&request_url).map_err(|e| Error::HttpGet(e))?;

        serde_json::from_str(&s).map_err(|e| Error::ParseError(e))
    }

    fn match_url_scheme(&self, url: &str) -> Option<&str> {
        self.schemes.as_ref().and_then(|schemes| {
            schemes
                .iter()
                .filter(|s| url_matches_scheme(url, &s))
                .next()
                .map(|s| s.as_str())
        })
    }
}

fn url_matches_scheme(url: &str, scheme: &str) -> bool {
    let mut url_chars = url.chars().peekable();
    let mut scheme_chars = scheme.chars().peekable();

    let mut current_scheme_char = scheme_chars.next();

    while let Some(url_char) = url_chars.next() {
        match current_scheme_char {
            Some('*') if scheme_chars.peek() == url_chars.peek() => (),
            Some('*') => continue,

            Some(scheme_char) if scheme_char == url_char => (),

            Some(_) | None => return false,
        }

        current_scheme_char = scheme_chars.next();
    }

    true
}

#[cfg(test)]
mod tests {
    #[test]
    fn url_matches_scheme() {
        assert_eq!(super::url_matches_scheme("spotify:abc", "spotify:*"), true);

        assert_eq!(
            super::url_matches_scheme("https://youtu.be/v/5mMOsl8qpfc", "https://youtu.be/*"),
            true
        );

        assert_eq!(
            super::url_matches_scheme(
                "https://www.youtube.com/watch?v=5mMOsl8qpfc",
                "https://www.youtube.com/watch*"
            ),
            true
        );

        assert_eq!(
            super::url_matches_scheme(
                "http://www.23hq.com/something/photo/a.jpg",
                "http://www.23hq.com/*/photo/*",
            ),
            true
        )
    }
}
