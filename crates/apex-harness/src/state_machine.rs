use crate::feature::{Feature, FeatureError, FeatureState};

pub struct StateMachineGatekeeper;

impl StateMachineGatekeeper {
    /// Validates whether a feature is allowed to transition to a new state.
    pub fn can_transition(current: FeatureState, target: FeatureState) -> bool {
        match (current, target) {
            (FeatureState::NotStarted, FeatureState::Active) => true,
            (FeatureState::Active, FeatureState::Blocked) => true,
            (FeatureState::Blocked, FeatureState::Active) => true,
            // Transition to Passing is ONLY permitted through verification gate
            (FeatureState::Active, FeatureState::Passing) => true,
            // Re-certification / verification renewal of already passing feature
            (FeatureState::Passing, FeatureState::Passing) => true,
            // Regression detected by harness
            (FeatureState::Passing, FeatureState::Active) => true,
            (FeatureState::Passing, FeatureState::Blocked) => true,
            _ => false,
        }
    }

    /// Transitions a feature state strictly through validation rules.
    pub fn transition(feature: &mut Feature, target: FeatureState) -> Result<(), FeatureError> {
        if Self::can_transition(feature.state, target) {
            feature.state = target;
            Ok(())
        } else {
            Err(FeatureError::InvalidTransition {
                from: feature.state,
                to: target,
            })
        }
    }
}
