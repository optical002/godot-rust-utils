use crate::interfaces::{GetInitContext, GetInitId, Init};

pub trait FreeInitExt {
    fn free_init(&self);
}

impl<T> FreeInitExt for T
where
    T: Init + GetInitId + GetInitContext,
{
    fn free_init(&self) {
        self.get_ctx().storage.free(self.get_id());
    }
}
