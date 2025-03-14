#![cfg(feature = "plus")]

use bencher_json::JsonServerStats;
use dropshot::{endpoint, HttpError, RequestContext, TypedBody};
use http::StatusCode;
use slog::Logger;

use crate::{
    context::ApiContext,
    endpoints::{
        endpoint::{CorsResponse, Get, Post, ResponseAccepted, ResponseOk},
        Endpoint,
    },
    error::issue_error,
    model::{
        server::QueryServer,
        user::{admin::AdminUser, auth::BearerToken},
    },
};

#[allow(clippy::unused_async)]
#[endpoint {
    method = OPTIONS,
    path =  "/v0/server/stats",
    tags = ["server", "stats"]
}]
pub async fn server_stats_options(
    _rqctx: RequestContext<ApiContext>,
) -> Result<CorsResponse, HttpError> {
    Ok(Endpoint::cors(&[Get.into(), Post.into()]))
}

#[endpoint {
    method = GET,
    path =  "/v0/server/stats",
    tags = ["server", "stats"]
}]
pub async fn server_stats_get(
    rqctx: RequestContext<ApiContext>,
    bearer_token: BearerToken,
) -> Result<ResponseOk<JsonServerStats>, HttpError> {
    let _admin_user = AdminUser::from_token(rqctx.context(), bearer_token).await?;
    let json = get_one_inner(rqctx.context()).await?;
    Ok(Get::auth_response_ok(json))
}

async fn get_one_inner(context: &ApiContext) -> Result<JsonServerStats, HttpError> {
    let conn = &mut *context.conn().await;
    let query_server = QueryServer::get_server(conn)?;
    let is_bencher_cloud = context.is_bencher_cloud();
    query_server.get_stats(conn, is_bencher_cloud)
}

#[endpoint {
    method = POST,
    path =  "/v0/server/stats",
    tags = ["server", "stats"]
}]
pub async fn server_stats_post(
    rqctx: RequestContext<ApiContext>,
    body: TypedBody<JsonServerStats>,
) -> Result<ResponseAccepted<()>, HttpError> {
    post_inner(&rqctx.log, rqctx.context(), body.into_inner()).await?;
    Ok(Post::auth_response_accepted(()))
}

async fn post_inner(
    log: &Logger,
    context: &ApiContext,
    json_server_stats: JsonServerStats,
) -> Result<(), HttpError> {
    let _biller = context.biller()?;
    let conn = &mut *context.conn().await;

    let server_stats = serde_json::to_string_pretty(&json_server_stats).map_err(|e| {
        slog::error!(log, "Failed to serialize stats: {e}");
        issue_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to serialize stats",
            &format!("Failed to serialize stats: {json_server_stats:?}"),
            e,
        )
    })?;
    slog::info!(log, "Self-Hosted Stats: {server_stats:?}");
    QueryServer::send_stats_to_backend(log, conn, &context.messenger, &server_stats, false)?;

    Ok(())
}
