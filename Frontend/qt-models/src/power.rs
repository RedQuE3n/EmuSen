//! Linux power actions (Sleep, Restart, Shut down) behind an injectable runner so tests never call systemctl.

use std::process::Command;

/// System power actions offered by Eclipse's Power dialog (board #15).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    Sleep,
    Restart,
    ShutDown,
}

impl Action {
    /// Parses a HomeCard action key into a system power action.
    pub fn from_key(key: &str) -> Option<Self> {
        match key {
            crate::home_card::POWER_SLEEP => Some(Action::Sleep),
            crate::home_card::POWER_RESTART => Some(Action::Restart),
            crate::home_card::POWER_SHUTDOWN => Some(Action::ShutDown),
            _ => None,
        }
    }

    /// The systemd verb passed to `systemctl --no-ask-password`.
    pub fn verb(self) -> &'static str {
        match self {
            Action::Sleep => "suspend",
            Action::Restart => "reboot",
            Action::ShutDown => "poweroff",
        }
    }

    /// Human-readable action name for failure notices.
    pub fn label(self) -> &'static str {
        match self {
            Action::Sleep => "Sleep",
            Action::Restart => "Restart",
            Action::ShutDown => "Shut down",
        }
    }

    /// Whether the running game session should close (vs stay suspended after saving) before the action.
    pub fn closes_session(self) -> bool {
        matches!(self, Action::Restart | Action::ShutDown)
    }
}

/// Injectable executor for system power verbs so unit and integration tests never invoke real systemctl.
pub trait Runner: Send + Sync {
    fn run(&self, verb: &str) -> Result<(), String>;
}

/// Production runner using `systemctl --no-ask-password <verb>`.
pub struct SystemctlRunner;

impl Runner for SystemctlRunner {
    fn run(&self, verb: &str) -> Result<(), String> {
        let out = Command::new("systemctl").args(["--no-ask-password", verb]).output().map_err(|e| format!("systemctl could not start: {e}"))?;
        if out.status.success() {
            return Ok(());
        }
        let err = String::from_utf8_lossy(&out.stderr).trim().to_owned();
        if err.is_empty() {
            Err(format!("systemctl exited with {}", out.status))
        } else {
            Err(err)
        }
    }
}

/// Formats a power action failure as heading + body for the Eclipse Notice banner.
pub fn failed_notice(action: Action, why: &str) -> String {
    format!("{} could not run\n{why}", action.label())
}

/// Saves any active game first, aborting without calling `runner` if saving fails, then runs the power verb.
pub fn execute<F>(action: Action, runner: &dyn Runner, save_game: F) -> Result<(), String>
where
    F: FnOnce(bool) -> Result<(), String>,
{
    save_game(action.closes_session()).map_err(|why| format!("{} cancelled\nCould not save the suspended game: {why}", action.label()))?;
    runner.run(action.verb()).map_err(|why| failed_notice(action, &why))
}
