use crate::app_state::AppState;
use crate::db;
use crate::ui;

use askama::Template;
use axum::extract::{Query, State};
use axum::http::header::SET_COOKIE;
use axum::http::StatusCode;
use axum::response::{AppendHeaders, Html, IntoResponse, Redirect, Response};
use axum_extra::extract::cookie::{Cookie, SameSite};
use axum_extra::routing::TypedPath;
use chrono::{self, Utc};
use openidconnect::{
    core::CoreAuthenticationFlow, AuthorizationCode, CsrfToken, Nonce, Scope, TokenResponse,
};
use serde::Deserialize;
use time::Duration;
use tracing::instrument;
use tracing::{debug, error, info};

#[derive(TypedPath, Debug, Clone, Copy)]
#[typed_path("/auth/status")]
pub struct StatusPath;

#[instrument(level = "trace", ret)]
pub async fn get_status_handler(
    _: StatusPath,
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
) -> Result<Response, Response> {
    let cookie = headers
        .get("Cookie")
        .and_then(|header_value| header_value.to_str().ok())
        .and_then(|cookie_str| {
            Cookie::split_parse(cookie_str)
                .filter_map(Result::ok)
                .find(|cookie| cookie.name() == "session_id")
        })
        .ok_or_else(|| {
            debug!("Couldn't find session_id cookie");
            Html(ui::LoginButton.render().unwrap()).into_response()
        })?;

    let user_session = db::get_user_session(&state.db_pool, cookie.value())
        .await
        .map_err(|err| {
            error!(
                "Failed to get the user session from the database for session_id '{}': {err}",
                cookie.value()
            );
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Html(ui::LoginButton.render().unwrap()),
            )
                .into_response()
        })?;

    if let Some(user_session) = user_session {
        let claims = user_session
            .id_token
            .claims(
                &state.openid_client.id_token_verifier(),
                &user_session.nonce,
            )
            .map_err(|err| {
                error!("Nonce verification failed: {err}");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Html(ui::LoginButton.render().unwrap()),
                )
                    .into_response()
            })?;
        if claims.expiration() > Utc::now() {
            Ok(Html(ui::LogoutButton.render().unwrap()).into_response())
        } else {
            debug!("User session {} has expired", user_session.id);
            Ok(Html(ui::LoginButton.render().unwrap()).into_response())
        }
    } else {
        Ok(Html(ui::LoginButton.render().unwrap()).into_response())
    }
}

#[derive(TypedPath, Debug, Clone, Copy)]
#[typed_path("/auth/login")]
pub struct LoginPath;

#[instrument(level = "trace", ret)]
pub async fn get_login_handler(_: LoginPath, state: State<AppState>) -> Response {
    let (auth_url, csrf_token, nonce) = state
        .openid_client
        .authorize_url(
            CoreAuthenticationFlow::AuthorizationCode,
            CsrfToken::new_random,
            Nonce::new_random,
        )
        .add_scope(Scope::new("openid".to_string()))
        .url();

    match db::new_authentication_session(&state.db_pool, &csrf_token, &nonce).await {
        Err(err) => {
            error!("Failed to create a new authentication session: {err}");
            Html(
                ui::ErrorMessage {
                    message: "Failed to start login process, please try again",
                }
                .render()
                .unwrap(),
            )
            .into_response()
        }
        Ok(_) => AppendHeaders([("hx-redirect", auth_url.as_str())]).into_response(),
    }
}

#[derive(TypedPath, Debug, Clone, Copy)]
#[typed_path("/auth/callback")]
pub struct OAuthCallbackPath;

#[derive(Debug, Deserialize)]
pub struct OAuthCallbackParams {
    code: String,
    state: String,
}

