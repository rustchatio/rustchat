//! MCP Server Capabilities and Tool Schema Definitions
//!
//! Defines the capabilities and tool schemas for the MCP server.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

use super::protocol::{ServerCapabilities, ToolsCapability};

/// Tool definition with schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub input_schema: ToolSchema,
    pub requires_approval: bool,
    pub rate_limit_key: String,
}

impl ToolDefinition {
    pub fn new(
        name: impl Into<String>,
        description: impl Into<String>,
        input_schema: ToolSchema,
    ) -> Self {
        let name_str = name.into();
        Self {
            name: name_str.clone(),
            description: description.into(),
            input_schema,
            requires_approval: false,
            rate_limit_key: format!("tool:{}", name_str),
        }
    }

    pub fn with_approval(mut self) -> Self {
        self.requires_approval = true;
        self
    }

    pub fn with_rate_limit_key(mut self, key: impl Into<String>) -> Self {
        self.rate_limit_key = key.into();
        self
    }
}

/// JSON Schema for tool input
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolSchema {
    #[serde(rename = "type")]
    pub schema_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub properties: Option<HashMap<String, SchemaProperty>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

impl ToolSchema {
    pub fn object(properties: HashMap<String, SchemaProperty>, required: Vec<String>) -> Self {
        Self {
            schema_type: "object".to_string(),
            properties: Some(properties),
            required: Some(required),
            description: None,
        }
    }

    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }
}

/// Schema property definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaProperty {
    #[serde(rename = "type")]
    pub property_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "enum")]
    pub enum_values: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub items: Option<Box<SchemaProperty>>,
}

impl SchemaProperty {
    pub fn string(description: impl Into<String>) -> Self {
        Self {
            property_type: "string".to_string(),
            description: Some(description.into()),
            enum_values: None,
            format: None,
            default: None,
            items: None,
        }
    }

    pub fn integer(description: impl Into<String>) -> Self {
        Self {
            property_type: "integer".to_string(),
            description: Some(description.into()),
            enum_values: None,
            format: None,
            default: None,
            items: None,
        }
    }

    pub fn boolean(description: impl Into<String>) -> Self {
        Self {
            property_type: "boolean".to_string(),
            description: Some(description.into()),
            enum_values: None,
            format: None,
            default: None,
            items: None,
        }
    }

    pub fn array(items: SchemaProperty, description: impl Into<String>) -> Self {
        Self {
            property_type: "array".to_string(),
            description: Some(description.into()),
            enum_values: None,
            format: None,
            default: None,
            items: Some(Box::new(items)),
        }
    }

    pub fn object(_properties: HashMap<String, SchemaProperty>, description: impl Into<String>) -> Self {
        Self {
            property_type: "object".to_string(),
            description: Some(description.into()),
            enum_values: None,
            format: None,
            default: None,
            items: None,
        }
    }

    pub fn with_enum(mut self, values: Vec<String>) -> Self {
        self.enum_values = Some(values);
        self
    }

    pub fn with_format(mut self, format: impl Into<String>) -> Self {
        self.format = Some(format.into());
        self
    }

    pub fn with_default(mut self, default: Value) -> Self {
        self.default = Some(default);
        self
    }
}

/// Default server capabilities
pub fn default_server_capabilities() -> ServerCapabilities {
    ServerCapabilities {
        experimental: None,
        logging: Some(serde_json::json!({})),
        prompts: None,
        resources: None,
        tools: Some(ToolsCapability { list_changed: true }),
    }
}

/// Tool registry for available tools
#[derive(Debug, Clone, Default)]
pub struct ToolRegistry {
    tools: HashMap<String, ToolDefinition>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
        }
    }

    pub fn register(&mut self, tool: ToolDefinition) {
        self.tools.insert(tool.name.clone(), tool);
    }

    pub fn get(&self, name: &str) -> Option<&ToolDefinition> {
        self.tools.get(name)
    }

    pub fn list(&self) -> Vec<&ToolDefinition> {
        self.tools.values().collect()
    }

    pub fn contains(&self, name: &str) -> bool {
        self.tools.contains_key(name)
    }

    pub fn len(&self) -> usize {
        self.tools.len()
    }

    pub fn is_empty(&self) -> bool {
        self.tools.is_empty()
    }
}

/// Schema builders for common types
pub mod schemas {
    use super::*;

    /// UUID string format
    pub fn uuid_property(description: impl Into<String>) -> SchemaProperty {
        SchemaProperty::string(description).with_format("uuid")
    }

    /// Date-time string format
    pub fn datetime_property(description: impl Into<String>) -> SchemaProperty {
        SchemaProperty::string(description).with_format("date-time")
    }

    /// Channel ID parameter
    pub fn channel_id() -> SchemaProperty {
        uuid_property("The unique identifier of the channel")
    }

    /// Team ID parameter
    pub fn team_id() -> SchemaProperty {
        uuid_property("The unique identifier of the team")
    }

    /// User ID parameter
    pub fn user_id() -> SchemaProperty {
        uuid_property("The unique identifier of the user")
    }

    /// Message content parameter
    pub fn message_content() -> SchemaProperty {
        SchemaProperty::string("The message content to post")
    }

    /// Limit parameter with default
    pub fn limit(default: i64) -> SchemaProperty {
        SchemaProperty::integer("Maximum number of results to return")
            .with_default(serde_json::json!(default))
    }

    /// Search query parameter
    pub fn search_query() -> SchemaProperty {
        SchemaProperty::string("Search query string")
    }

    /// Thread ID parameter (optional)
    pub fn thread_id() -> SchemaProperty {
        SchemaProperty::string("Optional thread ID to reply to")
    }

    /// File ID parameter
    pub fn file_id() -> SchemaProperty {
        uuid_property("The unique identifier of the file")
    }

    /// Username parameter
    pub fn username() -> SchemaProperty {
        SchemaProperty::string("Username of the user")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_definition() {
        let mut props = HashMap::new();
        props.insert(
            "channel_id".to_string(),
            SchemaProperty::string("Channel ID"),
        );

        let schema = ToolSchema::object(props, vec!["channel_id".to_string()]);
        let tool = ToolDefinition::new("test_tool", "A test tool", schema);

        assert_eq!(tool.name, "test_tool");
        assert!(!tool.requires_approval);
    }

    #[test]
    fn test_tool_registry() {
        let mut registry = ToolRegistry::new();

        let mut props = HashMap::new();
        props.insert(
            "channel_id".to_string(),
            SchemaProperty::string("Channel ID"),
        );

        let schema = ToolSchema::object(props, vec!["channel_id".to_string()]);
        let tool = ToolDefinition::new("test_tool", "A test tool", schema);

        registry.register(tool);

        assert_eq!(registry.len(), 1);
        assert!(registry.contains("test_tool"));
        assert!(!registry.contains("missing"));
    }

    #[test]
    fn test_schema_property_types() {
        let string_prop = SchemaProperty::string("A string");
        assert_eq!(string_prop.property_type, "string");

        let int_prop = SchemaProperty::integer("An integer");
        assert_eq!(int_prop.property_type, "integer");

        let bool_prop = SchemaProperty::boolean("A boolean");
        assert_eq!(bool_prop.property_type, "boolean");
    }
}
