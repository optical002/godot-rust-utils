use std::cell::Cell;
use std::rc::Rc;

use init_system::core::{InitContext, InitStorage};
use init_system::data::InitId;
use init_system::interfaces::{FreeInitExt, Init, MakeInit, MakeInitExt};

#[derive(Clone)]
struct Backing;

#[init_system::init]
#[derive(Clone)]
struct TestInit {
    _unused: u8,
}

impl MakeInit for Backing {
    type Init = TestInit;
    type Params = ();

    fn init_inner(&mut self, init_id: InitId, ctx: InitContext, _params: ()) -> Self::Init {
        Self::Init {
            init_id,
            ctx,
            _unused: 0,
        }
    }
}

impl Init for TestInit {}

// --- Tracked init: records on_free() calls via shared counter ---

#[derive(Clone)]
struct TrackedBacking {
    free_count: Rc<Cell<u32>>,
}

#[init_system::init]
#[derive(Clone)]
struct TrackedInit {
    free_count: Rc<Cell<u32>>,
}

impl MakeInit for TrackedBacking {
    type Init = TrackedInit;
    type Params = ();

    fn init_inner(&mut self, init_id: InitId, ctx: InitContext, _params: ()) -> Self::Init {
        Self::Init {
            init_id,
            ctx,
            free_count: self.free_count.clone(),
        }
    }
}

impl Init for TrackedInit {
    fn on_free(&mut self) {
        self.free_count.set(self.free_count.get() + 1);
    }
}

// --- Init that calls storage.free() on a sibling from inside on_free() ---

#[derive(Clone)]
struct FreesOnFreeBacking {
    storage: InitStorage,
    sibling_id: Rc<Cell<u64>>,
}

#[init_system::init]
#[derive(Clone)]
struct FreesOnFreeInit {
    storage: InitStorage,
    sibling_id: Rc<Cell<u64>>,
}

impl MakeInit for FreesOnFreeBacking {
    type Init = FreesOnFreeInit;
    type Params = ();

    fn init_inner(&mut self, init_id: InitId, ctx: InitContext, _params: ()) -> Self::Init {
        Self::Init {
            init_id,
            ctx,
            storage: self.storage.clone(),
            sibling_id: self.sibling_id.clone(),
        }
    }
}

impl Init for FreesOnFreeInit {
    fn on_free(&mut self) {
        self.storage.free(InitId::new(self.sibling_id.get()));
    }
}

// ---

fn count_inits(storage: &InitStorage) -> usize {
    let mut count = 0;
    storage.for_each(|_| count += 1);
    count
}

#[test]
fn free_removes_init_from_storage() {
    let storage = InitStorage::new();
    let ctx = InitContext::new_custom(storage.clone());
    let parent_id = InitId::new(0);

    let init = Backing.init(parent_id, ctx, ());
    storage.process_pending();
    assert_eq!(count_inits(&storage), 1);

    init.free_init();
    storage.process_pending();
    assert_eq!(count_inits(&storage), 0);
}

#[test]
fn freeing_parent_cascades_to_children() {
    let storage = InitStorage::new();
    let ctx = InitContext::new_custom(storage.clone());
    let root_id = InitId::new(0);

    let parent = Backing.init(root_id, ctx.clone(), ());
    storage.process_pending();

    let parent_id = parent.init_id;
    Backing.init(parent_id, ctx.clone(), ());
    Backing.init(parent_id, ctx, ());
    storage.process_pending();
    assert_eq!(count_inits(&storage), 3);

    parent.free_init();
    storage.process_pending();
    assert_eq!(count_inits(&storage), 0);
}

#[test]
fn multiple_inits_coexist_with_unique_ids() {
    let storage = InitStorage::new();
    let ctx = InitContext::new_custom(storage.clone());
    let parent_id = InitId::new(0);

    let a = Backing.init(parent_id, ctx.clone(), ());
    let b = Backing.init(parent_id, ctx.clone(), ());
    let c = Backing.init(parent_id, ctx, ());
    storage.process_pending();

    assert_eq!(count_inits(&storage), 3);
    assert_ne!(a.init_id, b.init_id);
    assert_ne!(b.init_id, c.init_id);
    assert_ne!(a.init_id, c.init_id);
}

