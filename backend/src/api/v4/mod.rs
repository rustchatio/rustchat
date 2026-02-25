use crate::api::AppState;
use axum::{
    http::{HeaderName, HeaderValue},
    response::IntoResponse,
    Json, Router,
};
use serde_json::json;
use tower_http::set_header::SetResponseHeaderLayer;

pub mod access_control;
pub mod admin;
pub mod ai;
pub mod bots;
pub mod brand;
pub mod calls_plugin;
pub mod categories;
pub mod channel_bookmarks;
pub mod channels;
pub mod cloud;
pub mod cluster;
pub mod commands;
pub mod compliance;
pub mod config_client;
pub mod content_flagging;
pub mod custom_profile;
pub mod data_retention;
pub mod dialogs;
pub mod emoji;
pub mod extractors;
pub mod files;
pub mod groups;
pub mod hooks;
pub mod image;
pub mod imports_exports;
pub mod ip_filtering;
pub mod jobs;
pub mod ldap;
pub mod oauth;
pub mod plugins;
pub mod posts;

pub mod recaps;
pub mod status;
pub mod reports;
pub mod roles;
pub mod saml;
pub mod schemes;
pub mod shared_channels;
pub mod system;
pub mod teams;
pub mod terms_of_service;
pub mod threads;
pub mod uploads;
pub mod usage;
pub mod users;
pub mod websocket;

pub fn router() -> Router<AppState> {
    Router::new()
        .merge(users::router())
        .merge(teams::router())
        .merge(groups::router())
        .merge(channels::router())
        .merge(emoji::router())
        .merge(commands::router())
        .merge(plugins::router())
        .merge(categories::router())
        .merge(posts::router())
        .merge(status::router())
        .merge(channel_bookmarks::router())
        .merge(files::router())
        .merge(system::router())
        .merge(config_client::router())
        .merge(hooks::router())
        .merge(bots::router())
        // Email admin routes (must be before admin::router() to avoid fallback shadowing)
        .merge(crate::api::admin_email::router())
        .merge(admin::router())
        .merge(saml::router())
        .merge(oauth::router())
        .merge(schemes::router())
        .merge(cluster::router())
        .merge(brand::router())
        .merge(ldap::router())
        .merge(access_control::router())
        .merge(content_flagging::router())
        .merge(usage::router())
        .merge(data_retention::router())
        .merge(custom_profile::router())
        .merge(roles::router())
        .merge(cloud::router())
        .merge(jobs::router())
        .merge(recaps::router())
        .merge(compliance::router())
        .merge(shared_channels::router())
        .merge(ai::router())
        .merge(reports::router())
        .merge(ip_filtering::router())
        .merge(imports_exports::router())
        .merge(terms_of_service::router())
        .merge(dialogs::router())
        .merge(uploads::router())
        .merge(threads::router())
        .merge(image::router())
        .merge(calls_plugin::router()) // Mattermost Calls plugin API
        .route(
            "/websocket",
            axum::routing::get(websocket::handle_websocket),
        )
        .fallback(not_implemented)
        .layer(SetResponseHeaderLayer::overriding(
            HeaderName::from_static("x-mm-compat"),
            HeaderValue::from_static("1"),
        ))
}

pub fn mm_not_implemented(
    id: &str,
    message: &str,
    detailed_error: &str,
) -> (axum::http::StatusCode, Json<serde_json::Value>) {
    (
        axum::http::StatusCode::NOT_IMPLEMENTED,
        Json(json!({
            "id": id,
            "message": message,
            "detailed_error": detailed_error,
            "request_id": "",
            "status_code": 501
        })),
    )
}

async fn not_implemented() -> impl IntoResponse {
    mm_not_implemented(
        "api.route.not_implemented.app_error",
        "This API route is not implemented.",
        "The requested Mattermost v4 endpoint is not available in this build.",
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn v4_router_builds_without_overlaps() {
        let _ = router();
    }
}
