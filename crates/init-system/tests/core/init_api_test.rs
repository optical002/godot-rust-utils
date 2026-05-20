use std::cell::Cell;
use std::rc::Rc;

use init_system::core::{InitContext, InitStorage};
use init_system::data::InitId;
use init_system::interfaces::{FreeInitExt, Init, MakeInit, MakeInitExt};

#[derive(Clone)]
struct ApiTestBacking {
    calls: Rc<ApiCalls>,
}

#[derive(Default)]
struct ApiCalls {
    ready: Cell<u32>,
    process: Cell<u32>,
    last_process_delta: Cell<f64>,
    physics_process: Cell<u32>,
    last_physics_delta: Cell<f64>,
    enter_tree: Cell<u32>,
    exit_tree: Cell<u32>,
    on_free: Cell<u32>,
}

#[init_system::init]
#[derive(Clone)]
struct ApiTestInit {
    calls: Rc<ApiCalls>,
}

impl MakeInit for ApiTestBacking {
    type Init = ApiTestInit;
    type Params = ();

    fn init_inner(&mut self, init_id: InitId, ctx: InitContext, _params: ()) -> Self::Init {
        Self::Init {
            init_id,
            ctx,
            calls: self.calls.clone(),
        }
    }
}

impl Init for ApiTestInit {
    fn ready(&mut self) {
        self.calls.ready.set(self.calls.ready.get() + 1);
    }

    fn process(&mut self, delta: f64) {
        self.calls.process.set(self.calls.process.get() + 1);
        self.calls.last_process_delta.set(delta);
    }

    fn physics_process(&mut self, delta: f64) {
        self.calls.physics_process.set(self.calls.physics_process.get() + 1);
        self.calls.last_physics_delta.set(delta);
    }

    fn enter_tree(&mut self) {
        self.calls.enter_tree.set(self.calls.enter_tree.get() + 1);
    }

    fn exit_tree(&mut self) {
        self.calls.exit_tree.set(self.calls.exit_tree.get() + 1);
    }

    fn on_free(&mut self) {
        self.calls.on_free.set(self.calls.on_free.get() + 1);
    }
}

fn setup() -> (InitStorage, Rc<ApiCalls>) {
    let calls = Rc::new(ApiCalls::default());
    let storage = InitStorage::new();
    let ctx = InitContext::new_custom(storage.clone());
    let mut backing = ApiTestBacking { calls: calls.clone() };
    backing.init(InitId::new(0), ctx, ());
    storage.process_pending();
    (storage, calls)
}

#[test]
fn ready_is_called_on_init() {
    let (storage, calls) = setup();
    storage.for_each(|init| init.ready());
    assert_eq!(calls.ready.get(), 1);
}

#[test]
fn ready_is_called_once_per_invocation() {
    let (storage, calls) = setup();
    storage.for_each(|init| init.ready());
    storage.for_each(|init| init.ready());
    assert_eq!(calls.ready.get(), 2);
}

#[test]
fn process_is_called_with_correct_delta() {
    let (storage, calls) = setup();
    storage.for_each(|init| init.process(0.016));
    assert_eq!(calls.process.get(), 1);
    assert!((calls.last_process_delta.get() - 0.016).abs() < f64::EPSILON);
}

#[test]
fn physics_process_is_called_with_correct_delta() {
    let (storage, calls) = setup();
    storage.for_each(|init| init.physics_process(0.032));
    assert_eq!(calls.physics_process.get(), 1);
    assert!((calls.last_physics_delta.get() - 0.032).abs() < f64::EPSILON);
}

#[test]
fn enter_tree_is_called_on_init() {
    let (storage, calls) = setup();
    storage.for_each(|init| init.enter_tree());
    assert_eq!(calls.enter_tree.get(), 1);
}

#[test]
fn exit_tree_is_called_on_init() {
    let (storage, calls) = setup();
    storage.for_each(|init| init.exit_tree());
    assert_eq!(calls.exit_tree.get(), 1);
}

#[test]
fn on_free_is_called_when_init_is_removed() {
    let calls = Rc::new(ApiCalls::default());
    let storage = InitStorage::new();
    let ctx = InitContext::new_custom(storage.clone());
    let mut backing = ApiTestBacking { calls: calls.clone() };
    let init = backing.init(InitId::new(0), ctx, ());
    storage.process_pending();

    init.free_init();
    storage.process_pending();

    assert_eq!(calls.on_free.get(), 1);
}

// Input methods (input, shortcut_input, unhandled_input, unhandled_key_input) require
// constructing Gd<InputEvent> which is only possible inside the Godot engine runtime.
#[test]
#[ignore]
fn input_is_called_on_init() {}

#[test]
#[ignore]
fn shortcut_input_is_called_on_init() {}

#[test]
#[ignore]
fn unhandled_input_is_called_on_init() {}

#[test]
#[ignore]
fn unhandled_key_input_is_called_on_init() {}
