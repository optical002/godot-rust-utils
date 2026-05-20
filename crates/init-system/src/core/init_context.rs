use std::cell::RefCell;
use std::rc::Rc;

use crate::core::init_storage::InitStorage;
use crate::data::InitId;
use crate::interfaces::Init;

/// A wrapper for not exposing 'InitStorage'.
#[derive(Clone)]
pub struct InitContext {
    pub(crate) storage: InitStorage,
}

impl InitContext {
    pub fn new() -> Self {
        Self::new_custom(InitStorage::new())
    }

    pub fn new_custom(storage: InitStorage) -> Self {
        Self { storage }
    }

    pub fn get_init(&self, id: InitId) -> Option<Rc<RefCell<Box<dyn Init>>>> {
        self.storage.get_init(id)
    }

    pub fn add_init(&self, parent_id: InitId, id: InitId, init: Box<dyn Init>) {
        self.storage.add_init(parent_id, id, init);
    }

    pub fn free(&self, id: InitId) {
        self.storage.free(id);
    }

    pub fn for_each<F>(&self, f: F)
    where
        F: FnMut(&mut dyn Init),
    {
        self.storage.for_each(f);
    }

    pub fn process_pending(&self) {
        self.storage.process_pending();
    }
}

impl Default for InitContext {
    fn default() -> Self {
        Self::new()
    }
}
