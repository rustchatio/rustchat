use crate::api::AppState;
use crate::error::{ApiResult, AppError};
use crate::services::membership_policies::apply_auto_membership_for_team_join;
use serde_json::Value;
use std::collections::HashSet;
use uuid::Uuid;

const TEAM_DEFAULT_CHANNELS_KEY: &str = "team_default_channels";
const TEST_FAIL_DEFAULT_CHANNEL_JOIN_KEY: &str = "test_force_default_channel_join_failure";
const TOWN_SQUARE: &str = "town-square";
const OFF_TOPIC: &str = "off-topic";

pub fn normalize_channel_name(name: &str) -> Option<String> {
    let normalized = name.trim().to_ascii_lowercase();
    if normalized.is_empty() {
        return None;
    }
    Some(normalized)
}

pub fn normalize_configured_default_channels(value: &Value) -> Vec<String> {
    let Some(items) = value.as_array() else {
        return Vec::new();
    };

    let mut seen = HashSet::new();
    let mut channels = Vec::new();
    for item in items {
        let Some(name) = item.as_str().and_then(normalize_channel_name) else {
            continue;
        };
        if seen.insert(name.clone()) {
            channels.push(name);
        }
    }

    channels
}

pub fn resolve_team_default_channel_names(configured: &[String]) -> Vec<String> {
    if configured.is_empty() {
        return vec![TOWN_SQUARE.to_string(), OFF_TOPIC.to_string()];
    }

    let mut seen = HashSet::new();
    let mut channels = Vec::new();

    seen.insert(TOWN_SQUARE.to_string());
    channels.push(TOWN_SQUARE.to_string());

    for name in configured {
        if let Some(normalized) = normalize_channel_name(name) {
            if seen.insert(normalized.clone()) {
                channels.push(normalized);
            }
        }
    }

    channels
}

pub async fn get_configured_default_channels(state: &AppState) -> ApiResult<Vec<String>> {
    let experimental: Option<Value> =
        sqlx::query_scalar("SELECT experimental FROM server_config WHERE id = 'default'")
            .fetch_optional(&state.db)
            .await?;

    let configured = experimental
        .as_ref()
        .and_then(|v| v.get(TEAM_DEFAULT_CHANNELS_KEY))
        .map(normalize_configured_default_channels)
        .unwrap_or_default();

    Ok(configured)
}

pub async fn get_team_default_channel_names(state: &AppState) -> ApiResult<Vec<String>> {
    let configured = get_configured_default_channels(state).await?;
    Ok(resolve_team_default_channel_names(&configured))
}

fn default_display_name(channel_name: &str) -> String {
    match channel_name {
        TOWN_SQUARE => "Town Square".to_string(),
        OFF_TOPIC => "Off-Topic".to_string(),
        _ => channel_name
            .split(['-', '_'])
            .filter(|part| !part.is_empty())
            .map(|part| {
                let mut chars = part.chars();
                match chars.next() {
                    Some(first) => {
                        format!(
                            "{}{}",
                            first.to_ascii_uppercase(),
                            chars.as_str().to_ascii_lowercase()
                        )
                    }
                    None => String::new(),
                }
            })
            .collect::<Vec<_>>()
            .join(" "),
    }
}

pub async fn ensure_default_channels_for_team(
    state: &AppState,
    team_id: Uuid,
    creator_id: Uuid,
) -> ApiResult<()> {
    let names = get_team_default_channel_names(state).await?;
    for channel_name in names {
        let display_name = default_display_name(&channel_name);
        sqlx::query(
            r#"
            INSERT INTO channels (team_id, type, name, display_name, creator_id)
            VALUES ($1, 'public'::channel_type, $2, $3, $4)
            ON CONFLICT (team_id, name) DO NOTHING
            "#,
        )
        .bind(team_id)
        .bind(channel_name)
        .bind(display_name)
        .bind(creator_id)
        .execute(&state.db)
        .await?;
    }

    Ok(())
}

pub async fn apply_default_channel_membership_for_team_join(
    state: &AppState,
    team_id: Uuid,
    user_id: Uuid,
) -> ApiResult<()> {
    if state.config.environment == "test" {
        let experimental: Option<Value> =
            sqlx::query_scalar("SELECT experimental FROM server_config WHERE id = 'default'")
                .fetch_optional(&state.db)
                .await?;

        let should_fail = experimental
            .as_ref()
            .and_then(|v| v.get(TEST_FAIL_DEFAULT_CHANNEL_JOIN_KEY))
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        if should_fail {
            return Err(AppError::Internal(
                "Forced default channel auto-join failure for tests".to_string(),
            ));
        }
    }

    // Apply legacy default channel membership
    let default_names = get_team_default_channel_names(state).await?;
    if !default_names.is_empty() {
        sqlx::query(
            r#"
            INSERT INTO channel_members (channel_id, user_id)
            SELECT c.id, $1
            FROM channels c
            WHERE c.team_id = $2
              AND c.type = 'public'::channel_type
              AND c.name = ANY($3::text[])
            ON CONFLICT (channel_id, user_id) DO NOTHING
            "#,
        )
        .bind(user_id)
        .bind(team_id)
        .bind(&default_names)
        .execute(&state.db)
        .await?;
    }

    // Apply auto-membership policies (soft-fail for parity)
    let policy_results =
        apply_auto_membership_for_team_join(state, user_id, team_id, "team_join").await;

    // Log policy application results but don't fail the join if policies fail
    if let Ok(audit_entries) = policy_results {
        let success_count = audit_entries
            .iter()
            .filter(|e| e.status == "success")
            .count();
        let failed_count = audit_entries
            .iter()
            .filter(|e| e.status == "failed")
            .count();

        if failed_count > 0 {
            tracing::warn!(
                "Auto-membership policy partially failed for user {} joining team {}: {} succeeded, {} failed",
                user_id, team_id, success_count, failed_count
            );
        }
    } else if let Err(e) = policy_results {
        tracing::error!(
            "Auto-membership policy application failed for user {} joining team {}: {}",
            user_id,
            team_id,
            e
        );
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        normalize_configured_default_channels, resolve_team_default_channel_names, OFF_TOPIC,
        TOWN_SQUARE,
    };
    use serde_json::json;

    #[test]
    fn configured_defaults_are_normalized_and_deduplicated() {
        let configured = normalize_configured_default_channels(&json!([
            "Announcements",
            " announcements ",
            "",
            10,
            "off-topic"
        ]));

        assert_eq!(configured, vec!["announcements", "off-topic"]);
    }

    #[test]
    fn fallback_includes_town_square_and_off_topic() {
        let resolved = resolve_team_default_channel_names(&[]);
        assert_eq!(resolved, vec![TOWN_SQUARE, OFF_TOPIC]);
    }

    #[test]
    fn custom_defaults_always_include_town_square_once() {
        let resolved = resolve_team_default_channel_names(&[
            "off-topic".to_string(),
            "town-square".to_string(),
            "engineering".to_string(),
        ]);

        assert_eq!(resolved, vec!["town-square", "off-topic", "engineering"]);
    }
}
