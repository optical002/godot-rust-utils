use godot::obj::WithBaseField;
use godot::prelude::*;
use init_system::core::InitContext;
use init_system::data::InitId;
use init_system::interfaces::{FreeInitExt, Init, MakeInit, MakeInitExt};

#[derive(Clone)]
pub struct MyBacking {
    backing_value: f32,
}

#[init_system::init]
#[derive(Clone)]
pub struct MyInit {
    value: f32,
    backing_value: f32,
}

impl MakeInit for MyBacking {
    type Init = MyInit;
    type Params = f32;

    fn init_inner(
        &mut self,
        init_id: InitId,
        ctx: InitContext,
        params: Self::Params,
    ) -> Self::Init {
        let value = params;
        Self::Init {
            init_id,
            ctx,
            value,
            backing_value: self.backing_value,
        }
    }
}

impl Init for MyInit {}

#[test]
fn test_simple_creation() {
    let ctx = InitContext::new();

    let parent_id = InitId::new(0);
    let mut backing = MyBacking { backing_value: 13. };

    let init_a = backing.clone().init(parent_id, ctx.clone(), 62.);
    let init_b = backing.init(parent_id, ctx, 48.);

    assert_eq!(init_a.value, 62., "Should equal param value");
    assert_eq!(init_a.backing_value, 13., "Should equal backing value");

    assert_eq!(init_b.value, 48., "Should equal param value");
    assert_eq!(init_b.backing_value, 13., "Should equal backing value");

    init_a.free_init();
}

#[derive(GodotClass)]
#[class(base=Node)]
pub struct MyNode {
    base: Base<Node>,
}

#[godot_api]
impl INode for MyNode {
    fn init(base: Base<Node>) -> Self {
        Self { base }
    }
}

#[init_system::init]
#[derive(Clone)]
pub struct MyNodeInit {
    _backing: Gd<MyNode>,
    pub _value: f32,
}

impl MakeInit for MyNode {
    type Init = MyNodeInit;
    type Params = f32;

    fn init_inner(
        &mut self,
        init_id: InitId,
        ctx: InitContext,
        params: Self::Params,
    ) -> Self::Init {
        let value = params;
        let backing = self.base().clone().cast::<MyNode>();

        Self::Init {
            init_id,
            ctx,
            _backing: backing,
            _value: value,
        }
    }
}

impl Init for MyNodeInit {}

// Leaving this test as ignore, because Gd::from_init_fn requires to be ran inside godot engine
// where tests are ran via cargo. Regardless this still gives us ability to check if the code
// compiles even if we do not run it.
#[test]
#[ignore]
fn test_node_simple() {
    let ctx = InitContext::new();
    let parent_id = InitId::new(0);
    let mut node = Gd::from_init_fn(|node| MyNode { base: node });

    node.clone().init(parent_id, ctx.clone(), 63.);
    let init = node.init(parent_id, ctx, 47.);

    init.free_init();
}
