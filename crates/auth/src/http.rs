//! HTTP client for OIDC discovery and token exchange.
use crate::oidc::{invalid_config, provider_unavailable};
use openidconnect::{HttpRequest, HttpResponse, http};
use std::time::Duration;
use wyrm_utils::result::WyrmResult;

const TIMEOUT: Duration = Duration::from_secs(15);

/// No redirects, so the client secret can't be forwarded to another host.
pub(crate) fn client() -> WyrmResult<reqwest::Client> {
    reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .timeout(TIMEOUT)
        .build()
        .map_err(|_| invalid_config("could not create HTTP client"))
}

/// Sends an openidconnect request through reqwest.
pub(crate) async fn send(
    client: &reqwest::Client,
    request: HttpRequest,
) -> WyrmResult<HttpResponse> {
    let response = client.execute(request.try_into()?).await?;

    let mut builder = http::Response::builder().status(response.status());
    for (name, value) in response.headers() {
        builder = builder.header(name, value);
    }

    let body = response.bytes().await?;
    builder.body(body.to_vec()).map_err(provider_unavailable)
}
