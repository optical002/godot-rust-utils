use godot::obj::InstanceId;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct InitId(u64);

impl InitId {
    pub fn new(id: u64) -> Self {
        Self(id)
    }

    pub fn get_id(&self) -> u64 {
        self.0
    }

    pub fn random() -> Self {
        Self(Uuid::new_v4().as_u128() as u64)
    }

    pub fn from_instance_id(instance_id: InstanceId) -> Self {
        instance_id.into()
    }
}

impl From<InstanceId> for InitId {
    fn from(value: InstanceId) -> Self {
        // Safe to use on i64 .into, since InstanceId internally stores it as u64,
        // but does not expose the API to get it.
        InitId::new(value.to_i64() as u64)
    }
}
