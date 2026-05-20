use crate::data::init_id::InitId;

pub trait GetInitId {
    fn get_id(&self) -> InitId;
}