// domain-owned-vocabulary: build.contract.notReady build.contract.expired build.source.mismatch build.recipe.mismatch build.runner.unavailable build.secretBoundary.blocked build.compatibility.unavailable runner.grant.unavailable runner.resource.unavailable
use anyhow::{Result, anyhow};
use constitute_fabric::{HostFabricMemberContributionSpec, build_host_fabric_member_contribution};
use constitute_protocol::{
    BUILD_ARTIFACT_KIND_MODULE, BUILD_CONTRACT_STATE_READY, BUILD_PROOF_STATE_BLOCKED,
    BUILD_PROOF_STATE_PROVED, BUILD_RUN_STATE_BLOCKED, BUILD_RUN_STATE_SUCCEEDED, BuildArtifact,
    BuildContract, BuildProof, BuildRun, CAPABILITY_BUILD_RUN_EXECUTE,
    FABRIC_MEMBER_CONTRIBUTION_BLOCKED, FABRIC_MEMBER_CONTRIBUTION_RUNNING,
    FABRIC_MEMBER_ROLE_BUILD_PROCESSOR, HostFabricMemberContribution, RECORD_BUILD_ARTIFACT,
    RECORD_BUILD_CONTRACT, RECORD_BUILD_PROOF, RECORD_BUILD_RUN, RECORD_RUNNER_OPERATION,
    RUNNER_OPERATION_EXECUTE, RUNNER_OPERATION_STATE_BLOCKED, RUNNER_OPERATION_STATE_SUCCEEDED,
    RunnerOperationRecord, validate_build_artifact, validate_build_contract, validate_build_proof,
    validate_build_run, validate_host_fabric_member_contribution, validate_runner_operation,
};
use serde::{Deserialize, Serialize};
use std::{fs, path::Path};

