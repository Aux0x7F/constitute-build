use constitute_build::{
    append_build_run, build_fixture, build_state_status, build_status, default_build_output_plan,
    default_build_run_request, default_build_state, default_now, reduce_build_run,
    validate_build_fixture, validate_build_state,
};
use constitute_protocol::{
    BUILD_RUN_STATE_BLOCKED, BUILD_RUN_STATE_SUCCEEDED, FABRIC_MEMBER_CONTRIBUTION_BLOCKED,
    FABRIC_MEMBER_CONTRIBUTION_RUNNING, FABRIC_MEMBER_ROLE_BUILD_PROCESSOR,
    RUNNER_OPERATION_STATE_BLOCKED, RUNNER_OPERATION_STATE_SUCCEEDED,
};

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
        fixture.run.content_index_refs,
        fixture.contract.content_index_refs
    );
    assert_eq!(
        fixture.run.processor_contract_refs,
        fixture.contract.processor_contract_refs
    );
    assert_eq!(
        fixture.run.processor_role_refs,
        fixture.contract.processor_role_refs
    );
    assert_eq!(
        fixture.run.source_operation_refs,
        fixture.contract.source_operation_refs
    );
    assert_eq!(
        fixture.proof.source_operation_refs,
        fixture.contract.source_operation_refs
    );
    assert_eq!(
        fixture.proof.processor_contract_refs,
        fixture.contract.processor_contract_refs
    );
    assert_eq!(
        fixture.proof.processor_role_refs,
        fixture.contract.processor_role_refs
    );
    assert_eq!(fixture.run.project_refs, fixture.contract.project_refs);
    assert_eq!(fixture.run.work_item_refs, fixture.contract.work_item_refs);
    assert_eq!(
        fixture.run.release_candidate_refs,
        vec!["release:candidate:build-runner-proof"]
    );
    assert_eq!(
        fixture.runner_operation.state,
        RUNNER_OPERATION_STATE_SUCCEEDED
    );
    assert_eq!(
        fixture.runner_operation.contract_ref,
        fixture.contract.build_contract_ref
    );
    assert_eq!(
        fixture.runner_operation.output_refs,
        vec![
            "build:artifact:module",
            "build:proof:build-runner-proof",
            "release:candidate:build-runner-proof"
        ]
    );
    assert_eq!(
        fixture.host_fabric_contribution.role,
        FABRIC_MEMBER_ROLE_BUILD_PROCESSOR
    );
    assert_eq!(
        fixture.host_fabric_contribution.state,
        FABRIC_MEMBER_CONTRIBUTION_RUNNING
    );
    assert_eq!(
        fixture.host_fabric_contribution.subject_ref,
        fixture.run.run_ref
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
    assert_eq!(
        fixture.runner_operation.state,
        RUNNER_OPERATION_STATE_BLOCKED
    );
    assert_eq!(
        fixture.runner_operation.blocked_reasons,
        vec!["runner.resource.unavailable"]
    );
    assert_eq!(
        fixture.host_fabric_contribution.state,
        FABRIC_MEMBER_CONTRIBUTION_BLOCKED
    );
    assert_eq!(
        fixture.host_fabric_contribution.blocked_reasons,
        vec!["runner.resource.unavailable"]
    );
}

#[test]
fn status_is_bounded() {
    let status = build_status().expect("status builds");
    assert!(status.build_contract_ref.starts_with("build:contract:"));
    assert!(status.runner_ref.starts_with("runner:instance:"));
    assert!(status.runner_operation_ref.starts_with("runner:operation:"));
    assert_eq!(status.source_operation_ref_count, 2);
    assert_eq!(status.processor_contract_ref_count, 1);
    assert_eq!(status.processor_role_ref_count, 1);
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

#[test]
fn build_state_persists_runner_operation_and_artifact_posture() {
    let mut state = default_build_state(default_now()).expect("state builds");
    validate_build_state(&state).expect("state validates");
    let initial = build_state_status(&state).expect("status builds");
    assert_eq!(initial.runner_operation_count, 1);
    assert_eq!(initial.host_fabric_contribution_count, 1);

    let fixture = build_fixture(default_now(), BUILD_RUN_STATE_SUCCEEDED).expect("fixture builds");
    let request = default_build_run_request(default_now() + 100);
    let artifact = fixture.artifact.as_ref().expect("fixture has artifact");
    let outcome = append_build_run(
        &mut state,
        request,
        default_build_output_plan(artifact, &fixture.proof),
    )
    .expect("append succeeds");

    assert_eq!(
        outcome.runner_operation.state,
        RUNNER_OPERATION_STATE_SUCCEEDED
    );
    assert_eq!(state.runs.len(), 2);
    assert_eq!(state.artifacts.len(), 2);
    assert_eq!(state.proofs.len(), 2);
    assert_eq!(state.runner_operations.len(), 2);
    assert_eq!(state.host_fabric_contributions.len(), 2);
    assert_eq!(
        state
            .runner_operations
            .last()
            .expect("runner op")
            .contract_ref,
        state.contract.build_contract_ref
    );
    assert_eq!(
        state.runs.last().expect("run").source_operation_refs,
        state.contract.source_operation_refs
    );
    assert_eq!(
        outcome.host_fabric_contribution.role,
        FABRIC_MEMBER_ROLE_BUILD_PROCESSOR
    );
    assert_eq!(
        state
            .host_fabric_contributions
            .last()
            .expect("fabric contribution")
            .contract_ref,
        state.contract.build_contract_ref
    );
}
