# gd-init-system-macros

Procedural macros for [`gd-init-system`](../init-system). This crate is a companion and not intended to be used directly — depend on `gd-init-system` instead.

## `#[init]`

Attribute macro applied to structs that participate in the init-system lifecycle. It automatically injects two fields and derives the required traits:

```rust
use gd_init_system::prelude::*;

#[gd_init_system::init]
#[derive(Clone)]
pub struct PlayerInit {
    pub speed: f32,
    // init_id: InitId  ← injected
    // ctx: InitContext ← injected
}

impl Init for PlayerInit {
    fn process(&mut self, delta: f64) { /* ... */ }
}
```

**Requirements:**
- Must be applied to a struct with named fields
- Struct name must end with `Init`
- Struct must implement `Clone` and `Init`

## License

MIT