const DEFAULT_NOW: u64 = 1_779_266_000_000;
const REASON_CONTRACT_NOT_READY: &str = "build.contract.notReady";
const REASON_CONTRACT_EXPIRED: &str = "build.contract.expired";
const REASON_SOURCE_MISMATCH: &str = "build.source.mismatch";
const REASON_RECIPE_MISMATCH: &str = "build.recipe.mismatch";
const REASON_RUNNER_UNAVAILABLE: &str = "build.runner.unavailable";
const REASON_SECRET_BLOCKED: &str = "build.secretBoundary.blocked";
const REASON_COMPATIBILITY_UNAVAILABLE: &str = "build.compatibility.unavailable";
const REASON_GRANT_UNAVAILABLE: &str = "runner.grant.unavailable";
const REASON_RESOURCE_UNAVAILABLE: &str = "runner.resource.unavailable";
const DEFAULT_RUNNER_MEMBER_REF: &str =
    "4a29ff60c5c3837e9e20555bfeb2a046be3eb140818144628691fcf7efb1d2f1";

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BuildFixture {
    pub contract: BuildContract,
    pub runner_operation: RunnerOperationRecord,
    pub host_fabric_contribution: HostFabricMemberContribution,
    pub run: BuildRun,
    pub artifact: Option<BuildArtifact>,
    pub proof: BuildProof,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BuildStatus {
    pub build_contract_ref: String,
    pub source_snapshot_ref: String,
    pub source_operation_ref_count: usize,
    pub runner_ref: String,
    pub runner_operation_ref: String,
    pub artifact_ref: String,
    pub state: String,
    pub run_count: usize,
    pub artifact_count: usize,
    pub proof_count: usize,
    pub runner_operation_count: usize,
    pub host_fabric_contribution_count: usize,
}

#[derive(Clone, Debug)]
pub struct BuildRunRequest {
    pub fabric_ref: String,
    pub source_snapshot_ref: String,
    pub recipe_ref: String,
    pub runner_ref: String,
    pub runner_member_ref: String,
    pub host_ref: String,
    pub requester_ref: String,
    pub runner_operation_ref: String,
    pub grant_refs: Vec<String>,
    pub resource_available: bool,
    pub secret_ready: bool,
    pub compatibility_ready: bool,
    pub now: u64,
}

#[derive(Clone, Debug)]
pub struct BuildOutputPlan {
    pub artifact_refs: Vec<String>,
    pub storage_refs: Vec<String>,
    pub proof_refs: Vec<String>,
    pub log_refs: Vec<String>,
    pub metric_refs: Vec<String>,
    pub release_candidate_refs: Vec<String>,
    pub evidence_refs: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BuildState {
    pub contract: BuildContract,
    #[serde(default)]
    pub runs: Vec<BuildRun>,
    #[serde(default)]
    pub artifacts: Vec<BuildArtifact>,
    #[serde(default)]
    pub proofs: Vec<BuildProof>,
    #[serde(default)]
    pub runner_operations: Vec<RunnerOperationRecord>,
    #[serde(default)]
    pub host_fabric_contributions: Vec<HostFabricMemberContribution>,
    pub updated_at: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BuildRunOutcome {
    pub runner_operation: RunnerOperationRecord,
    pub host_fabric_contribution: HostFabricMemberContribution,
    pub run: BuildRun,
    pub artifact: Option<BuildArtifact>,
    pub proof: BuildProof,
}

pub fn build_fixture(now: u64, state: &str) -> Result<BuildFixture> {
    let blocked = state == BUILD_RUN_STATE_BLOCKED;
    let contract = BuildContract {
        kind: Some(RECORD_BUILD_CONTRACT.to_string()),
        build_contract_ref: "build:contract:build-runner-proof".to_string(),
        app_contract_ref: "app:contract:build-runner-proof@0.1.0".to_string(),
        source_graph_ref: "source:graph:constitute-git".to_string(),
        source_snapshot_ref: "source:snapshot:head".to_string(),
        recipe_ref: "build:recipe:browser-module".to_string(),
        state: BUILD_CONTRACT_STATE_READY.to_string(),
        source_operation_refs: vec![
            "source:operation:ref-update".to_string(),
            "source:operation:project-link".to_string(),
        ],
        content_index_refs: vec!["content-index:source:constitute-git".to_string()],
        runner_role_refs: vec!["runner:role:build".to_string()],
        runner_refs: vec!["runner:instance:local".to_string()],
        resource_grant_refs: vec!["resource:grant:build-lite".to_string()],
        secret_boundary_refs: vec!["secret:boundary:not-required".to_string()],
        compatibility_refs: vec!["compat:surface-app:0.1".to_string()],
        project_refs: vec!["project:constituency".to_string()],
        work_item_refs: vec!["work-item:source-build-lifecycle".to_string()],
        expected_artifact_refs: vec!["build:artifact:module".to_string()],
        evidence_refs: vec!["source:update:main".to_string()],
        blocked_reasons: vec![],
        issued_at: now,
        expires_at: Some(now + 86_400_000),
    };
    let artifact_plan = BuildArtifact {
        kind: Some(RECORD_BUILD_ARTIFACT.to_string()),
        artifact_ref: "build:artifact:module".to_string(),
        run_ref: "build:run:build-runner-proof".to_string(),
        artifact_kind: BUILD_ARTIFACT_KIND_MODULE.to_string(),
        storage_object_ref: "storage:object:build-runner-module".to_string(),
        digest_ref: "digest:sha256:build-runner-module".to_string(),
        compatibility_ref: "compat:surface-app:0.1".to_string(),
        media_type: "application/javascript".to_string(),
        size_bytes: 2048,
        evidence_refs: vec!["build:evidence:artifact-hash".to_string()],
        issued_at: now + 2,
    };
    let proof_plan = BuildProof {
        kind: Some(RECORD_BUILD_PROOF.to_string()),
        proof_ref: "build:proof:build-runner-proof".to_string(),
        run_ref: artifact_plan.run_ref.clone(),
        state: BUILD_PROOF_STATE_PROVED.to_string(),
        source_snapshot_ref: contract.source_snapshot_ref.clone(),
        runner_ref: "runner:instance:local".to_string(),
        source_operation_refs: contract.source_operation_refs.clone(),
        artifact_refs: vec![artifact_plan.artifact_ref.clone()],
        log_refs: vec!["storage:object:build-log".to_string()],
        metric_refs: vec!["metrics:build:build-runner-proof".to_string()],
        evidence_refs: vec!["runner:evidence:build".to_string()],
        blocked_reasons: vec![],
        observed_at: now + 3,
        expires_at: Some(now + 86_400_000),
    };
    let mut request = default_build_run_request(now);
    if blocked {
        request.resource_available = false;
    }
    let output = default_build_output_plan(&artifact_plan, &proof_plan);
    let run = reduce_build_run(&contract, request.clone(), output.clone())?;
    let succeeded = run.state == BUILD_RUN_STATE_SUCCEEDED;
    let proof = BuildProof {
        state: if succeeded {
            BUILD_PROOF_STATE_PROVED.to_string()
        } else {
            BUILD_PROOF_STATE_BLOCKED.to_string()
        },
        artifact_refs: if succeeded {
            vec![artifact_plan.artifact_ref.clone()]
        } else {
            vec![]
        },
        blocked_reasons: if succeeded {
            vec![]
        } else {
            run.blocked_reasons.clone()
        },
        observed_at: if succeeded { now + 3 } else { now + 1 },
        ..proof_plan
    };
    let runner_operation = build_runner_operation(&contract, &run, &request, &output)?;
    let host_fabric_contribution =
        build_host_fabric_contribution_for_run(&contract, &run, &request, &output)?;
    let fixture = BuildFixture {
        contract,
        runner_operation,
        host_fabric_contribution,
        run,
        artifact: succeeded.then_some(artifact_plan),
        proof,
    };
    validate_build_fixture(&fixture)?;
    Ok(fixture)
}

pub fn default_build_run_request(now: u64) -> BuildRunRequest {
    BuildRunRequest {
        fabric_ref: "fabric:runner-lab".to_string(),
        source_snapshot_ref: "source:snapshot:head".to_string(),
        recipe_ref: "build:recipe:browser-module".to_string(),
        runner_ref: "runner:instance:local".to_string(),
        runner_member_ref: DEFAULT_RUNNER_MEMBER_REF.to_string(),
        host_ref: "host:runner-lab".to_string(),
        requester_ref: "identity:aux".to_string(),
        runner_operation_ref: "runner:operation:build-build-runner-proof".to_string(),
        grant_refs: vec!["authority:grant:runner-build".to_string()],
        resource_available: true,
        secret_ready: true,
        compatibility_ready: true,
        now,
    }
}

pub fn default_build_output_plan(artifact: &BuildArtifact, proof: &BuildProof) -> BuildOutputPlan {
    BuildOutputPlan {
        artifact_refs: vec![artifact.artifact_ref.clone()],
        storage_refs: vec![artifact.storage_object_ref.clone()],
        proof_refs: vec![proof.proof_ref.clone()],
        log_refs: vec!["storage:object:build-log".to_string()],
        metric_refs: vec!["metrics:build:build-runner-proof".to_string()],
        release_candidate_refs: vec!["release:candidate:build-runner-proof".to_string()],
        evidence_refs: vec!["runner:evidence:build".to_string()],
    }
}

pub fn reduce_build_run(
    contract: &BuildContract,
    request: BuildRunRequest,
    output: BuildOutputPlan,
) -> Result<BuildRun> {
    validate_build_contract(contract)?;
    let mut blocked_reasons = Vec::new();
    if contract.state != BUILD_CONTRACT_STATE_READY {
        blocked_reasons.push(REASON_CONTRACT_NOT_READY.to_string());
    }
    if contract
        .expires_at
        .is_some_and(|expires_at| expires_at <= request.now)
    {
        blocked_reasons.push(REASON_CONTRACT_EXPIRED.to_string());
    }
    if request.source_snapshot_ref != contract.source_snapshot_ref {
        blocked_reasons.push(REASON_SOURCE_MISMATCH.to_string());
    }
    if request.recipe_ref != contract.recipe_ref {
        blocked_reasons.push(REASON_RECIPE_MISMATCH.to_string());
    }
    if !contract
        .runner_refs
        .iter()
        .any(|runner_ref| runner_ref == &request.runner_ref)
    {
        blocked_reasons.push(REASON_RUNNER_UNAVAILABLE.to_string());
    }
    if request.grant_refs.is_empty() {
        blocked_reasons.push(REASON_GRANT_UNAVAILABLE.to_string());
    }
    if contract.resource_grant_refs.is_empty() || !request.resource_available {
        blocked_reasons.push(REASON_RESOURCE_UNAVAILABLE.to_string());
    }
    if contract.secret_boundary_refs.is_empty() || !request.secret_ready {
        blocked_reasons.push(REASON_SECRET_BLOCKED.to_string());
    }
    if contract.compatibility_refs.is_empty() || !request.compatibility_ready {
        blocked_reasons.push(REASON_COMPATIBILITY_UNAVAILABLE.to_string());
    }
    blocked_reasons.sort();
    blocked_reasons.dedup();
    let succeeded = blocked_reasons.is_empty();

    let run = BuildRun {
        kind: Some(RECORD_BUILD_RUN.to_string()),
        run_ref: "build:run:build-runner-proof".to_string(),
        build_contract_ref: contract.build_contract_ref.clone(),
        source_snapshot_ref: request.source_snapshot_ref,
        recipe_ref: request.recipe_ref,
        runner_ref: request.runner_ref,
        runner_operation_ref: request.runner_operation_ref,
        state: if succeeded {
            BUILD_RUN_STATE_SUCCEEDED.to_string()
        } else {
            BUILD_RUN_STATE_BLOCKED.to_string()
        },
        source_operation_refs: contract.source_operation_refs.clone(),
        content_index_refs: contract.content_index_refs.clone(),
        grant_refs: request.grant_refs,
        resource_grant_refs: contract.resource_grant_refs.clone(),
        secret_boundary_refs: contract.secret_boundary_refs.clone(),
        artifact_refs: if succeeded {
            output.artifact_refs.clone()
        } else {
            vec![]
        },
        log_refs: output.log_refs,
        proof_refs: if succeeded {
            output.proof_refs.clone()
        } else {
            vec![]
        },
        metric_refs: output.metric_refs,
        storage_refs: if succeeded {
            output.storage_refs
        } else {
            vec![]
        },
        compatibility_refs: if succeeded {
            contract.compatibility_refs.clone()
        } else {
            vec![]
        },
        release_candidate_refs: if succeeded {
            output.release_candidate_refs
        } else {
            vec![]
        },
        project_refs: contract.project_refs.clone(),
        work_item_refs: contract.work_item_refs.clone(),
        evidence_refs: output.evidence_refs,
        blocked_reasons,
        safe_facts: serde_json::json!({
            "durationMs": if succeeded { 91 } else { 0 },
            "artifactCount": if succeeded { output.artifact_refs.len() } else { 0 },
            "resourceAvailable": request.resource_available,
            "secretReady": request.secret_ready,
            "compatibilityReady": request.compatibility_ready
        }),
        requested_at: request.now,
        started_at: succeeded.then_some(request.now + 1),
        completed_at: succeeded.then_some(request.now + 3),
        expires_at: Some(request.now + 86_400_000),
    };
    validate_build_run(&run)?;
    Ok(run)
}

pub fn build_runner_operation(
    contract: &BuildContract,
    run: &BuildRun,
    request: &BuildRunRequest,
    output: &BuildOutputPlan,
) -> Result<RunnerOperationRecord> {
    validate_build_contract(contract)?;
    validate_build_run(run)?;
    let succeeded = run.state == BUILD_RUN_STATE_SUCCEEDED;
    let state = if succeeded {
        RUNNER_OPERATION_STATE_SUCCEEDED
    } else {
        RUNNER_OPERATION_STATE_BLOCKED
    };
    let runner_operation = RunnerOperationRecord {
        kind: Some(RECORD_RUNNER_OPERATION.to_string()),
        operation_id: request.runner_operation_ref.clone(),
        runner_id: request.runner_ref.clone(),
        runner_ref: request.runner_member_ref.clone(),
        host_ref: request.host_ref.clone(),
        requester_ref: request.requester_ref.clone(),
        subject_ref: contract.build_contract_ref.clone(),
        contract_ref: contract.build_contract_ref.clone(),
        operation: RUNNER_OPERATION_EXECUTE.to_string(),
        state: state.to_string(),
        grant_refs: request.grant_refs.clone(),
        capability_refs: vec![CAPABILITY_BUILD_RUN_EXECUTE.to_string()],
        input_refs: [
            vec![
                contract.source_snapshot_ref.clone(),
                contract.recipe_ref.clone(),
            ],
            contract.source_operation_refs.clone(),
            contract.content_index_refs.clone(),
        ]
        .concat(),
        output_refs: if succeeded {
            [
                output.artifact_refs.clone(),
                output.proof_refs.clone(),
                output.release_candidate_refs.clone(),
            ]
            .concat()
        } else {
            Vec::new()
        },
        evidence_refs: output.evidence_refs.clone(),
        proof_refs: if succeeded {
            output.proof_refs.clone()
        } else {
            Vec::new()
        },
        release_refs: if succeeded {
            output.release_candidate_refs.clone()
        } else {
            Vec::new()
        },
        resource_budget: serde_json::json!({
            "profileRef": "resource-profile:build-lite",
            "maxMemoryMiB": 512,
            "maxCpuPct": 35
        }),
        resource_posture: None,
        secret_boundary: serde_json::json!({
            "state": if request.secret_ready { "notRequired" } else { "blocked" },
            "blockedReasons": if request.secret_ready { Vec::<String>::new() } else { vec![REASON_SECRET_BLOCKED.to_string()] }
        }),
        release_posture: serde_json::json!({
            "state": if succeeded { "buildReady" } else { "blocked" },
            "buildRef": contract.build_contract_ref,
            "releaseRef": output.release_candidate_refs.first().cloned().unwrap_or_default(),
            "rollbackRef": format!("rollback:{}", contract.build_contract_ref.replace(':', "-")),
            "blockedReasons": if succeeded { Vec::<String>::new() } else { run.blocked_reasons.clone() }
        }),
        rollback_posture: serde_json::json!({
            "state": if succeeded { "rollbackReady" } else { "blocked" },
            "rollbackRef": format!("rollback:{}", contract.build_contract_ref.replace(':', "-")),
            "blockedReasons": if succeeded { Vec::<String>::new() } else { run.blocked_reasons.clone() }
        }),
        release_ref: succeeded
            .then(|| output.release_candidate_refs.first().cloned())
            .flatten(),
        rollback_ref: Some(format!(
            "rollback:{}",
            contract.build_contract_ref.replace(':', "-")
        )),
        blocked_reasons: run.blocked_reasons.clone(),
        safe_facts: serde_json::json!({
            "processorContract": "build",
            "sourceSnapshotRef": contract.source_snapshot_ref,
            "sourceOperationCount": contract.source_operation_refs.len(),
            "recipeRef": contract.recipe_ref,
            "artifactCount": if succeeded { output.artifact_refs.len() } else { 0 },
            "releaseCandidateCount": if succeeded { output.release_candidate_refs.len() } else { 0 }
        }),
        requested_at: run.requested_at,
        accepted_at: succeeded.then_some(run.requested_at + 1),
        started_at: run.started_at,
        completed_at: run.completed_at,
        observed_at: Some(run.completed_at.unwrap_or(run.requested_at + 1)),
        expires_at: run.expires_at,
    };
    validate_runner_operation(&runner_operation)?;
    Ok(runner_operation)
}

pub fn build_host_fabric_contribution_for_run(
    contract: &BuildContract,
    run: &BuildRun,
    request: &BuildRunRequest,
    output: &BuildOutputPlan,
) -> Result<HostFabricMemberContribution> {
    validate_build_contract(contract)?;
    validate_build_run(run)?;
    let succeeded = run.state == BUILD_RUN_STATE_SUCCEEDED;
    let contribution = build_host_fabric_member_contribution(HostFabricMemberContributionSpec {
        contribution_id: format!("fabric-contribution:build:{}", sanitize_ref(&run.run_ref)),
        fabric_ref: request.fabric_ref.clone(),
        host_ref: request.host_ref.clone(),
        member_ref: request.runner_member_ref.clone(),
        role: FABRIC_MEMBER_ROLE_BUILD_PROCESSOR.to_string(),
        state: if succeeded {
            FABRIC_MEMBER_CONTRIBUTION_RUNNING.to_string()
        } else {
            FABRIC_MEMBER_CONTRIBUTION_BLOCKED.to_string()
        },
        contract_ref: contract.build_contract_ref.clone(),
        subject_ref: run.run_ref.clone(),
        capability_refs: vec![CAPABILITY_BUILD_RUN_EXECUTE.to_string()],
        grant_refs: request.grant_refs.clone(),
        input_refs: [
            vec![
                contract.source_snapshot_ref.clone(),
                contract.recipe_ref.clone(),
                request.runner_operation_ref.clone(),
            ],
            contract.source_operation_refs.clone(),
            contract.content_index_refs.clone(),
        ]
        .concat(),
        output_refs: if succeeded {
            [
                output.artifact_refs.clone(),
                output.proof_refs.clone(),
                output.release_candidate_refs.clone(),
            ]
            .concat()
        } else {
            Vec::new()
        },
        evidence_refs: output.evidence_refs.clone(),
        lifecycle_plan_refs: vec![format!(
            "lifecycle-plan:build:{}",
            sanitize_ref(&run.run_ref)
        )],
        release_refs: if succeeded {
            output.release_candidate_refs.clone()
        } else {
            Vec::new()
        },
        resource_posture: None,
        blocked_reasons: run.blocked_reasons.clone(),
        safe_facts: serde_json::json!({
            "processorContract": "build",
            "buildContractRef": contract.build_contract_ref,
            "runRef": run.run_ref,
            "runState": run.state,
            "sourceOperationCount": contract.source_operation_refs.len(),
            "artifactCount": if succeeded { output.artifact_refs.len() } else { 0 },
            "releaseCandidateCount": if succeeded { output.release_candidate_refs.len() } else { 0 }
        }),
        observed_at: run.completed_at.unwrap_or(request.now + 1),
        expires_at: run.expires_at,
    })?;
    validate_host_fabric_member_contribution(&contribution)?;
    Ok(contribution)
}

pub fn default_build_state(now: u64) -> Result<BuildState> {
    let fixture = build_fixture(now, BUILD_RUN_STATE_SUCCEEDED)?;
    let state = BuildState {
        contract: fixture.contract,
        runs: vec![fixture.run],
        artifacts: fixture.artifact.into_iter().collect(),
        proofs: vec![fixture.proof],
        runner_operations: vec![fixture.runner_operation],
        host_fabric_contributions: vec![fixture.host_fabric_contribution],
        updated_at: now,
    };
    validate_build_state(&state)?;
    Ok(state)
}

pub fn load_build_state(path: impl AsRef<Path>, now: u64) -> Result<BuildState> {
    let path = path.as_ref();
    if !path.exists() {
        return default_build_state(now);
    }
    let text = fs::read_to_string(path)?;
    let state = serde_json::from_str::<BuildState>(&text)?;
    validate_build_state(&state)?;
    Ok(state)
}

pub fn save_build_state(path: impl AsRef<Path>, state: &BuildState) -> Result<()> {
    validate_build_state(state)?;
    let path = path.as_ref();
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)?;
        }
    }
    fs::write(path, serde_json::to_string_pretty(state)?)?;
    Ok(())
}

