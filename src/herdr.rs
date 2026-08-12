use std::{fmt, path::Path, process::Output};

use serde::{Deserialize, Serialize};
use tokio::process::Command;

#[derive(Clone, Debug)]
pub struct HerdrClient {
    binary: String,
}

impl HerdrClient {
    pub fn new(binary: String) -> Self {
        Self { binary }
    }

    pub async fn agents(&self) -> Result<Vec<Agent>, HerdrError> {
        let output = Command::new(&self.binary)
            .args(["agent", "list"])
            .output()
            .await
            .map_err(HerdrError::Launch)?;

        parse_agent_list(&output)
    }

    pub async fn focus(&self, pane_id: &str) -> Result<(), HerdrError> {
        if !valid_pane_id(pane_id) {
            return Err(HerdrError::InvalidPane);
        }

        let output = Command::new(&self.binary)
            .args(["agent", "focus", pane_id])
            .output()
            .await
            .map_err(HerdrError::Launch)?;

        if output.status.success() {
            Ok(())
        } else {
            Err(HerdrError::Command(
                String::from_utf8_lossy(&output.stderr).trim().to_owned(),
            ))
        }
    }
}

fn parse_agent_list(output: &Output) -> Result<Vec<Agent>, HerdrError> {
    if !output.status.success() {
        return Err(HerdrError::Command(
            String::from_utf8_lossy(&output.stderr).trim().to_owned(),
        ));
    }

    let envelope: AgentListEnvelope =
        serde_json::from_slice(&output.stdout).map_err(HerdrError::Json)?;
    Ok(envelope.result.agents)
}

