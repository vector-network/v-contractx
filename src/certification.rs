use serde::{Deserialize, Serialize};

use crate::errors::{KernelError, Result};
use crate::utils::AUTH_RATIO_SCALE;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct MicroScore(pub u32);

impl MicroScore {
    pub fn new(value: u32) -> Result<Self> {
        if value > AUTH_RATIO_SCALE as u32 {
            return Err(KernelError::InvalidParameter(
                "micro score must be within [0, 1_000_000]".to_string(),
            ));
        }
        Ok(Self(value))
    }

    pub fn zero() -> Self {
        Self(0)
    }

    pub fn one() -> Self {
        Self(AUTH_RATIO_SCALE as u32)
    }

    pub fn as_f64(&self) -> f64 {
        self.0 as f64 / AUTH_RATIO_SCALE as f64
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthWeights {
    pub magnitude: u32,
    pub composition: u32,
    pub ownership: u32,
    pub policy: u32,
}

impl AuthWeights {
    pub fn normalized(magnitude: u32, composition: u32, ownership: u32, policy: u32) -> Result<Self> {
        let sum = magnitude as u64 + composition as u64 + ownership as u64 + policy as u64;
        if sum != AUTH_RATIO_SCALE {
            return Err(KernelError::InvalidParameter(format!(
                "auth weights must sum to {}",
                AUTH_RATIO_SCALE
            )));
        }
        Ok(Self { magnitude, composition, ownership, policy })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthInputs {
    pub magnitude: MicroScore,
    pub composition: MicroScore,
    pub ownership: MicroScore,
    pub policy: MicroScore,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthRatio(pub u32);

impl AuthRatio {
    pub fn new(value: u32) -> Result<Self> {
        if value > AUTH_RATIO_SCALE as u32 {
            return Err(KernelError::InvalidParameter(
                "auth ratio must be within [0, 1_000_000]".to_string(),
            ));
        }
        Ok(Self(value))
    }

    pub fn as_f64(&self) -> f64 {
        self.0 as f64 / AUTH_RATIO_SCALE as f64
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CertificationThreshold(pub u32);

impl CertificationThreshold {
    pub fn new(value: u32) -> Result<Self> {
        if value > AUTH_RATIO_SCALE as u32 {
            return Err(KernelError::InvalidParameter(
                "threshold must be within [0, 1_000_000]".to_string(),
            ));
        }
        Ok(Self(value))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CertificationState {
    Certified,
    Uncertified,
    Suspended,
    Revoked,
    Pending,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CertificationContext {
    pub vector_type: String,
    pub operation_class: String,
    pub space_policy_version: String,
    pub risk_profile: String,
    pub protocol_version: String,
    pub threshold: CertificationThreshold,
    pub weights: AuthWeights,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CertificationReport {
    pub auth_ratio: AuthRatio,
    pub threshold: CertificationThreshold,
    pub state: CertificationState,
    pub policy_version: String,
    pub reason: Option<String>,
}

pub fn evaluate_auth_ratio(inputs: AuthInputs, weights: AuthWeights) -> Result<AuthRatio> {
    let sum = (weights.magnitude as u64 * inputs.magnitude.0 as u64)
        + (weights.composition as u64 * inputs.composition.0 as u64)
        + (weights.ownership as u64 * inputs.ownership.0 as u64)
        + (weights.policy as u64 * inputs.policy.0 as u64);
    let result = (sum / AUTH_RATIO_SCALE) as u32;
    AuthRatio::new(result)
}

pub fn certify(inputs: AuthInputs, ctx: &CertificationContext) -> Result<CertificationReport> {
    let auth_ratio = evaluate_auth_ratio(inputs, ctx.weights.clone())?;
    let state = if auth_ratio.0 >= ctx.threshold.0 {
        CertificationState::Certified
    } else {
        CertificationState::Uncertified
    };

    Ok(CertificationReport {
        auth_ratio,
        threshold: ctx.threshold.clone(),
        state,
        policy_version: ctx.protocol_version.clone(),
        reason: match state {
            CertificationState::Certified => None,
            _ => Some("auth ratio below threshold".to_string()),
        },
    })
}