pub fn append_build_run(
    state: &mut BuildState,
    request: BuildRunRequest,
    output: BuildOutputPlan,
) -> Result<BuildRunOutcome> {
    validate_build_state(state)?;
    let run = reduce_build_run(&state.contract, request.clone(), output.clone())?;
    let runner_operation = build_runner_operation(&state.contract, &run, &request, &output)?;
    let host_fabric_contribution =
        build_host_fabric_contribution_for_run(&state.contract, &run, &request, &output)?;
    let artifact = if run.state == BUILD_RUN_STATE_SUCCEEDED {
        Some(default_artifact_for_run(&run, &output, request.now)?)
    } else {
        None
    };
    let proof = build_proof_for_run(&run, &output, request.now)?;
    state.runs.push(run.clone());
    if let Some(artifact) = artifact.clone() {
        state.artifacts.push(artifact);
    }
    state.proofs.push(proof.clone());
    state.runner_operations.push(runner_operation.clone());
    state
        .host_fabric_contributions
        .push(host_fabric_contribution.clone());
    state.updated_at = request.now;
    validate_build_state(state)?;
    Ok(BuildRunOutcome {
        runner_operation,
        host_fabric_contribution,
        run,
        artifact,
        proof,
    })
}

pub fn build_status() -> Result<BuildStatus> {
    build_state_status(&default_build_state(DEFAULT_NOW)?)
}

