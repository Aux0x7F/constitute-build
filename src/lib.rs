// domain-owned-vocabulary: build.contract.notReady build.contract.expired build.source.mismatch build.recipe.mismatch build.runner.unavailable build.secretBoundary.blocked build.compatibility.unavailable runner.grant.unavailable runner.resource.unavailable
use anyhow::{Result, anyhow};
use constitute_protocol::{
    BUILD_ARTIFACT_KIND_MODULE, BUILD_CONTRACT_STATE_READY, BUILD_PROOF_STATE_BLOCKED,
    BUILD_PROOF_STATE_PROVED, BUILD_RUN_STATE_BLOCKED, BUILD_RUN_STATE_SUCCEEDED, BuildArtifact,
    BuildContract, BuildProof, BuildRun, RECORD_BUILD_ARTIFACT, RECORD_BUILD_CONTRACT,
    RECORD_BUILD_PROOF, RECORD_BUILD_RUN, validate_build_artifact, validate_build_contract,
    validate_build_proof, validate_build_run,
};
use serde::Serialize;

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

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BuildFixture {
    pub contract: BuildContract,
    pub run: BuildRun,
    pub artifact: Option<BuildArtifact>,
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

#[derive(Clone, Debug)]
pub struct BuildRunRequest {
    pub source_snapshot_ref: String,
    pub recipe_ref: String,
    pub runner_ref: String,
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
    let artifact_plan = BuildArtifact {
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
    let proof_plan = BuildProof {
        kind: Some(RECORD_BUILD_PROOF.to_string()),
        proof_ref: "build:proof:cybersec-bootstrap".to_string(),
        run_ref: artifact_plan.run_ref.clone(),
        state: BUILD_PROOF_STATE_PROVED.to_string(),
        source_snapshot_ref: contract.source_snapshot_ref.clone(),
        runner_ref: "runner:instance:local".to_string(),
        artifact_refs: vec![artifact_plan.artifact_ref.clone()],
        log_refs: vec!["storage:object:build-log".to_string()],
        metric_refs: vec!["metrics:build:cybersec-bootstrap".to_string()],
        evidence_refs: vec!["runner:evidence:build".to_string()],
        blocked_reasons: vec![],
        observed_at: now + 3,
        expires_at: Some(now + 86_400_000),
    };
    let mut request = default_build_run_request(now);
    if blocked {
        request.resource_available = false;
    }
    let run = reduce_build_run(
        &contract,
        request,
        default_build_output_plan(&artifact_plan, &proof_plan),
    )?;
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
    let fixture = BuildFixture {
        contract,
        run,
        artifact: succeeded.then_some(artifact_plan),
        proof,
    };
    validate_build_fixture(&fixture)?;
    Ok(fixture)
}

pub fn default_build_run_request(now: u64) -> BuildRunRequest {
    BuildRunRequest {
        source_snapshot_ref: "source:snapshot:head".to_string(),
        recipe_ref: "build:recipe:browser-module".to_string(),
        runner_ref: "runner:instance:local".to_string(),
        runner_operation_ref: "runner:operation:build-cybersec-bootstrap".to_string(),
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
        metric_refs: vec!["metrics:build:cybersec-bootstrap".to_string()],
        release_candidate_refs: vec!["release:candidate:cybersec-bootstrap".to_string()],
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
        run_ref: "build:run:cybersec-bootstrap".to_string(),
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

pub fn build_status() -> Result<BuildStatus> {
    let fixture = build_fixture(DEFAULT_NOW, BUILD_RUN_STATE_SUCCEEDED)?;
    Ok(BuildStatus {
        build_contract_ref: fixture.contract.build_contract_ref,
        source_snapshot_ref: fixture.contract.source_snapshot_ref,
        runner_ref: fixture.run.runner_ref,
        artifact_ref: fixture
            .artifact
            .expect("succeeded status fixture has artifact")
            .artifact_ref,
        state: fixture.run.state,
    })
}

pub fn validate_build_fixture(fixture: &BuildFixture) -> Result<()> {
    validate_build_contract(&fixture.contract)?;
    validate_build_run(&fixture.run)?;
    validate_build_proof(&fixture.proof)?;
    if fixture.run.state == BUILD_RUN_STATE_SUCCEEDED {
        let artifact = fixture
            .artifact
            .as_ref()
            .ok_or_else(|| anyhow!("succeeded build fixture missing artifact"))?;
        validate_build_artifact(artifact)?;
        if fixture.run.source_snapshot_ref != fixture.proof.source_snapshot_ref {
            return Err(anyhow!("build run and proof source snapshots diverge"));
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
        return Err(anyhow!(
            "blocked build fixture must not materialize artifact"
        ));
    }
    Ok(())
}

pub fn default_now() -> u64 {
    DEFAULT_NOW
}
