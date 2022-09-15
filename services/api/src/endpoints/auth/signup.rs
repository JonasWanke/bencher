use std::sync::Arc;

use bencher_json::{auth::Role, jwt::JsonWebToken, JsonEmpty, JsonSignup};
use bencher_rbac::organization::LEADER_ROLE;
use bencher_rbac::organization::MEMBER_ROLE;
use diesel::dsl::count;
use diesel::QueryDsl;
use diesel::RunQueryDsl;
use dropshot::{
    endpoint, HttpError, HttpResponseAccepted, HttpResponseHeaders, RequestContext, TypedBody,
};
use tracing::info;

use crate::endpoints::endpoint::pub_response_accepted;
use crate::endpoints::endpoint::ResponseAccepted;
use crate::endpoints::Endpoint;
use crate::endpoints::Method;
use crate::error::api_error;
use crate::model::organization::InsertOrganization;
use crate::model::organization::QueryOrganization;
use crate::model::user::auth::auth_header_error;
use crate::model::user::auth::map_auth_header_error;
use crate::model::user::auth::INVALID_JWT;
use crate::model::user::organization::InsertOrganizationRole;
use crate::model::user::QueryUser;
use crate::util::cors::CorsResponse;
use crate::ApiError;
use crate::{
    model::user::InsertUser,
    schema,
    util::{cors::get_cors, headers::CorsHeaders, http_error, map_http_error, Context},
};

use super::Resource;

const SIGNUP_RESOURCE: Resource = Resource::Signup;

#[endpoint {
    method = OPTIONS,
    path =  "/v0/auth/signup",
    tags = ["auth"]
}]
pub async fn options(_rqctx: Arc<RequestContext<Context>>) -> Result<CorsResponse, HttpError> {
    Ok(get_cors::<Context>())
}

#[endpoint {
    method = POST,
    path =  "/v0/auth/signup",
    tags = ["auth"]
}]
pub async fn post(
    rqctx: Arc<RequestContext<Context>>,
    body: TypedBody<JsonSignup>,
) -> Result<ResponseAccepted<JsonEmpty>, HttpError> {
    let endpoint = Endpoint::new(SIGNUP_RESOURCE, Method::Post);

    let json = post_inner(rqctx.context(), body.into_inner())
        .await
        .map_err(|e| endpoint.err(e))?;

    pub_response_accepted!(endpoint, json)
}

async fn post_inner(context: &Context, mut json_signup: JsonSignup) -> Result<JsonEmpty, ApiError> {
    let api_context = &mut *context.lock().await;
    let conn = &mut api_context.db_conn;

    let invite = json_signup.invite.take();
    let mut insert_user = InsertUser::from_json(conn, json_signup)?;

    let count = schema::user::table
        .select(count(schema::user::id))
        .first::<i64>(conn)
        .map_err(api_error!())?;
    // The first user to signup is admin
    if count == 0 {
        insert_user.admin = true;
    }

    // Insert user
    diesel::insert_into(schema::user::table)
        .values(&insert_user)
        .execute(conn)
        .map_err(api_error!())?;
    let user_id = QueryUser::get_id(conn, &insert_user.uuid)?;

    let insert_org_role = if let Some(invite) = invite {
        let token_data = invite
            .validate_invite(&api_context.secret_key)
            .map_err(map_auth_header_error!(INVALID_JWT))?;
        let org_claims = token_data
            .claims
            .org()
            .ok_or_else(auth_header_error!(INVALID_JWT))?;

        // Connect the user to the organization with the given role
        let organization_id = QueryOrganization::get_id(conn, org_claims.uuid)?;
        InsertOrganizationRole {
            user_id,
            organization_id,
            // TODO better type casting
            role: match org_claims.role {
                Role::Member => MEMBER_ROLE,
                Role::Leader => LEADER_ROLE,
            }
            .into(),
        }
    } else {
        // Create an organization for the user
        let insert_org = InsertOrganization::from_user(&insert_user);
        diesel::insert_into(schema::organization::table)
            .values(&insert_org)
            .execute(conn)
            .map_err(api_error!())?;
        let organization_id = QueryOrganization::get_id(conn, &insert_org.uuid)?;

        // Connect the user to the organization as a `Leader`
        InsertOrganizationRole {
            user_id,
            organization_id,
            // TODO better type casting
            role: LEADER_ROLE.into(),
        }
    };

    diesel::insert_into(schema::organization_role::table)
        .values(&insert_org_role)
        .execute(conn)
        .map_err(api_error!())?;

    let token = JsonWebToken::new_auth(&api_context.secret_key, insert_user.email.clone())
        .map_err(api_error!())?;

    // TODO log this as trace if SMTP is configured
    info!("Confirm \"{}\" with: {token}", insert_user.email);

    Ok(JsonEmpty::default())
}