pub fn build_state_status(state: &BuildState) -> Result<BuildStatus> {
    validate_build_state(state)?;
    let last_run = state
        .runs
        .last()
        .ok_or_else(|| anyhow!("build state missing runs"))?;
    Ok(BuildStatus {
        build_contract_ref: state.contract.build_contract_ref.clone(),
        source_snapshot_ref: state.contract.source_snapshot_ref.clone(),
        source_operation_ref_count: state.contract.source_operation_refs.len(),
        runner_ref: last_run.runner_ref.clone(),
        runner_operation_ref: last_run.runner_operation_ref.clone(),
        artifact_ref: state
            .artifacts
            .last()
            .map(|artifact| artifact.artifact_ref.clone())
            .unwrap_or_default(),
        state: last_run.state.clone(),
        run_count: state.runs.len(),
        artifact_count: state.artifacts.len(),
        proof_count: state.proofs.len(),
        runner_operation_count: state.runner_operations.len(),
        host_fabric_contribution_count: state.host_fabric_contributions.len(),
    })
}

pub fn validate_build_fixture(fixture: &BuildFixture) -> Result<()> {
    validate_build_contract(&fixture.contract)?;
    validate_build_run(&fixture.run)?;
    validate_build_proof(&fixture.proof)?;
    validate_runner_operation(&fixture.runner_operation)?;
    validate_host_fabric_member_contribution(&fixture.host_fabric_contribution)?;
    if fixture.host_fabric_contribution.role != FABRIC_MEMBER_ROLE_BUILD_PROCESSOR {
        return Err(anyhow!(
            "build fixture host-fabric contribution must be buildProcessor"
        ));
    }
    if fixture.host_fabric_contribution.contract_ref != fixture.contract.build_contract_ref {
        return Err(anyhow!(
            "build fixture host-fabric contribution contract mismatch"
        ));
    }
    if fixture.host_fabric_contribution.subject_ref != fixture.run.run_ref {
        return Err(anyhow!(
            "build fixture host-fabric contribution subject mismatch"
        ));
    }
    if fixture.run.source_operation_refs != fixture.contract.source_operation_refs {
        return Err(anyhow!("build run source operation refs diverge"));
    }
    if fixture.run.state == BUILD_RUN_STATE_SUCCEEDED {
        let artifact = fixture
            .artifact
            .as_ref()
            .ok_or_else(|| anyhow!("succeeded build fixture missing artifact"))?;
        validate_build_artifact(artifact)?;
        if fixture.run.source_snapshot_ref != fixture.proof.source_snapshot_ref {
            return Err(anyhow!("build run and proof source snapshots diverge"));
        }
        if fixture.run.source_operation_refs != fixture.proof.source_operation_refs {
            return Err(anyhow!("build run and proof source operation refs diverge"));
        }
        if !fixture
            .run
            .compatibility_refs
            .iter()
            .any(|compatibility_ref| compatibility_ref == &artifact.compatibility_ref)
        {
            return Err(anyhow!("build run missing artifact compatibility posture"));
        }
        if fixture.run.release_candidate_refs.is_empty() {
            return Err(anyhow!("build run missing release candidate posture"));
        }
    } else if fixture.artifact.is_some() {
        return Err(anyhow!("blocked build fixture must not emit artifact"));
    }
    Ok(())
}