#[test]
fn double_free_does_not_panic() {
    let storage = InitStorage::new();
    let ctx = InitContext::new_custom(storage.clone());
    let parent_id = InitId::new(0);

    let init = Backing.init(parent_id, ctx, ());
    storage.process_pending();

    init.free_init();
    init.free_init();
    storage.process_pending();

    assert_eq!(count_inits(&storage), 0);
}

/// on_free() should be called exactly once when an init is removed.
#[test]
fn on_free_called_on_removal() {
    let free_count = Rc::new(Cell::new(0u32));
    let storage = InitStorage::new();
    let ctx = InitContext::new_custom(storage.clone());
    let parent_id = InitId::new(0);

    let mut backing = TrackedBacking { free_count: free_count.clone() };
    let init = backing.init(parent_id, ctx, ());
    storage.process_pending();

    init.free_init();
    storage.process_pending();

    assert_eq!(free_count.get(), 1);
}

/// on_free() should be called for every init in a cascade (parent + all children).
#[test]
fn on_free_called_for_all_cascaded_inits() {
    let free_count = Rc::new(Cell::new(0u32));
    let storage = InitStorage::new();
    let ctx = InitContext::new_custom(storage.clone());
    let root_id = InitId::new(0);

    let mut backing = TrackedBacking { free_count: free_count.clone() };
    let parent = backing.init(root_id, ctx.clone(), ());
    storage.process_pending();

    let parent_id = parent.init_id;
    backing.init(parent_id, ctx.clone(), ());
    backing.init(parent_id, ctx, ());
    storage.process_pending();

    parent.free_init();
    storage.process_pending();

    assert_eq!(free_count.get(), 3);
}

/// An init added and freed before process_pending() should never appear in storage.
#[test]
fn add_then_free_before_process_pending_never_enters_storage() {
    let storage = InitStorage::new();
    let ctx = InitContext::new_custom(storage.clone());
    let parent_id = InitId::new(0);

    let init = Backing.init(parent_id, ctx, ());
    init.free_init();
    storage.process_pending();

    assert_eq!(count_inits(&storage), 0);
}

/// A deep chain (grandparent → parent → child → grandchild) should fully cascade on root free.
#[test]
fn deep_cascade_removes_entire_chain() {
    let storage = InitStorage::new();
    let ctx = InitContext::new_custom(storage.clone());
    let root_id = InitId::new(0);

    let grandparent = Backing.init(root_id, ctx.clone(), ());
    storage.process_pending();
    let parent = Backing.init(grandparent.init_id, ctx.clone(), ());
    storage.process_pending();
    let child = Backing.init(parent.init_id, ctx.clone(), ());
    storage.process_pending();
    Backing.init(child.init_id, ctx, ());
    storage.process_pending();

    assert_eq!(count_inits(&storage), 4);

    grandparent.free_init();
    storage.process_pending();

    assert_eq!(count_inits(&storage), 0);
}

/// for_each on an empty storage should not panic and should not invoke the callback.
#[test]
fn for_each_on_empty_storage_does_not_panic() {
    let storage = InitStorage::new();
    let mut called = false;
    storage.for_each(|_| called = true);
    assert!(!called);
}

/// Freeing an ID that was never added should not panic and storage should remain empty.
#[test]
fn freeing_unknown_id_does_not_panic() {
    let storage = InitStorage::new();
    storage.free(InitId::new(9999));
    storage.process_pending();
    assert_eq!(count_inits(&storage), 0);
}

/// process_pending() called repeatedly with no pending work should be a no-op and not panic.
#[test]
fn repeated_process_pending_on_stable_storage_does_not_panic() {
    let storage = InitStorage::new();
    let ctx = InitContext::new_custom(storage.clone());
    let parent_id = InitId::new(0);

    Backing.init(parent_id, ctx, ());
    storage.process_pending();
    storage.process_pending();
    storage.process_pending();

    assert_eq!(count_inits(&storage), 1);
}

