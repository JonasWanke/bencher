use bencher_json::JsonEmpty;
use dropshot::{endpoint, HttpError, RequestContext};

use crate::{
    context::ApiContext,
    endpoints::{
        endpoint::{pub_response_ok, CorsResponse, ResponseOk},
        Endpoint,
    },
};

#[allow(clippy::unused_async)]
#[endpoint {
    method = OPTIONS,
    path =  "/",
    tags = ["server"]
}]
pub async fn server_root_options(
    _rqctx: RequestContext<ApiContext>,
) -> Result<CorsResponse, HttpError> {
    Ok(Endpoint::cors(&[Endpoint::GetOne]))
}

#[allow(clippy::unused_async)]
#[endpoint {
    method = GET,
    path = "/",
    tags = ["server"]
}]
pub async fn server_root_get(
    _rqctx: RequestContext<ApiContext>,
) -> Result<ResponseOk<JsonEmpty>, HttpError> {
    let endpoint = Endpoint::GetOne;
    pub_response_ok!(endpoint, JsonEmpty {})
}