pub fn validate_build_state(state: &BuildState) -> Result<()> {
    validate_build_contract(&state.contract)?;
    for run in &state.runs {
        validate_build_run(run)?;
        if run.build_contract_ref != state.contract.build_contract_ref {
            return Err(anyhow!("build state run contract mismatch"));
        }
        if run.source_operation_refs != state.contract.source_operation_refs {
            return Err(anyhow!("build state run source operation refs diverge"));
        }
    }
    for artifact in &state.artifacts {
        validate_build_artifact(artifact)?;
    }
    for proof in &state.proofs {
        validate_build_proof(proof)?;
    }
    for runner_operation in &state.runner_operations {
        validate_runner_operation(runner_operation)?;
        if runner_operation.contract_ref != state.contract.build_contract_ref {
            return Err(anyhow!("build state runner operation contract mismatch"));
        }
    }
    for host_fabric_contribution in &state.host_fabric_contributions {
        validate_host_fabric_member_contribution(host_fabric_contribution)?;
        if host_fabric_contribution.contract_ref != state.contract.build_contract_ref {
            return Err(anyhow!(
                "build state host-fabric contribution contract mismatch"
            ));
        }
    }
    Ok(())
}

fn sanitize_ref(value: &str) -> String {
    value
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || character == '-' {
                character
            } else {
                '-'
            }
        })
        .collect()
}

