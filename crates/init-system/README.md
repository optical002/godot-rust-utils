# init-system

A lifecycle and initialization management system for Godot 4 games built with [godot-rust](https://github.com/godot-rust/gdext).

Decouples game object logic from Godot nodes by letting you define plain Rust structs as "inits" — objects that receive Godot lifecycle callbacks (`ready`, `process`, `physics_process`, `input`, etc.) without being Godot nodes themselves. An `InitSystemNode` placed in the scene tree drives the lifecycle for all registered inits.

## Concepts

**`Init` trait** — implement this on a struct to make it a managed game object. All lifecycle methods are optional with default no-op implementations.

**`InitContext`** — a shared handle to the storage that holds all registered inits. Clone and pass it freely; it is backed by `Rc<RefCell<_>>` and stays cheap to clone.

**`InitId`** — a `u64`-backed unique identifier for each init. Inits are registered under a parent id, forming a tree — freeing a parent cascades to all its children.

**`#[init]` macro** — attribute macro that auto-inserts `init_id` and `ctx` fields into a struct and derives the `GetInitId` and `GetInitContext` traits.

**`InitSystemNode`** — a Godot `Node` subclass. Add one to your scene tree; it holds an `InitContext` and forwards all Godot lifecycle events to every registered init each frame.

## Usage

### 1. Add to `Cargo.toml`

```toml
[dependencies]
init-system = "0.1"
```

### 2. Define an init struct

```rust
use init_system::prelude::*;

#[init_system::init]  // injects init_id and ctx fields, derives GetInitId + GetInitContext
#[derive(Clone)]
pub struct PlayerInit {
    pub speed: f32,
}

impl Init for PlayerInit {
    fn ready(&mut self) {
        // called when the parent InitSystemNode is ready
    }

    fn process(&mut self, delta: f64) {
        // called every frame
    }
}
```

The `#[init]` macro requires:
- The struct to have named fields
- The struct name to end with `Init`
- The struct to implement `Clone`

### 3. Implement `MakeInit` on the factory

```rust
use init_system::prelude::*;

pub struct PlayerFactory {
    speed: f32,
}

impl MakeInit for PlayerFactory {
    type Init = PlayerInit;
    type Params = ();

    fn init_inner(&mut self, init_id: InitId, ctx: InitContext, _params: ()) -> PlayerInit {
        PlayerInit { init_id, ctx, speed: self.speed }
    }
}
```

### 4. Register an init

```rust
// inside a Godot node that has access to an InitContext
let ctx = init_system_node.bind().init_ctx.clone();
let parent_id = InitId::new(0);  // root parent

let player = factory.init(parent_id, ctx, ());
// player is now registered and will receive lifecycle callbacks
```

### 5. Free an init

```rust
use init_system::prelude::*;

player.free_init();  // calls on_free(), removes from storage, cascades to children
```

Removal is deferred to the next `process_pending()` call (which `InitSystemNode` runs at the start of each lifecycle phase), so it is safe to call `free_init()` from within a lifecycle callback.

## Scene setup

Add an `InitSystemNode` to your scene tree. It is a plain Godot `Node` — no configuration needed. Access its `init_ctx` field from any other node to register inits:

```rust
let node: Gd<InitSystemNode> = get_node_as("InitSystemNode");
let ctx = node.bind().init_ctx.clone();
```

## Init ID for Godot nodes

When the factory itself is a `Gd<T>`, calling `.init()` on it automatically uses the Godot instance ID as the `InitId`, ensuring a stable identity tied to the node's lifetime:

```rust
let mut my_node: Gd<MyNode> = ...;
let init = my_node.init(parent_id, ctx, params);
```

## Testing

Tests that use Godot types (`GodotClass`, `Gd<T>`) require a running Godot engine and cannot be compiled in a headless environment. Run the full test suite locally with the Godot binary available. The CI pipeline runs `cargo check` and `cargo clippy` for this crate but skips `cargo test`.

## License

MIT
