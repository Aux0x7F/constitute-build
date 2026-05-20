use anyhow::{Result, anyhow};
use constitute_protocol::{
    BUILD_ARTIFACT_KIND_MODULE, BUILD_CONTRACT_STATE_READY, BUILD_PROOF_STATE_PROVED,
    BUILD_RUN_STATE_BLOCKED, BUILD_RUN_STATE_SUCCEEDED, BuildArtifact, BuildContract, BuildProof,
    BuildRun, RECORD_BUILD_ARTIFACT, RECORD_BUILD_CONTRACT, RECORD_BUILD_PROOF, RECORD_BUILD_RUN,
    validate_build_artifact, validate_build_contract, validate_build_proof, validate_build_run,
};
use serde::Serialize;

const DEFAULT_NOW: u64 = 1_779_266_000_000;

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BuildFixture {
    pub contract: BuildContract,
    pub run: BuildRun,
    pub artifact: BuildArtifact,
    pub proof: BuildProof,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BuildStatus {
    pub build_contract_ref: String,
    pub source_snapshot_ref: String,
    pub runner_ref: String,
    pub artifact_ref: String,
    pub state: String,
}

pub fn build_fixture(now: u64, state: &str) -> Result<BuildFixture> {
    let blocked = state == BUILD_RUN_STATE_BLOCKED;
    let contract = BuildContract {
        kind: Some(RECORD_BUILD_CONTRACT.to_string()),
        build_contract_ref: "build:contract:cybersec-bootstrap".to_string(),
        app_contract_ref: "app:contract:cybersec@0.1.0".to_string(),
        source_graph_ref: "source:graph:constitute-git".to_string(),
        source_snapshot_ref: "source:snapshot:head".to_string(),
        recipe_ref: "build:recipe:browser-module".to_string(),
        state: BUILD_CONTRACT_STATE_READY.to_string(),
        runner_role_refs: vec!["runner:role:build".to_string()],
        runner_refs: vec!["runner:instance:local".to_string()],
        resource_grant_refs: vec!["resource:grant:build-lite".to_string()],
        secret_boundary_refs: vec!["secret:boundary:not-required".to_string()],
        compatibility_refs: vec!["compat:surface-app:0.1".to_string()],
        expected_artifact_refs: vec!["build:artifact:module".to_string()],
        evidence_refs: vec!["source:update:main".to_string()],
        blocked_reasons: vec![],
        issued_at: now,
        expires_at: Some(now + 86_400_000),
    };
    let artifact = BuildArtifact {
        kind: Some(RECORD_BUILD_ARTIFACT.to_string()),
        artifact_ref: "build:artifact:module".to_string(),
        run_ref: "build:run:cybersec-bootstrap".to_string(),
        artifact_kind: BUILD_ARTIFACT_KIND_MODULE.to_string(),
        storage_object_ref: "storage:object:cybersec-module".to_string(),
        digest_ref: "digest:sha256:cybersec-module".to_string(),
        compatibility_ref: "compat:surface-app:0.1".to_string(),
        media_type: "application/javascript".to_string(),
        size_bytes: 2048,
        evidence_refs: vec!["build:evidence:artifact-hash".to_string()],
        issued_at: now + 2,
    };
    let proof = BuildProof {
        kind: Some(RECORD_BUILD_PROOF.to_string()),
        proof_ref: "build:proof:cybersec-bootstrap".to_string(),
        run_ref: artifact.run_ref.clone(),
        state: BUILD_PROOF_STATE_PROVED.to_string(),
        source_snapshot_ref: contract.source_snapshot_ref.clone(),
        runner_ref: "runner:instance:local".to_string(),
        artifact_refs: vec![artifact.artifact_ref.clone()],
        log_refs: vec!["storage:object:build-log".to_string()],
        metric_refs: vec!["metrics:build:cybersec-bootstrap".to_string()],
        evidence_refs: vec!["runner:evidence:build".to_string()],
        blocked_reasons: vec![],
        observed_at: now + 3,
        expires_at: Some(now + 86_400_000),
    };
    let run = BuildRun {
        kind: Some(RECORD_BUILD_RUN.to_string()),
        run_ref: artifact.run_ref.clone(),
        build_contract_ref: contract.build_contract_ref.clone(),
        source_snapshot_ref: contract.source_snapshot_ref.clone(),
        recipe_ref: contract.recipe_ref.clone(),
        runner_ref: "runner:instance:local".to_string(),
        runner_operation_ref: "runner:operation:build-cybersec-bootstrap".to_string(),
        state: state.to_string(),
        grant_refs: vec!["authority:grant:runner-build".to_string()],
        artifact_refs: if blocked {
            vec![]
        } else {
            vec![artifact.artifact_ref.clone()]
        },
        log_refs: vec!["storage:object:build-log".to_string()],
        proof_refs: if blocked {
            vec![]
        } else {
            vec![proof.proof_ref.clone()]
        },
        metric_refs: vec!["metrics:build:cybersec-bootstrap".to_string()],
        storage_refs: if blocked {
            vec![]
        } else {
            vec![artifact.storage_object_ref.clone()]
        },
        evidence_refs: vec!["runner:evidence:build".to_string()],
        blocked_reasons: if blocked {
            vec!["runner.resource.unavailable".to_string()]
        } else {
            vec![]
        },
        safe_facts: serde_json::json!({
            "durationMs": 91,
            "artifactCount": if blocked { 0 } else { 1 }
        }),
        requested_at: now,
        started_at: Some(now + 1),
        completed_at: Some(now + 3),
        expires_at: Some(now + 86_400_000),
    };
    let fixture = BuildFixture {
        contract,
        run,
        artifact,
        proof,
    };
    validate_build_fixture(&fixture)?;
    Ok(fixture)
}

pub fn build_status() -> Result<BuildStatus> {
    let fixture = build_fixture(DEFAULT_NOW, BUILD_RUN_STATE_SUCCEEDED)?;
    Ok(BuildStatus {
        build_contract_ref: fixture.contract.build_contract_ref,
        source_snapshot_ref: fixture.contract.source_snapshot_ref,
        runner_ref: fixture.run.runner_ref,
        artifact_ref: fixture.artifact.artifact_ref,
        state: fixture.run.state,
    })
}

pub fn validate_build_fixture(fixture: &BuildFixture) -> Result<()> {
    validate_build_contract(&fixture.contract)?;
    validate_build_run(&fixture.run)?;
    if fixture.run.state == BUILD_RUN_STATE_SUCCEEDED {
        validate_build_artifact(&fixture.artifact)?;
        validate_build_proof(&fixture.proof)?;
        if fixture.run.source_snapshot_ref != fixture.proof.source_snapshot_ref {
            return Err(anyhow!("build run and proof source snapshots diverge"));
        }
    }
    Ok(())
}

pub fn default_now() -> u64 {
    DEFAULT_NOW
}
