use std::any::Any;

use godot::classes::InputEvent;
use godot::obj::Gd;

pub trait Init: Any {
    fn ready(&mut self) {}
    fn process(&mut self, _delta: f64) {}
    fn physics_process(&mut self, _delta: f64) {}
    fn enter_tree(&mut self) {}
    fn exit_tree(&mut self) {}
    fn input(&mut self, _event: Gd<InputEvent>) {}
    fn shortcut_input(&mut self, _event: Gd<InputEvent>) {}
    fn unhandled_input(&mut self, _event: Gd<InputEvent>) {}
    fn unhandled_key_input(&mut self, _event: Gd<InputEvent>) {}
    fn on_free(&mut self) {}
}

impl dyn Init {
    pub fn as_any(&self) -> &dyn Any {
        self
    }
}
