use godot::classes::{InputEvent, Node};
use godot::obj::Base;
use godot::prelude::*;

use crate::core::InitContext;

#[derive(GodotClass)]
#[class(base=Node)]
pub struct InitSystemNode {
    base: Base<Node>,
    pub init_ctx: InitContext,
}

#[godot_api]
impl INode for InitSystemNode {
    fn init(base: Base<Node>) -> Self {
        Self {
            base,
            init_ctx: InitContext::new(),
        }
    }

    fn ready(&mut self) {
        self.init_ctx.storage.for_each(|init| init.ready());
        self.init_ctx.storage.process_pending();
    }

    fn process(&mut self, delta: f64) {
        self.init_ctx.storage.process_pending();
        self.init_ctx.storage.for_each(|init| init.process(delta));
    }

    fn physics_process(&mut self, delta: f64) {
        self.init_ctx.storage.process_pending();
        self.init_ctx
            .storage
            .for_each(|init| init.physics_process(delta));
    }

    fn input(&mut self, event: Gd<InputEvent>) {
        self.init_ctx
            .storage
            .for_each(|init| init.input(event.clone()));
    }

    fn shortcut_input(&mut self, event: Gd<InputEvent>) {
        self.init_ctx
            .storage
            .for_each(|init| init.shortcut_input(event.clone()));
    }

    fn unhandled_input(&mut self, event: Gd<InputEvent>) {
        self.init_ctx
            .storage
            .for_each(|init| init.unhandled_input(event.clone()));
    }

    fn unhandled_key_input(&mut self, event: Gd<InputEvent>) {
        self.init_ctx
            .storage
            .for_each(|init| init.unhandled_key_input(event.clone()));
    }

    fn enter_tree(&mut self) {
        self.init_ctx.storage.for_each(|init| init.enter_tree());
        self.init_ctx.storage.process_pending();
    }

    fn exit_tree(&mut self) {
        self.init_ctx.storage.for_each(|init| init.exit_tree());
        self.init_ctx.storage.process_pending();
    }
}