fn valid_pane_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b':' | b'_' | b'-'))
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct AgentListEnvelope {
    result: AgentListResult,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct AgentListResult {
    agents: Vec<Agent>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Agent {
    #[serde(rename = "agent")]
    pub kind: String,
    #[serde(rename = "agent_status")]
    pub status: AgentStatus,
    pub cwd: String,
    pub focused: bool,
    pub pane_id: String,
    pub revision: u64,
    pub terminal_title_stripped: Option<String>,
    pub workspace_id: String,
}

impl Agent {
    pub fn task(&self) -> &str {
        self.terminal_title_stripped
            .as_deref()
            .filter(|title| !title.trim().is_empty())
            .unwrap_or("Untitled session")
    }

    pub fn project(&self) -> &str {
        Path::new(&self.cwd)
            .file_name()
            .and_then(|name| name.to_str())
            .filter(|name| !name.is_empty())
            .unwrap_or(&self.workspace_id)
    }

    pub fn matches(&self, query: &str, status: &str) -> bool {
        let status_matches = status == "all" || self.status.as_str() == status;
        let needle = query.trim().to_ascii_lowercase();
        let text_matches = needle.is_empty()
            || self.task().to_ascii_lowercase().contains(&needle)
            || self.project().to_ascii_lowercase().contains(&needle)
            || self.cwd.to_ascii_lowercase().contains(&needle)
            || self.kind.to_ascii_lowercase().contains(&needle);
        status_matches && text_matches
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum AgentStatus {
    Idle,
    Working,
    Blocked,
    Done,
    #[serde(other)]
    Unknown,
}

impl AgentStatus {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Working => "working",
            Self::Blocked => "blocked",
            Self::Done => "done",
            Self::Unknown => "unknown",
        }
    }

    pub const fn label(self) -> &'static str {
        match self {
            Self::Idle => "Idle",
            Self::Working => "Working",
            Self::Blocked => "Blocked",
            Self::Done => "Done",
            Self::Unknown => "Unknown",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct AgentSummary {
    pub total: usize,
    pub working: usize,
    pub blocked: usize,
    pub idle: usize,
}

impl AgentSummary {
    pub fn from_agents(agents: &[Agent]) -> Self {
        Self {
            total: agents.len(),
            working: agents
                .iter()
                .filter(|agent| agent.status == AgentStatus::Working)
                .count(),
            blocked: agents
                .iter()
                .filter(|agent| agent.status == AgentStatus::Blocked)
                .count(),
            idle: agents
                .iter()
                .filter(|agent| agent.status == AgentStatus::Idle)
                .count(),
        }
    }
}

#[derive(Debug)]
pub enum HerdrError {
    Launch(std::io::Error),
    Command(String),
    Json(serde_json::Error),
    InvalidPane,
}

impl fmt::Display for HerdrError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Launch(error) => write!(formatter, "could not launch herdr: {error}"),
            Self::Command(message) if message.is_empty() => {
                formatter.write_str("herdr command failed")
            }
            Self::Command(message) => write!(formatter, "herdr command failed: {message}"),
            Self::Json(error) => write!(formatter, "invalid herdr response: {error}"),
            Self::InvalidPane => formatter.write_str("invalid pane id"),
        }
    }
}

impl std::error::Error for HerdrError {}

#[cfg(test)]
mod tests {
    use std::process::ExitStatus;

    use super::*;

    #[cfg(unix)]
    fn success() -> ExitStatus {
        use std::os::unix::process::ExitStatusExt;
        ExitStatus::from_raw(0)
    }

    #[cfg(unix)]
    fn failure(code: i32) -> ExitStatus {
        use std::os::unix::process::ExitStatusExt;
        ExitStatus::from_raw(code << 8)
    }

    #[test]
    fn parses_and_summarizes_agents() {
        let output = Output {
            status: success(),
            stdout: br#"{"result":{"agents":[{"agent":"claude","agent_status":"working","cwd":"/repo/herdr","focused":true,"pane_id":"wA:p1","revision":3,"terminal_title_stripped":"Ship dashboard","workspace_id":"wA"},{"agent":"codex","agent_status":"blocked","cwd":"/repo/api","focused":false,"pane_id":"wB:p2","revision":1,"terminal_title_stripped":null,"workspace_id":"wB"}]}}"#.to_vec(),
            stderr: Vec::new(),
        };

        let agents = parse_agent_list(&output).expect("valid Herdr JSON");
        assert_eq!(agents[0].project(), "herdr");
        assert_eq!(agents[1].task(), "Untitled session");
        assert_eq!(
            AgentSummary::from_agents(&agents),
            AgentSummary {
                total: 2,
                working: 1,
                blocked: 1,
                idle: 0,
            }
        );
    }

    #[test]
    fn filters_by_status_and_search_text() {
        let agent = Agent {
            kind: "claude".into(),
            status: AgentStatus::Idle,
            cwd: "/repo/herdr-toolkit".into(),
            focused: false,
            pane_id: "wD:p1".into(),
            revision: 2,
            terminal_title_stripped: Some("Install plugin".into()),
            workspace_id: "wD".into(),
        };

        assert!(agent.matches("plugin", "idle"));
        assert!(agent.matches("toolkit", "all"));
        assert!(!agent.matches("plugin", "working"));
    }

    #[test]
    fn reports_stderr_when_agent_list_fails() {
        let output = Output {
            status: failure(1),
            stdout: Vec::new(),
            stderr: b"  herdr: unknown subcommand\n".to_vec(),
        };

        let error = parse_agent_list(&output).expect_err("non-zero exit is an error");
        match error {
            HerdrError::Command(message) => assert_eq!(message, "herdr: unknown subcommand"),
            other => panic!("expected HerdrError::Command, got {other:?}"),
        }
    }

    #[test]
    fn reports_parse_failure_for_malformed_output() {
        let output = Output {
            status: success(),
            stdout: b"not json at all".to_vec(),
            stderr: Vec::new(),
        };

        let error = parse_agent_list(&output).expect_err("malformed JSON is an error");
        assert!(
            matches!(error, HerdrError::Json(_)),
            "expected HerdrError::Json, got {error:?}"
        );
    }

    #[test]
    fn validates_pane_ids_before_command_execution() {
        assert!(valid_pane_id("wM:p1"));
        assert!(!valid_pane_id("wM:p1;rm"));
        assert!(!valid_pane_id(""));
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn reports_stderr_when_focus_fails() {
        use std::os::unix::fs::PermissionsExt;

        let script = std::env::temp_dir().join(format!("herdr-focus-fail-{}", std::process::id()));
        std::fs::write(&script, "#!/bin/sh\necho '  pane not found  ' >&2\nexit 1\n")
            .expect("write stub herdr");
        std::fs::set_permissions(&script, std::fs::Permissions::from_mode(0o755))
            .expect("make stub herdr executable");

        let client = HerdrClient::new(script.to_string_lossy().into_owned());
        let result = client.focus("wA:p1").await;
        // Remove the stub before asserting so a failure cannot leak it into the temp dir.
        std::fs::remove_file(&script).expect("clean up stub herdr");

        match result.expect_err("non-zero exit is an error") {
            HerdrError::Command(message) => assert_eq!(message, "pane not found"),
            other => panic!("expected HerdrError::Command, got {other:?}"),
        }
    }
}
