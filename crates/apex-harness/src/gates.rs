use crate::feature::{Feature, FeatureError, FeatureState};
use crate::state_machine::StateMachineGatekeeper;
use std::process::Command;

pub struct ThreeLayerGate;

#[derive(Debug)]
pub struct VerificationOutcome {
    pub success: bool,
    pub command: String,
    pub stdout: String,
    pub stderr: String,
    pub exit_code: Option<i32>,
}

impl ThreeLayerGate {
    /// Executes the feature's verification command in shell.
    pub fn execute_verification(feature: &Feature) -> VerificationOutcome {
        let cmd = &feature.verification;
        let output = Command::new("sh").arg("-c").arg(cmd).output();

        match output {
            Ok(out) => VerificationOutcome {
                success: out.status.success(),
                command: cmd.clone(),
                stdout: String::from_utf8_lossy(&out.stdout).to_string(),
                stderr: String::from_utf8_lossy(&out.stderr).to_string(),
                exit_code: out.status.code(),
            },
            Err(e) => VerificationOutcome {
                success: false,
                command: cmd.clone(),
                stdout: String::new(),
                stderr: format!("Failed to spawn command: {e}"),
                exit_code: None,
            },
        }
    }

    /// Attempts to certify and transition a feature to Passing.
    pub fn certify_feature(feature: &mut Feature) -> Result<VerificationOutcome, FeatureError> {
        // Ensure state is active before running verification
        if feature.state == FeatureState::NotStarted {
            StateMachineGatekeeper::transition(feature, FeatureState::Active)?;
        }

        let outcome = Self::execute_verification(feature);
        if outcome.success {
            StateMachineGatekeeper::transition(feature, FeatureState::Passing)?;
            let timestamp = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            feature.evidence = Some(format!(
                "Certified at epoch {timestamp} via '{}' (exit code: 0)",
                feature.verification
            ));
        } else {
            StateMachineGatekeeper::transition(feature, FeatureState::Blocked)?;
            feature.evidence = Some(format!(
                "Verification failed via '{}': {}",
                feature.verification,
                outcome.stderr.lines().last().unwrap_or("Non-zero exit")
            ));
        }

        Ok(outcome)
    }
}
