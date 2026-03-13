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
    /// Present for Stop events — true when continuing from a previous stop hook
    pub stop_hook_active: Option<bool>,
    /// Present for Stop events — Claude's final response text
    pub last_assistant_message: Option<String>,
    /// Present for elicitation_dialog notifications
    pub message: Option<String>,
    /// Present for TaskCompleted events
    pub task_id: Option<String>,
    /// Present for TaskCompleted events — title of the task
    pub task_subject: Option<String>,
    /// Present for TaskCompleted events — detailed description
    pub task_description: Option<String>,
    /// Present for TaskCompleted events — name of the teammate
    pub teammate_name: Option<String>,
    /// Present for TaskCompleted events — name of the team
    pub team_name: Option<String>,
}
