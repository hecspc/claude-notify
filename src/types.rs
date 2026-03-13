use serde::Deserialize;

/// Represents the JSON payload Claude Code sends to hooks via stdin.
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct HookEvent {
    pub session_id: String,
    pub cwd: Option<String>,
    pub hook_event_name: String,
    /// Present for Notification events
    pub notification_type: Option<String>,
    /// Present for permission_prompt notifications
    pub tool_name: Option<String>,
    /// Present for permission_prompt notifications
    pub tool_input: Option<serde_json::Value>,
    /// Present for Stop / TaskCompleted events
    pub stop_hook_active: Option<bool>,
    /// Present for elicitation_dialog notifications
    pub message: Option<String>,
}
