use constitute_build::{
    BUILD_MATERIALIZATION_DEGRADED, BUILD_MATERIALIZATION_READY, BuildMaterializationProjection,
    BuildMaterializedFileEntry, GeneratedBuildManifestAdapterInput,
    GeneratedBuildManifestArtifactProjection, RECORD_BUILD_MATERIALIZATION_PROJECTION,
    append_build_run, build_fixture, build_fixture_from_projection, build_projection_status,
    build_state_status, build_status, default_build_output_plan, default_build_run_request,
    default_build_state, default_now, reduce_build_run, validate_build_fixture,
    validate_build_materialization_projection, validate_build_state,
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

#[test]
fn build_projection_consumes_materialized_source_refs_without_raw_paths() {
    let projection = BuildMaterializationProjection {
        kind: Some(RECORD_BUILD_MATERIALIZATION_PROJECTION.to_string()),
        projection_ref: "build-materialization:projection:constitute-build".to_string(),
        state: BUILD_MATERIALIZATION_DEGRADED.to_string(),
        source_snapshot_ref: "source:snapshot:native-dev:constitute-build:head".to_string(),
        content_index_ref: "content-index:native-dev:constitute-build:head".to_string(),
        materialized_root_ref: "materialized:root:native-dev:constitute-build".to_string(),
        materialized_path_ref: "materialized:path:native-dev:constitute-build".to_string(),
        file_entries: vec![BuildMaterializedFileEntry {
            file_ref: "source:file:native-dev:constitute-build:lib".to_string(),
            path_ref: "source:path:native-dev:constitute-build:src-lib".to_string(),
            virtual_path: "src/lib.rs".to_string(),
            hash_ref: "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
                .to_string(),
            byte_length: 512,
            storage_object_ref: Some(
                "storage:object:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"
                    .to_string(),
            ),
            materialized_path_ref: "materialized:file:native-dev:constitute-build:src-lib"
                .to_string(),
            evidence_refs: vec!["evidence:projection:file-map".to_string()],
        }],
        dependency_refs: vec!["dependency:cargo:constitute-protocol".to_string()],
        toolchain_refs: vec!["toolchain:rust:cargo".to_string()],
        generated_build_manifest_adapter_input_ref: None,
        generated_build_manifest_artifact_projection_ref: None,
        generated_build_manifest_adapter_input: None,
        generated_build_manifest_artifact_projection: None,
        adapter_refs: vec!["adapter:workspace-fs".to_string()],
        reverse_mapping_refs: vec!["reverse-map:source:file-to-materialized-path".to_string()],
        conflict_refs: vec![
            "transition-conflict:constitute-build:cargo-git:constitute-protocol".to_string(),
        ],
        evidence_refs: vec!["evidence:build-materialization:projection".to_string()],
        blocked_reasons: vec![],
        safe_facts: serde_json::json!({ "repoRef": "repo:constitute-build" }),
        observed_at: default_now(),
        expires_at: Some(default_now() + 600),
    };

    validate_build_materialization_projection(&projection).expect("projection validates");
    let status = build_projection_status(&projection).expect("projection status reduces");
    assert_eq!(status.file_count, 1);
    assert_eq!(status.conflict_count, 1);
    let fulfilled =
        build_fixture_from_projection(&projection, default_now(), BUILD_RUN_STATE_SUCCEEDED)
            .expect("projection drives build fulfillment");
    assert_eq!(
        fulfilled.contract.source_snapshot_ref,
        projection.source_snapshot_ref
    );
    assert_eq!(
        fulfilled.run.content_index_refs,
        vec![projection.content_index_ref.clone()]
    );
    assert!(
        fulfilled
            .runner_operation
            .input_refs
            .contains(&projection.projection_ref)
    );
    assert!(
        fulfilled
            .artifact
            .as_ref()
            .expect("artifact")
            .storage_object_ref
            .starts_with("storage:object:")
    );

    let mut raw_path = projection.clone();
    raw_path.file_entries[0].virtual_path = "C:\\dev\\src\\lib.rs".to_string();
    assert!(validate_build_materialization_projection(&raw_path).is_err());

    let mut symbolic_object = projection;
    symbolic_object.file_entries[0].storage_object_ref =
        Some("storage:object:constitute-build".to_string());
    assert!(validate_build_materialization_projection(&symbolic_object).is_err());
}

#[test]
fn generated_manifest_artifact_projection_becomes_selected_build_input() {
    let now = default_now();
    let artifact_projection = GeneratedBuildManifestArtifactProjection {
        kind: Some("build.generated-manifest.artifact-projection".to_string()),
        projection_ref: "build-manifest:artifact-projection:native-dev:constitute-build:test"
            .to_string(),
        state: BUILD_MATERIALIZATION_READY.to_string(),
        input_ref: "build-manifest:adapter-input:native-dev:constitute-build:test".to_string(),
        repo_ref: "repo:constitute-build".to_string(),
        module_ref: "module:native-dev:constitute-build".to_string(),
        content_hash_ref: "sha256:cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc"
            .to_string(),
        manifest_artifact_refs: vec![
            "build-manifest:artifact:cargo-manifest:native-dev:constitute-build:test".to_string(),
            "build-manifest:artifact:cargo-source-patch:native-dev:constitute-build:test"
                .to_string(),
        ],
        source_snapshot_refs: vec!["source:snapshot:native-dev:constitute-build:head".to_string()],
        content_index_refs: vec!["content-index:native-dev:constitute-build:head".to_string()],
        dependency_refs: vec!["dependency:cargo:constitute-protocol".to_string()],
        native_dependency_refs: vec!["module:native-dev:constitute-protocol".to_string()],
        dependency_source_snapshot_refs: vec![
            "source:snapshot:native-dev:constitute-protocol:head".to_string(),
        ],
        dependency_content_index_refs: vec![
            "content-index:native-dev:constitute-protocol:head".to_string(),
        ],
        dependency_folder_projection_refs: vec![
            "materialized:folder-projection:workspace-dev:constitute-protocol".to_string(),
        ],
        dependency_tool_materialization_refs: vec![
            "materialized:folder-projection:workspace-dev:constitute-protocol".to_string(),
        ],
        storage_backed_dependency_input_refs: vec![
            "build-input:storage-source-pack:native-dev:constitute-protocol:test".to_string(),
        ],
        storage_object_refs: vec![
            "storage:object:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"
                .to_string(),
        ],
        availability_refs: vec!["storage-availability:constitute-protocol".to_string()],
        toolchain_refs: vec!["toolchain:rust:cargo".to_string()],
        reverse_mapping_refs: vec![
            "reverse-map:generated-build-manifest:constitute-build:test".to_string(),
        ],
        deletion_gate: "generated-manifest-artifact-must-be-consumed-before-cargo-path-retirement"
            .to_string(),
        blocked_reasons: vec![],
        safe_facts: serde_json::json!({
            "generatedFromNativeRefs": true,
            "handMaintainedCargoTomlIsTargetModel": false
        }),
    };
    let projection = BuildMaterializationProjection {
        kind: Some(RECORD_BUILD_MATERIALIZATION_PROJECTION.to_string()),
        projection_ref: "build-materialization:projection:native-dev:constitute-build:test"
            .to_string(),
        state: BUILD_MATERIALIZATION_READY.to_string(),
        source_snapshot_ref: "source:snapshot:native-dev:constitute-build:head".to_string(),
        content_index_ref: "content-index:native-dev:constitute-build:head".to_string(),
        materialized_root_ref: "materialized:root:native-dev:constitute-build".to_string(),
        materialized_path_ref: "materialized:path:native-dev:constitute-build".to_string(),
        file_entries: vec![BuildMaterializedFileEntry {
            file_ref: "source:file:native-dev:constitute-build:lib".to_string(),
            path_ref: "source:path:native-dev:constitute-build:src-lib".to_string(),
            virtual_path: "src/lib.rs".to_string(),
            hash_ref: "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
                .to_string(),
            byte_length: 512,
            storage_object_ref: Some(
                "storage:object:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"
                    .to_string(),
            ),
            materialized_path_ref: "materialized:file:native-dev:constitute-build:src-lib"
                .to_string(),
            evidence_refs: vec!["evidence:projection:file-map".to_string()],
        }],
        dependency_refs: vec!["dependency:cargo:constitute-protocol".to_string()],
        toolchain_refs: vec!["toolchain:rust:cargo".to_string()],
        generated_build_manifest_adapter_input_ref: Some(
            "build-manifest:adapter-input:native-dev:constitute-build:test".to_string(),
        ),
        generated_build_manifest_artifact_projection_ref: Some(
            artifact_projection.projection_ref.clone(),
        ),
        generated_build_manifest_adapter_input: Some(GeneratedBuildManifestAdapterInput {
            kind: Some("build.generated-manifest.adapter-input".to_string()),
            input_ref: "build-manifest:adapter-input:native-dev:constitute-build:test".to_string(),
            state: BUILD_MATERIALIZATION_READY.to_string(),
            repo_ref: "repo:constitute-build".to_string(),
            module_ref: "module:native-dev:constitute-build".to_string(),
            source_snapshot_ref: "source:snapshot:native-dev:constitute-build:head".to_string(),
            content_index_ref: "content-index:native-dev:constitute-build:head".to_string(),
            projection_ref: "build-materialization:projection:native-dev:constitute-build:test"
                .to_string(),
            toolchain_refs: vec!["toolchain:rust:cargo".to_string()],
            input_refs: vec!["module:native-dev:constitute-protocol".to_string()],
            artifact_projection_ref: Some(artifact_projection.projection_ref.clone()),
            artifact_projection: Some(artifact_projection.clone()),
            artifact_refs: artifact_projection.manifest_artifact_refs.clone(),
            output_refs: artifact_projection.manifest_artifact_refs.clone(),
            blocked_reasons: vec![],
            safe_facts: serde_json::json!({
                "generatedFromNativeRefs": true,
                "cargoManifestAuthority": false
            }),
            observed_at: now,
            expires_at: Some(now + 600),
        }),
        generated_build_manifest_artifact_projection: Some(artifact_projection.clone()),
        adapter_refs: vec!["adapter:workspace-fs".to_string()],
        reverse_mapping_refs: vec!["reverse-map:source:file-to-materialized-path".to_string()],
        conflict_refs: vec![],
        evidence_refs: vec!["evidence:build-materialization:projection".to_string()],
        blocked_reasons: vec![],
        safe_facts: serde_json::json!({ "repoRef": "repo:constitute-build" }),
        observed_at: now,
        expires_at: Some(now + 600),
    };

    let fulfilled = build_fixture_from_projection(&projection, now, BUILD_RUN_STATE_SUCCEEDED)
        .expect("generated manifest projection drives build fulfillment");
    assert_eq!(
        fulfilled.contract.recipe_ref,
        "build:recipe:generated-manifest-artifact"
    );
    assert!(
        !fulfilled
            .contract
            .compatibility_refs
            .contains(&"compat:adapter:cargo-path-residency-fallback".to_string())
    );
    for artifact_ref in &artifact_projection.manifest_artifact_refs {
        assert!(fulfilled.runner_operation.input_refs.contains(artifact_ref));
        assert!(
            fulfilled
                .host_fabric_contribution
                .input_refs
                .contains(artifact_ref)
        );
        assert!(fulfilled.proof.evidence_refs.contains(artifact_ref));
    }
    for input_ref in &artifact_projection.storage_backed_dependency_input_refs {
        assert!(fulfilled.runner_operation.input_refs.contains(input_ref));
        assert!(
            fulfilled
                .host_fabric_contribution
                .input_refs
                .contains(input_ref)
        );
        assert!(fulfilled.proof.evidence_refs.contains(input_ref));
    }
    assert_eq!(
        fulfilled.run.safe_facts["generatedBuildManifestSelected"],
        serde_json::Value::Bool(true)
    );
    assert!(
        fulfilled.runner_operation.safe_facts["compatibilityAdapterRefs"]
            .as_array()
            .expect("compatibility adapter refs")
            .is_empty()
    );
}
