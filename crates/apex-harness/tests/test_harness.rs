use apex_harness::{Feature, FeatureState, StateMachineGatekeeper, ThreeLayerGate};

#[test]
fn test_state_machine_valid_transitions() {
    assert!(StateMachineGatekeeper::can_transition(FeatureState::NotStarted, FeatureState::Active));
    assert!(StateMachineGatekeeper::can_transition(FeatureState::Active, FeatureState::Passing));
    assert!(StateMachineGatekeeper::can_transition(FeatureState::Active, FeatureState::Blocked));
    assert!(StateMachineGatekeeper::can_transition(FeatureState::Blocked, FeatureState::Active));
    assert!(!StateMachineGatekeeper::can_transition(FeatureState::NotStarted, FeatureState::Passing));
}

#[test]
fn test_pass_state_gating_success() {
    let mut feature = Feature {
        id: "TEST_01".to_string(),
        module: "apex-test".to_string(),
        behavior: "Test feature that passes".to_string(),
        verification: "echo 'success'".to_string(),
        state: FeatureState::NotStarted,
        evidence: None,
    };

    let outcome = ThreeLayerGate::certify_feature(&mut feature).unwrap();
    assert!(outcome.success);
    assert_eq!(feature.state, FeatureState::Passing);
    assert!(feature.evidence.is_some());
}

#[test]
fn test_pass_state_gating_failure_blocks_transition() {
    let mut feature = Feature {
        id: "TEST_02".to_string(),
        module: "apex-test".to_string(),
        behavior: "Test feature that fails".to_string(),
        verification: "exit 1".to_string(),
        state: FeatureState::NotStarted,
        evidence: None,
    };

    let outcome = ThreeLayerGate::certify_feature(&mut feature).unwrap();
    assert!(!outcome.success);
    assert_eq!(feature.state, FeatureState::Blocked);
    assert!(feature.evidence.is_some());
}
