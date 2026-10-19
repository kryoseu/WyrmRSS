use actix_web::web::{Data, Json, Path};
use api_utils::context::WyrmContext;
use database::{
    models::host_credential::{
        HostCredential,
        HostCredentialInsertForm,
        HostCredentialKind,
        HostCredentialUpdateForm,
    },
    newtypes::HostCredentialId,
};
use serde::Deserialize;
use wyrm_rss::http::{custom_header, parse_host_key};
use wyrm_utils::{error::WyrmError, result::WyrmResult};

#[derive(Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct CreateHostCredential {
    kind: HostCredentialKind,
    host: String,
    name: String,
    secret: String,
}

#[derive(Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct UpdateHostCredential {
    kind: Option<HostCredentialKind>,
    host: Option<String>,
    name: Option<String>,
    secret: Option<String>,
}

pub async fn create(
    Json(data): Json<CreateHostCredential>,
    ctx: Data<WyrmContext>,
) -> WyrmResult<Json<HostCredential>> {
    ctx.deny_in_demo()?;

    let parsed_host = parse_host_key(&data.host)?;

    match data.kind {
        HostCredentialKind::CustomHeader => {
            custom_header(&data.name, &data.secret)?;
        }
        HostCredentialKind::BasicAuth if data.name.is_empty() => {
            return Err(WyrmError::InvalidInput("username cannot be empty".into()));
        }
        HostCredentialKind::BasicAuth => {}
    }

    if data.secret.is_empty() {
        return Err(WyrmError::InvalidInput("secret cannot be empty".into()));
    }

    let host_credential = HostCredential::create(
        &ctx.db_pool,
        HostCredentialInsertForm {
            host: parsed_host,
            kind: data.kind,
            name: data.name,
            secret: data.secret,
        },
    )
    .await?;

    Ok(Json(host_credential))
}

pub async fn update(
    path: Path<HostCredentialId>,
    Json(data): Json<UpdateHostCredential>,
    ctx: Data<WyrmContext>,
) -> WyrmResult<Json<HostCredential>> {
    ctx.deny_in_demo()?;

    let id = path.into_inner();

    let parsed_host = data.host.as_deref().map(parse_host_key).transpose()?;

    let existing = HostCredential::get(&ctx.db_pool, id).await?;

    let kind = data.kind.as_ref().unwrap_or(&existing.kind);
    let name = data.name.as_deref().unwrap_or(&existing.name);
    match kind {
        HostCredentialKind::CustomHeader => {
            custom_header(name, data.secret.as_deref().unwrap_or(&existing.secret))?;
        }
        HostCredentialKind::BasicAuth if name.is_empty() => {
            return Err(WyrmError::InvalidInput("username cannot be empty".into()));
        }
        HostCredentialKind::BasicAuth => {}
    }

    let form = HostCredentialUpdateForm {
        id,
        host: parsed_host,
        kind: data.kind,
        name: data.name,
        secret: data.secret.filter(|s| !s.is_empty()),
    };

    // Diesel errors on an empty changeset, so a no-op PATCH returns the row as-is.
    if form.host.is_none() && form.kind.is_none() && form.name.is_none() && form.secret.is_none() {
        return Ok(Json(existing));
    }

    let host_credential = HostCredential::update(&ctx.db_pool, form).await?;

    Ok(Json(host_credential))
}

pub async fn delete(
    path: Path<HostCredentialId>,
    ctx: Data<WyrmContext>,
) -> WyrmResult<Json<HostCredential>> {
    ctx.deny_in_demo()?;
    let host_credential = HostCredential::delete(&ctx.db_pool, path.into_inner()).await?;
    Ok(Json(host_credential))
}
