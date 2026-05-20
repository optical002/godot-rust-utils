use std::cell::RefCell;
use std::collections::hash_map::Entry::Vacant;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

use crate::data::init_id::InitId;

use crate::interfaces::Init;

type InitEntry = Rc<RefCell<Box<dyn Init>>>;
type InitCollection = Rc<RefCell<HashMap<InitId, InitEntry>>>;
type IdsRelationship = Rc<RefCell<HashMap<InitId, Rc<RefCell<Vec<InitId>>>>>>;

#[derive(Eq, PartialEq, Hash, Debug)]
pub struct PendingAdditionKey {
    parent_id: InitId,
    id: InitId,
}

#[derive(Clone)]
pub struct InitStorage {
    init_collection: InitCollection,
    ids_relationship: IdsRelationship,
    pending_addition: Rc<RefCell<HashMap<PendingAdditionKey, Box<dyn Init>>>>,
    pending_removal: Rc<RefCell<HashSet<InitId>>>,
}

impl Default for InitStorage {
    fn default() -> Self {
        Self {
            init_collection: Rc::new(RefCell::new(HashMap::new())),
            ids_relationship: Rc::new(RefCell::new(HashMap::new())),
            pending_addition: Rc::new(RefCell::new(HashMap::new())),
            pending_removal: Rc::new(RefCell::new(HashSet::new())),
        }
    }
}

impl InitStorage {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_init(&self, parent_id: InitId, id: InitId, init: Box<dyn Init>) {
        let key = PendingAdditionKey { parent_id, id };
        let mut pending_addition = self.pending_addition.borrow_mut();
        if let Vacant(entry) = pending_addition.entry(key) {
            entry.insert(init);
        } else {
            tracing::warn!(
                "Trying to add a second time the same init with id: {:?}",
                id
            );
        }
    }

    pub fn free(&self, id: InitId) {
        let mut pending_removal = self.pending_removal.borrow_mut();
        if !pending_removal.insert(id) {
            tracing::warn!(
                "Trying to free a second time the same init with id: {:?}",
                id
            );
        }
    }

    pub fn process_pending(&self) {
        {
            let mut init_collection = self.init_collection.borrow_mut();
            let mut ids_relationships = self.ids_relationship.borrow_mut();
            let mut pending_addition = self.pending_addition.borrow_mut();

            pending_addition.retain(|key, _| !self.pending_removal.borrow().contains(&key.id));

            for (key, init) in pending_addition.drain() {
                let parent_id = key.parent_id;
                let id = key.id;

                if let Vacant(entry) = init_collection.entry(id) {
                    entry.insert(Rc::new(RefCell::new(init)));
                } else {
                    tracing::warn!(
                        "Trying to add init which already exists inside 'init_collections' with id {:?}",
                        id
                    );
                }

                ids_relationships
                    .entry(parent_id)
                    .or_insert_with(|| Rc::new(RefCell::new(Vec::new())))
                    .borrow_mut()
                    .push(id);
            }
        }

        let removal_ids: HashSet<InitId> = {
            let pending_removal = self.pending_removal.borrow();
            let ids_relationship = self.ids_relationship.borrow();

            let mut to_remove: HashSet<InitId> = pending_removal.iter().copied().collect();
            let mut frontier: Vec<InitId> = pending_removal.iter().copied().collect();

            while let Some(id) = frontier.pop() {
                if let Some(children_rc) = ids_relationship.get(&id) {
                    let children: Vec<InitId> = children_rc.borrow().clone();
                    for child in children {
                        if to_remove.insert(child) {
                            frontier.push(child);
                        }
                    }
                }
            }

            to_remove
        };

        {
            let mut init_collection = self.init_collection.borrow_mut();
            for id in &removal_ids {
                if let Some(init_rc) = init_collection.remove(id) {
                    let mut init_ref: std::cell::RefMut<Box<dyn Init>> = init_rc.borrow_mut();
                    init_ref.on_free();
                }
            }
        }

        {
            let mut ids_relationship = self.ids_relationship.borrow_mut();
            for id in &removal_ids {
                ids_relationship.remove(id);
            }
            for (_, children_rc) in ids_relationship.iter() {
                let mut children: std::cell::RefMut<Vec<InitId>> = children_rc.borrow_mut();
                children.retain(|child_id| !removal_ids.contains(child_id));
            }
        }

        // Only remove IDs we processed — anything queued during on_free() survives.
        let mut pending_removal = self.pending_removal.borrow_mut();
        for id in &removal_ids {
            pending_removal.remove(id);
        }
    }

    pub fn get_init(&self, id: InitId) -> Option<Rc<RefCell<Box<dyn Init>>>> {
        self.init_collection.borrow().get(&id).cloned()
    }

    pub fn for_each<F>(&self, mut f: F)
    where
        F: FnMut(&mut dyn Init),
    {
        for init_rc in self.init_collection.borrow().values() {
            let mut init_ref: std::cell::RefMut<Box<dyn Init>> = init_rc.borrow_mut();
            f(init_ref.as_mut());
        }
    }
}
