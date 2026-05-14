use serde::{Deserialize, Serialize};

use crate::errors::{KernelError, Result};
use crate::utils::{AUTH_RATIO_SCALE, clamp_u128};

pub type ComponentAmount = u128;
pub type Timestamp = u64;
pub type LogicalClock = u64;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Vector {
    pub components: Vec<ComponentAmount>,
}

impl Vector {
    pub fn new(components: Vec<ComponentAmount>) -> Self {
        Self { components }
    }

    pub fn zero(dimensions: usize) -> Self {
        Self { components: vec![0; dimensions] }
    }

    pub fn len(&self) -> usize {
        self.components.len()
    }

    pub fn is_zero(&self) -> bool {
        self.components.iter().all(|&c| c == 0)
    }

    pub fn magnitude(&self) -> ComponentAmount {
        self.components.iter().copied().sum()
    }

    pub fn checked_sub_vector(&self, rhs: &Vector) -> Result<Vector> {
        self.ensure_same_dim(rhs)?;
        let mut out = Vec::with_capacity(self.len());
        for (a, b) in self.components.iter().zip(rhs.components.iter()) {
            if b > a {
                return Err(KernelError::InvalidState(
                    "cannot subtract larger projected vector from source".to_string(),
                ));
            }
            out.push(a - b);
        }
        Ok(Vector::new(out))
    }

    pub fn checked_add_vector(&self, rhs: &Vector) -> Result<Vector> {
        self.ensure_same_dim(rhs)?;
        let mut out = Vec::with_capacity(self.len());
        for (a, b) in self.components.iter().zip(rhs.components.iter()) {
            out.push(a.saturating_add(*b));
        }
        Ok(Vector::new(out))
    }

    pub fn scale_bps(&self, bps: u32) -> Vector {
        let denom = AUTH_RATIO_SCALE as u128;
        let scaled = self
            .components
            .iter()
            .map(|c| ((*c).saturating_mul(bps as u128)) / denom)
            .collect();
        Vector::new(scaled)
    }

    pub fn scale_ratio_floor(&self, numerator: u128, denominator: u128) -> Result<Vector> {
        if denominator == 0 {
            return Err(KernelError::InvalidParameter("ratio denominator cannot be zero".to_string()));
        }
        let scaled = self
            .components
            .iter()
            .map(|c| (c.saturating_mul(numerator)) / denominator)
            .collect();
        Ok(Vector::new(scaled))
    }

    pub fn componentwise_min(&self, rhs: &Vector) -> Result<Vector> {
        self.ensure_same_dim(rhs)?;
        let out = self
            .components
            .iter()
            .zip(rhs.components.iter())
            .map(|(a, b)| (*a).min(*b))
            .collect();
        Ok(Vector::new(out))
    }

    pub fn ensure_same_dim(&self, rhs: &Vector) -> Result<()> {
        if self.len() != rhs.len() {
            return Err(KernelError::TypeMismatch(format!(
                "vector dimension mismatch: {} vs {}",
                self.len(),
                rhs.len()
            )));
        }
        Ok(())
    }

    pub fn clamp_non_negative_components(&self) -> Self {
        Self::new(self.components.iter().map(|&c| clamp_u128(c, 0, u128::MAX)).collect())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum VectorTypeTag {
    Position,
    Free,
    Bound,
    Unit,
    Zero,
    Spatial,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Wallet {
    pub wallet_id: String,
    pub pk: String,
    pub wallet_meta: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VectorState {
    pub vector: Vector,
    pub owner_pk: String,
    pub tau: VectorTypeTag,
    pub meta: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpaceId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EntityId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectionId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContractId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecordId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum OperationKind {
    Create,
    Certify,
    Transfer,
    Drain,
    Project,
    Reconstruct,
    Query,
    Record,
    Move,
    Rotate,
    Scale,
    Normalize,
    Constrain,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RegionId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContractEnvironmentId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProofReference(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SignatureReference(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct JsonEnvelope(pub serde_json::Value);
