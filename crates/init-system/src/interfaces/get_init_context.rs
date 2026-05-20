use crate::core::InitContext;

pub trait GetInitContext {
    fn get_ctx(&self) -> InitContext;
}
