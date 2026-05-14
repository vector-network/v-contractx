use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Role {
    Owner,
    Operator,
    Contract,
    Auditor,
    Validator,
    Admin,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AccessPolicy {
    pub allowed_roles: Vec<Role>,
    pub allow_owner_override: bool,
    pub requires_certification: bool,
}

impl AccessPolicy {
    pub fn open() -> Self {
        Self {
            allowed_roles: vec![Role::Owner, Role::Operator, Role::Contract, Role::Auditor, Role::Validator, Role::Admin],
            allow_owner_override: true,
            requires_certification: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AccessContext {
    pub caller_pk: String,
    pub roles: Vec<Role>,
    pub owner_pk: String,
    pub certification_required: bool,
}

pub fn is_authorized(ctx: &AccessContext, policy: &AccessPolicy) -> bool {
    if policy.allow_owner_override && ctx.caller_pk == ctx.owner_pk {
        return true;
    }

    ctx.roles.iter().any(|role| policy.allowed_roles.contains(role))
}