#[instrument(level = "trace", ret)]
pub async fn oauth_callback_handler(
    _: OAuthCallbackPath,
    Query(params): Query<OAuthCallbackParams>,
    State(state): State<AppState>,
) -> Result<Response, Response> {
    let received_csrf_token = CsrfToken::new(params.state);
    let maybe_nonce = db::get_nonce_by_csrf_token(&state.db_pool, &received_csrf_token)
        .await
        .map_err(|err| {
            error!("Database error fetching nonce: {}", err);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Html(
                    ui::ErrorMessage {
                        message: "We couldn’t process your login right now. Please try again.",
                    }
                    .render()
                    .unwrap(),
                ),
            )
                .into_response()
        })?;

    let nonce = maybe_nonce.ok_or_else(|| {
        debug!("Couldn't find a login session for the CSRF token, the login session has expired or is invalid.");
        (
            StatusCode::BAD_REQUEST,
            Html(ui::ErrorMessage { message: "Your login session has expired or is invalid. Please try again.." }.render().unwrap()
            ),
        )
            .into_response()
    })?;

    let code_token_request = state
        .openid_client
        .exchange_code(AuthorizationCode::new(params.code))
        .map_err(|err| {
            error!("Code exchange error: {}", err);
            (
                StatusCode::BAD_REQUEST,
                Html(
                    ui::ErrorMessage {
                        message: "There was an issue with the login request. Please try again.",
                    }
                    .render()
                    .unwrap(),
                ),
            )
                .into_response()
        })?;

    let token_response = code_token_request
        .request_async(&state.http_client)
        .await
        .map_err(|err| {
            error!("Token request error: {}", err);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Html(ui::ErrorMessage { message: "We couldn’t connect to the authorization server. Please try again later." }.render().unwrap()
                ),
            )
                .into_response()
        })?;

    let id_token = token_response.id_token().ok_or_else(|| {
        error!("No ID token in response");
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Html(ui::ErrorMessage { message: "The authorization server didn’t provide a valid response. Please try again." }.render().unwrap()
            ),
        )
            .into_response()
    })?;

    let id_token_verifier = state.openid_client.id_token_verifier();
    let claims = id_token.claims(&id_token_verifier, &nonce).map_err(|err| {
        error!("Nonce verification failed: {err}");
        (
            StatusCode::BAD_REQUEST,
            Html(
                ui::ErrorMessage {
                    message: "Login verification failed. Please start the login process again.",
                }
                .render()
                .unwrap(),
            ),
        )
            .into_response()
    })?;

    info!(
        "User {} with e-mail address {} has authenticated successfully",
        claims.subject().as_str(),
        claims
            .email()
            .map(|email| email.as_str())
            .unwrap_or("<not provided>"),
    );

    info!("Creating user session for the authenticated user...");
    let expires_at = Utc::now() + chrono::Duration::days(3);
    let session = db::new_user_session(&state.db_pool, id_token, &nonce, &expires_at)
        .await
        .map_err(|err| {
            error!("Failed to create a user session: {err}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Html(
                    ui::ErrorMessage {
                        message: "Failed to create a user session cookie.",
                    }
                    .render()
                    .unwrap(),
                ),
            )
                .into_response()
        })?;

    let cookie = Cookie::build(("session_id", session.id.to_string()))
        .http_only(true)
        .secure(true)
        .same_site(SameSite::Strict)
        .max_age(Duration::days(3))
        .path("/")
        .build();

    if let Err(err) =
        db::delete_authentication_sessios_for(&state.db_pool, &received_csrf_token).await
    {
        error!(
            "Database error deleting the authentication session: {}",
            err
        );
    }

    let redirect = Redirect::to("/");
    let mut response = redirect.into_response();
    response.headers_mut().append(
        SET_COOKIE,
        cookie
            .to_string()
            .parse()
            .expect("Failed to convert the identity token cookie string to a HeaderValue"),
    );
    response.headers_mut().append(
        "hx-redirect",
        state
            .host_url
            .as_str()
            .parse()
            .expect("Failed to convert the host url redirect string to a HeaderValue"),
    );
    Ok(response)
}

#[derive(TypedPath, Debug, Clone, Copy)]
#[typed_path("/auth/logout")]
pub struct LogoutPath;

#[instrument(level = "trace", ret)]
pub async fn get_logout_handler(_: LogoutPath, State(state): State<AppState>) -> Response {
    let expired_cookie = Cookie::build(("session_id", ""))
        .http_only(true)
        .secure(true)
        .same_site(SameSite::Strict)
        .expires(None)
        .max_age(Duration::seconds(0)) // Expires immediately
        .path("/")
        .build();

    let cookie_value = expired_cookie.to_string().parse().unwrap();
    let redirect_value = "/".parse().unwrap();

    // Do a client side redirect instead of a server side redirect i.e.
    // let mut response = axum::response::Redirect::to("/").into_response();
    // Because htmx would render the redirect value in the status section
    let mut response = Html("").into_response();
    response.headers_mut().append(SET_COOKIE, cookie_value);
    response.headers_mut().append("hx-redirect", redirect_value);
    response
}
