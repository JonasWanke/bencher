use bencher_json::{project::threshold::JsonStatistic, ResourceId, StatisticUuid};
use diesel::{ExpressionMethods, JoinOnDsl, QueryDsl, RunQueryDsl, SelectableHelper};
use dropshot::{endpoint, HttpError, Path, RequestContext};
use schemars::JsonSchema;
use serde::Deserialize;

use crate::{
    context::ApiContext,
    endpoints::{
        endpoint::{pub_response_ok, response_ok, ResponseOk},
        Endpoint,
    },
    model::project::{threshold::statistic::QueryStatistic, QueryProject},
    model::user::auth::AuthUser,
    schema,
    util::cors::{get_cors, CorsResponse},
    ApiError,
};

#[derive(Deserialize, JsonSchema)]
pub struct ProjStatisticParams {
    pub project: ResourceId,
    pub statistic: StatisticUuid,
}

#[allow(clippy::unused_async)]
#[endpoint {
    method = OPTIONS,
    path =  "/v0/projects/{project}/statistics/{statistic}",
    tags = ["projects", "statistics"]
}]
pub async fn proj_statistic_options(
    _rqctx: RequestContext<ApiContext>,
    _path_params: Path<ProjStatisticParams>,
) -> Result<CorsResponse, HttpError> {
    Ok(get_cors::<ApiContext>())
}

#[endpoint {
    method = GET,
    path =  "/v0/projects/{project}/statistics/{statistic}",
    tags = ["projects", "statistics"]
}]
pub async fn proj_statistic_get(
    rqctx: RequestContext<ApiContext>,
    path_params: Path<ProjStatisticParams>,
) -> Result<ResponseOk<JsonStatistic>, HttpError> {
    let auth_user = AuthUser::new(&rqctx).await.ok();
    let endpoint = Endpoint::GetOne;

    let json = get_one_inner(
        rqctx.context(),
        path_params.into_inner(),
        auth_user.as_ref(),
    )
    .await
    .map_err(|e| {
        if let ApiError::HttpError(e) = e {
            e
        } else {
            endpoint.err(e).into()
        }
    })?;

    if auth_user.is_some() {
        response_ok!(endpoint, json)
    } else {
        pub_response_ok!(endpoint, json)
    }
}

async fn get_one_inner(
    context: &ApiContext,
    path_params: ProjStatisticParams,
    auth_user: Option<&AuthUser>,
) -> Result<JsonStatistic, ApiError> {
    let conn = &mut *context.conn().await;

    let query_project =
        QueryProject::is_allowed_public(conn, &context.rbac, &path_params.project, auth_user)?;

    schema::statistic::table
        .inner_join(
            schema::threshold::table.on(schema::statistic::threshold_id.eq(schema::threshold::id)),
        )
        .filter(schema::threshold::project_id.eq(query_project.id))
        .filter(schema::statistic::uuid.eq(path_params.statistic))
        .select(QueryStatistic::as_select())
        .first(conn)
        .map_err(ApiError::from)?
        .into_json(conn)
}
