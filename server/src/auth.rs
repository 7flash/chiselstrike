// SPDX-FileCopyrightText: © 2021 ChiselStrike <info@chiselstrike.com>

use crate::api::{ApiService, Body};
use crate::db::SqlValue;
use crate::query::engine::{JsonObject, SqlWithArguments};
use crate::runtime;
use crate::types::{ObjectType, Type, OAUTHUSER_TYPE_NAME};
use anyhow::anyhow;
use futures::{Future, FutureExt};
use hyper::{header, Request, Response, StatusCode};
use serde_json::json;
use sqlx::Row;
use std::pin::Pin;
use std::sync::Arc;

const USERPATH: &str = "/__chiselstrike/auth/user/";

fn redirect(link: &str) -> Response<Body> {
    let bd: Body = format!("Redirecting to <a href='{}'>{}</a>\n", link, link).into();
    Response::builder()
        .status(StatusCode::TEMPORARY_REDIRECT)
        .header(header::LOCATION, link)
        .body(bd)
        .unwrap()
}

fn bad_request(msg: String) -> Response<Body> {
    let bd: Body = (msg + "\n").into();
    Response::builder()
        .status(StatusCode::BAD_REQUEST)
        .body(bd)
        .unwrap()
}

fn get_oauth_user_type() -> anyhow::Result<Arc<ObjectType>> {
    match runtime::get()
        .type_system
        .lookup_builtin_type(OAUTHUSER_TYPE_NAME)
    {
        Ok(Type::Object(t)) => Ok(t),
        _ => anyhow::bail!("Internal error: type {} not found", OAUTHUSER_TYPE_NAME),
    }
}

async fn insert_user_into_db(username: &str) -> anyhow::Result<()> {
    let oauth_user_type = get_oauth_user_type()?;
    let mut user = JsonObject::new();
    user.insert("username".into(), json!(username));
    let query_engine = { runtime::get().query_engine.clone() };

    query_engine.add_row(&oauth_user_type, &user).await?;
    Ok(())
}

pub(crate) async fn get_userid_from_db(username: String) -> anyhow::Result<String> {
    let oauth_user_type = get_oauth_user_type()?;
    let qeng = { runtime::get().query_engine.clone() };
    Ok(qeng
        .fetch_one(SqlWithArguments {
            sql: format!(
                "SELECT id FROM {} WHERE username=$1",
                oauth_user_type.backing_table()
            ),
            args: vec![SqlValue::String(username)],
        })
        .await?
        .get("id"))
}

fn handle_callback(
    req: Request<hyper::Body>,
) -> Pin<Box<dyn Future<Output = Result<Response<Body>, anyhow::Error>>>> {
    // TODO: Grab state out of the request, validate it, and grab the referrer URL out of it.
    async move {
        let params = req.uri().query();
        if params.is_none() {
            return Ok(bad_request("Callback error: parameter missing".into()));
        }
        let username = params.unwrap().strip_prefix("user=");
        if username.is_none() {
            return Ok(bad_request(
                "Callback error: parameter value missing".into(),
            ));
        }
        let username = username.unwrap();
        if username.is_empty() {
            return Ok(bad_request("Callback error: parameter value empty".into()));
        }
        let username = urldecode::decode(username.into());
        insert_user_into_db(&username).await?;
        let meta = runtime::get().meta.clone();
        Ok(redirect(&format!(
            // TODO: redirect to the URL from state.
            "http://localhost:3000/profile?chiselstrike_token={}",
            meta.new_session_token(&username).await?
        )))
    }
    .boxed_local()
}

fn lookup_user(
    req: Request<hyper::Body>,
) -> Pin<Box<dyn Future<Output = Result<Response<Body>, anyhow::Error>>>> {
    async move {
        let token = req
            .uri()
            .path()
            .strip_prefix(USERPATH)
            .ok_or_else(|| anyhow!("lookup_user on wrong URL"))?;
        let meta = runtime::get().meta.clone();
        let bd: Body = meta.get_username(token).await?.into();
        let resp = Response::builder().status(StatusCode::OK).body(bd).unwrap();
        Ok(resp)
    }
    .boxed_local()
}

pub(crate) fn init(api: &mut ApiService) {
    api.add_route(
        "/__chiselstrike/auth/callback".into(),
        Arc::new(handle_callback),
    );
    api.add_route(USERPATH.into(), Arc::new(lookup_user));
}

pub(crate) async fn get_username(req: &Request<hyper::Body>) -> anyhow::Result<Option<String>> {
    match req.headers().get("ChiselStrikeToken") {
        Some(token) => {
            let meta = { crate::runtime::get().meta.clone() };
            Ok(meta.get_username(token.to_str()?).await.ok())
        }
        None => Ok(None),
    }
}
