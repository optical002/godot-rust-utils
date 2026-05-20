use crate::core::InitContext;
use crate::data::init_id::InitId;
use crate::interfaces::Init;
use godot::obj::bounds::DeclUser;
use godot::obj::{Bounds, Gd};
use godot::prelude::*;

pub trait MakeInit {
    type Init;
    type Params;
    fn init_inner(&mut self, init_id: InitId, ctx: InitContext, params: Self::Params)
    -> Self::Init;
}

pub trait MakeInitExt {
    type Init;
    type Params;
    fn init(&mut self, parent_id: InitId, ctx: InitContext, params: Self::Params) -> Self::Init;
}

impl<T> MakeInitExt for T
where
    T: MakeInit,
    T::Init: Init + Clone + 'static,
{
    type Init = T::Init;
    type Params = T::Params;

    fn init(&mut self, parent_id: InitId, ctx: InitContext, params: Self::Params) -> Self::Init {
        let init_id = InitId::random();
        let init = self.init_inner(init_id, ctx.clone(), params);
        ctx.storage
            .add_init(parent_id, init_id, Box::new(init.clone()));
        init
    }
}

impl<T> MakeInitExt for Gd<T>
where
    T: GodotClass + MakeInit + Bounds<Declarer = DeclUser>,
    T::Init: Init + Clone + 'static,
{
    type Init = T::Init;
    type Params = T::Params;

    fn init(&mut self, parent_id: InitId, ctx: InitContext, params: Self::Params) -> Self::Init {
        let init_id = InitId::from_instance_id(self.instance_id());
        let init = self.bind_mut().init_inner(init_id, ctx.clone(), params);
        ctx.storage
            .add_init(parent_id, init_id, Box::new(init.clone()));
        init
    }
}
