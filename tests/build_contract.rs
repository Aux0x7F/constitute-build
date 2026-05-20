use constitute_build::{build_fixture, build_status, default_now, validate_build_fixture};
use constitute_protocol::{BUILD_RUN_STATE_BLOCKED, BUILD_RUN_STATE_SUCCEEDED};

#[test]
fn fixture_validates_build_contract_and_runner_fulfillment() {
    let fixture = build_fixture(default_now(), BUILD_RUN_STATE_SUCCEEDED).expect("fixture builds");
    validate_build_fixture(&fixture).expect("fixture validates");
    assert!(
        fixture
            .contract
            .source_snapshot_ref
            .starts_with("source:snapshot:")
    );
    assert!(
        fixture
            .artifact
            .storage_object_ref
            .starts_with("storage:object:")
    );
}

#[test]
fn blocked_build_is_posture_not_artifact_truth() {
    let fixture = build_fixture(default_now(), BUILD_RUN_STATE_BLOCKED).expect("blocked fixture");
    validate_build_fixture(&fixture).expect("blocked fixture validates");
    assert!(fixture.run.artifact_refs.is_empty());
    assert_eq!(
        fixture.run.blocked_reasons,
        vec!["runner.resource.unavailable"]
    );
}

#[test]
fn status_is_bounded() {
    let status = build_status().expect("status builds");
    assert!(status.build_contract_ref.starts_with("build:contract:"));
    assert!(status.runner_ref.starts_with("runner:instance:"));
}