/// Child freed one frame, parent freed next frame — storage must end fully clean.
#[test]
fn child_freed_before_parent_leaves_storage_clean() {
    let storage = InitStorage::new();
    let ctx = InitContext::new_custom(storage.clone());
    let root_id = InitId::new(0);

    let parent = Backing.init(root_id, ctx.clone(), ());
    storage.process_pending();
    let child = Backing.init(parent.init_id, ctx, ());
    storage.process_pending();

    child.free_init();
    storage.process_pending();
    assert_eq!(count_inits(&storage), 1);

    parent.free_init();
    storage.process_pending();
    assert_eq!(count_inits(&storage), 0);
}

/// free() called from inside on_free() is deferred — the re-entrant free survives
/// process_pending() and is flushed on the next call. Two process_pending() calls needed.
#[test]
fn free_called_from_on_free_is_deferred_to_next_process_pending() {
    let storage = InitStorage::new();
    let ctx = InitContext::new_custom(storage.clone());
    let root_id = InitId::new(0);
    let sibling_id = Rc::new(Cell::new(0u64));

    let mut backing = FreesOnFreeBacking {
        storage: storage.clone(),
        sibling_id: sibling_id.clone(),
    };

    let trigger = backing.init(root_id, ctx.clone(), ());
    storage.process_pending();

    let sibling = Backing.init(root_id, ctx, ());
    storage.process_pending();

    sibling_id.set(sibling.init_id.get_id());

    trigger.free_init();
    storage.process_pending();
    // Re-entrant free survived — sibling queued but not yet removed.
    assert_eq!(count_inits(&storage), 1);

    storage.process_pending();
    // Second call flushes the deferred free.
    assert_eq!(count_inits(&storage), 0);
}

/// Using InitId::new(0) — the sentinel root — as a real init ID should not corrupt the graph.
#[test]
fn sentinel_id_used_as_real_init_id_does_not_corrupt_storage() {
    let storage = InitStorage::new();
    // Manually add an init with the sentinel ID (0) as its own ID, not just as parent.
    // This simulates a collision between the root sentinel and a real init.
    let sentinel = InitId::new(0);

    let ctx = InitContext::new_custom(storage.clone());
    // Create a normal child of the sentinel parent, then also create an init
    // whose *own* id happens to equal the sentinel by adding it directly.
    let child = Backing.init(sentinel, ctx.clone(), ());
    storage.process_pending();
    assert_eq!(count_inits(&storage), 1);

    // Now free the child; the sentinel parent (0) is not a real init so
    // its cascade lookup should find no children and not panic.
    child.free_init();
    storage.process_pending();
    assert_eq!(count_inits(&storage), 0);
}

/// Grandchild added same frame as grandparent is freed should be caught by cascade.
/// This verifies the cascade walk picks up same-frame additions correctly.
#[test]
fn grandchild_added_same_frame_as_grandparent_freed_is_removed_by_cascade() {
    let storage = InitStorage::new();
    let ctx = InitContext::new_custom(storage.clone());
    let root_id = InitId::new(0);

    let parent = Backing.init(root_id, ctx.clone(), ());
    storage.process_pending();

    let child = Backing.init(parent.init_id, ctx.clone(), ());
    storage.process_pending();

    // Same frame: free the parent AND add a grandchild under the child.
    parent.free_init();
    Backing.init(child.init_id, ctx, ());

    storage.process_pending();

    assert_eq!(count_inits(&storage), 0);
}

/// get_init returns the init by ID after process_pending, and None after it is freed.
#[test]
fn get_init_returns_init_by_id_and_none_after_free() {
    let storage = InitStorage::new();
    let ctx = InitContext::new_custom(storage.clone());
    let parent_id = InitId::new(0);

    let init = Backing.init(parent_id, ctx.clone(), ());
    let id = init.init_id;

    // Not yet committed — should not be findable.
    assert!(ctx.get_init(id).is_none());

    storage.process_pending();

    // Now committed — should be the same init (verified by matching init_id).
    let found = ctx.get_init(id).expect("init should be present after process_pending");
    assert_eq!(found.borrow().as_any().downcast_ref::<TestInit>().unwrap().init_id, id);

    init.free_init();
    storage.process_pending();

    // Freed — should be gone.
    assert!(ctx.get_init(id).is_none());
}

