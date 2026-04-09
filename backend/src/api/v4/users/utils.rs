use serde::Deserialize;

pub fn normalize_notify_props(value: serde_json::Value) -> serde_json::Value {
    if value.is_null() {
        return serde_json::json!({"desktop": "default", "mark_unread": "all"});
    }

    if let Some(obj) = value.as_object() {
        if obj.is_empty() {
            return serde_json::json!({"desktop": "default", "mark_unread": "all"});
        }
    }

    value
}

pub fn parse_timezone_for_update(timezone: Option<&serde_json::Value>) -> Option<String> {
    let timezone = timezone?;
    let obj = timezone.as_object()?;

    let use_automatic = obj
        .get("useAutomaticTimezone")
        .and_then(|value| {
            value
                .as_bool()
                .or_else(|| value.as_str().map(|raw| raw.eq_ignore_ascii_case("true")))
        })
        .unwrap_or(true);

    let selected = if use_automatic {
        obj.get("automaticTimezone")
    } else {
        obj.get("manualTimezone")
    };

    selected
        .and_then(serde_json::Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string)
}

#[derive(Deserialize)]
#[serde(untagged)]
pub enum UsersByIdsRequest {
    Ids(Vec<String>),
    Wrapped { user_ids: Vec<String> },
}

#[derive(Deserialize)]
#[serde(untagged)]
pub enum UsersByUsernamesRequest {
    Usernames(Vec<String>),
    Wrapped { usernames: Vec<String> },
}
