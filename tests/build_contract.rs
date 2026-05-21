use constitute_build::{
    build_fixture, build_status, default_build_output_plan, default_build_run_request, default_now,
    reduce_build_run, validate_build_fixture,
};
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
            .as_ref()
            .expect("succeeded fixture has artifact")
            .storage_object_ref
            .starts_with("storage:object:")
    );
    assert_eq!(
        fixture.run.compatibility_refs,
        fixture.contract.compatibility_refs
    );
    assert_eq!(
        fixture.run.release_candidate_refs,
        vec!["release:candidate:cybersec-bootstrap"]
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
    assert!(fixture.run.release_candidate_refs.is_empty());
    assert!(fixture.artifact.is_none());
    assert_eq!(fixture.proof.state, "blocked");
}

#[test]
fn status_is_bounded() {
    let status = build_status().expect("status builds");
    assert!(status.build_contract_ref.starts_with("build:contract:"));
    assert!(status.runner_ref.starts_with("runner:instance:"));
}

#[test]
fn build_reducer_blocks_unavailable_runner() {
    let fixture = build_fixture(default_now(), BUILD_RUN_STATE_SUCCEEDED).expect("fixture builds");
    let mut request = default_build_run_request(default_now());
    request.runner_ref = "runner:instance:missing".to_string();

    let run = reduce_build_run(
        &fixture.contract,
        request,
        default_build_output_plan(
            fixture
                .artifact
                .as_ref()
                .expect("succeeded fixture has artifact"),
            &fixture.proof,
        ),
    )
    .expect("run reduces");

    assert_eq!(run.state, BUILD_RUN_STATE_BLOCKED);
    assert!(run.artifact_refs.is_empty());
    assert!(
        run.blocked_reasons
            .contains(&"build.runner.unavailable".to_string())
    );
}

#[test]
fn build_reducer_blocks_secret_boundary_before_artifacts() {
    let fixture = build_fixture(default_now(), BUILD_RUN_STATE_SUCCEEDED).expect("fixture builds");
    let mut request = default_build_run_request(default_now());
    request.secret_ready = false;

    let run = reduce_build_run(
        &fixture.contract,
        request,
        default_build_output_plan(
            fixture
                .artifact
                .as_ref()
                .expect("succeeded fixture has artifact"),
            &fixture.proof,
        ),
    )
    .expect("run reduces");

    assert_eq!(run.state, BUILD_RUN_STATE_BLOCKED);
    assert!(run.storage_refs.is_empty());
    assert!(
        run.blocked_reasons
            .contains(&"build.secretBoundary.blocked".to_string())
    );
}

#[test]
fn build_reducer_blocks_source_and_recipe_mismatch() {
    let fixture = build_fixture(default_now(), BUILD_RUN_STATE_SUCCEEDED).expect("fixture builds");
    let mut request = default_build_run_request(default_now());
    request.source_snapshot_ref = "source:snapshot:wrong".to_string();
    request.recipe_ref = "build:recipe:wrong".to_string();

    let run = reduce_build_run(
        &fixture.contract,
        request,
        default_build_output_plan(
            fixture
                .artifact
                .as_ref()
                .expect("succeeded fixture has artifact"),
            &fixture.proof,
        ),
    )
    .expect("run reduces");

    assert_eq!(run.state, BUILD_RUN_STATE_BLOCKED);
    assert!(
        run.blocked_reasons
            .contains(&"build.source.mismatch".to_string())
    );
    assert!(
        run.blocked_reasons
            .contains(&"build.recipe.mismatch".to_string())
    );
}