fn default_artifact_for_run(
    run: &BuildRun,
    output: &BuildOutputPlan,
    now: u64,
) -> Result<BuildArtifact> {
    let artifact = BuildArtifact {
        kind: Some(RECORD_BUILD_ARTIFACT.to_string()),
        artifact_ref: output
            .artifact_refs
            .first()
            .cloned()
            .ok_or_else(|| anyhow!("succeeded build missing artifact ref"))?,
        run_ref: run.run_ref.clone(),
        artifact_kind: BUILD_ARTIFACT_KIND_MODULE.to_string(),
        storage_object_ref: output
            .storage_refs
            .first()
            .cloned()
            .ok_or_else(|| anyhow!("succeeded build missing storage ref"))?,
        digest_ref: "digest:sha256:build-artifact".to_string(),
        compatibility_ref: run
            .compatibility_refs
            .first()
            .cloned()
            .ok_or_else(|| anyhow!("succeeded build missing compatibility ref"))?,
        media_type: "application/javascript".to_string(),
        size_bytes: 2048,
        evidence_refs: output.evidence_refs.clone(),
        issued_at: now + 2,
    };
    validate_build_artifact(&artifact)?;
    Ok(artifact)
}

fn build_proof_for_run(run: &BuildRun, output: &BuildOutputPlan, now: u64) -> Result<BuildProof> {
    let succeeded = run.state == BUILD_RUN_STATE_SUCCEEDED;
    let proof = BuildProof {
        kind: Some(RECORD_BUILD_PROOF.to_string()),
        proof_ref: output
            .proof_refs
            .first()
            .cloned()
            .unwrap_or_else(|| format!("build:proof:{}", run.run_ref.replace(':', "-"))),
        run_ref: run.run_ref.clone(),
        state: if succeeded {
            BUILD_PROOF_STATE_PROVED.to_string()
        } else {
            BUILD_PROOF_STATE_BLOCKED.to_string()
        },
        source_snapshot_ref: run.source_snapshot_ref.clone(),
        runner_ref: run.runner_ref.clone(),
        source_operation_refs: run.source_operation_refs.clone(),
        artifact_refs: if succeeded {
            output.artifact_refs.clone()
        } else {
            Vec::new()
        },
        log_refs: output.log_refs.clone(),
        metric_refs: output.metric_refs.clone(),
        evidence_refs: output.evidence_refs.clone(),
        blocked_reasons: if succeeded {
            Vec::new()
        } else {
            run.blocked_reasons.clone()
        },
        observed_at: now + 3,
        expires_at: run.expires_at,
    };
    validate_build_proof(&proof)?;
    Ok(proof)
}

pub fn default_now() -> u64 {
    DEFAULT_NOW
}
