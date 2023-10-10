use std::fmt;

use dropshot::{HttpCodedResponse, HttpResponseAccepted, HttpResponseHeaders, HttpResponseOk};
use schemars::JsonSchema;
use serde::Serialize;

use crate::{util::headers::CorsHeaders, ApiError};

pub type ResponseOk<T> = HttpResponseHeaders<HttpResponseOk<T>, CorsHeaders>;
pub type ResponseAccepted<T> = HttpResponseHeaders<HttpResponseAccepted<T>, CorsHeaders>;

#[derive(Copy, Clone)]
pub enum Endpoint {
    GetOne,
    GetLs,
    Post,
    Put,
    Patch,
    Delete,
}

impl From<Endpoint> for http::Method {
    fn from(endpoint: Endpoint) -> Self {
        match endpoint {
            Endpoint::GetOne | Endpoint::GetLs => http::Method::GET,
            Endpoint::Post => http::Method::POST,
            Endpoint::Put => http::Method::PUT,
            Endpoint::Patch => http::Method::PATCH,
            Endpoint::Delete => http::Method::DELETE,
        }
    }
}

impl Endpoint {
    #[allow(clippy::needless_pass_by_value)]
    pub fn err(self, _e: ApiError) -> ApiError {
        // tracing::info!("{api_error}: {e}");
        ApiError::Endpoint(self)
    }

    pub fn pub_response_headers<R, T>(self, body: R) -> HttpResponseHeaders<R, CorsHeaders>
    where
        R: HttpCodedResponse<Body = T>,
        T: JsonSchema + Serialize + Send + Sync,
    {
        HttpResponseHeaders::new(body, self.pub_header())
    }

    pub fn response_headers<R, T>(self, body: R) -> HttpResponseHeaders<R, CorsHeaders>
    where
        R: HttpCodedResponse<Body = T>,
        T: JsonSchema + Serialize + Send + Sync,
    {
        HttpResponseHeaders::new(body, self.header())
    }

    pub fn pub_header(self) -> CorsHeaders {
        CorsHeaders::new_pub(&self)
    }

    pub fn header(self) -> CorsHeaders {
        CorsHeaders::new_auth(&self)
    }
}

impl fmt::Debug for Endpoint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        <Endpoint as fmt::Display>::fmt(self, f)
    }
}

impl fmt::Display for Endpoint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{method}", method = http::Method::from(*self))
    }
}

macro_rules! pub_response_ok {
    ($endpoint:expr, $body:expr) => {
        #[allow(unused_qualifications)]
        Ok($endpoint.pub_response_headers(dropshot::HttpResponseOk($body)))
    };
}

pub(crate) use pub_response_ok;

macro_rules! pub_response_accepted {
    ($endpoint:expr, $body:expr) => {
        Ok($endpoint.pub_response_headers(dropshot::HttpResponseAccepted($body)))
    };
}

pub(crate) use pub_response_accepted;

macro_rules! response_ok {
    ($endpoint:expr, $body:expr) => {
        Ok($endpoint.response_headers(dropshot::HttpResponseOk($body)))
    };
}

pub(crate) use response_ok;

macro_rules! response_accepted {
    ($endpoint:expr, $body:expr) => {
        Ok($endpoint.response_headers(dropshot::HttpResponseAccepted($body)))
    };
}

pub(crate) use response_accepted;
