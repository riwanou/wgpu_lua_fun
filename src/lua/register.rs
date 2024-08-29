use std::ops::{Add, Div, Mul, Sub};

use anyhow::Result;
use glam::Vec3;
use log::info;
use mlua::{
    AnyUserData, Function, Lua, MetaMethod, Table, UserDataFields,
    UserDataMethods, UserDataRef, Variadic,
};

use crate::{
    render::{
        bundle::model, camera::Camera, mesh::MeshAssets, state::RenderState,
    },
    scene::Scene,
    transform::Transform,
};

use super::shared::Shared;

macro_rules! register_getters_shared {
    ($reg:expr, { $( $field:ident ),* } $( ,any: { $( $any_field:ident ),* } )?) => {
        $(
            $reg.add_field_method_get(stringify!($field), |_, this|
                Ok(this.borrow().$field)
            );
        )*
        $(
            $(
                $reg.add_field_method_get(stringify!($any_field), |_, this| {
                    Ok(AnyUserData::wrap(this.borrow().$any_field.clone()))
                });
            )*
        )?
    };
}

macro_rules! register_setters_shared {
    ($reg:expr, { $( $field:ident ),* } $( ,any: { $( $any_field:ident : $any_field_type:ty ),* } )?) => {
        $(
            $reg.add_field_method_set(stringify!($field), |_, this, val| {
                this.borrow_mut().$field = val;
                Ok(())
            });
        )*
        $(
            $(
                $reg.add_field_method_set(stringify!($any_field), |_, this, val: UserDataRef<$any_field_type>| {
                    this.borrow_mut().$any_field = val.clone();
                    Ok(())
                });
            )*
        )?
    };
}

macro_rules! register_meta_string {
    ($reg:expr) => {
        $reg.add_meta_method(MetaMethod::ToString, |_, this, _: ()| {
            Ok(format!("{:?}", this))
        });
    };
}

fn register_vec3(lua: &Lua) -> Result<()> {
    lua.register_userdata_type::<Shared<Vec3>>(|reg| {
        register_getters_shared!(reg, { x, y, z });
        register_setters_shared!(reg, { x, y, z });
        register_meta_string!(reg);
        let mut reg_meta_op =
            |method: MetaMethod, op: fn(Vec3, Vec3) -> Vec3| {
                reg.add_meta_function(
                    method,
                    move |_,
                          (this, other): (
                        UserDataRef<Shared<Vec3>>,
                        UserDataRef<Shared<Vec3>>,
                    )| {
                        Ok(AnyUserData::wrap(Shared::new(op(
                            *this.borrow(),
                            *other.borrow(),
                        ))))
                    },
                );
            };
        reg_meta_op(MetaMethod::Add, Vec3::add);
        reg_meta_op(MetaMethod::Sub, Vec3::sub);
        reg_meta_op(MetaMethod::Mul, Vec3::mul);
        reg_meta_op(MetaMethod::Div, Vec3::div);
    })?;

    let table = lua.create_table()?;
    table.set(
        "new",
        lua.create_function(|_, (x, y, z): (f32, f32, f32)| {
            Ok(AnyUserData::wrap(Shared::new(Vec3::new(x, y, z))))
        })?,
    )?;
    table.set(
        "splat",
        lua.create_function(|_, val: f32| {
            Ok(AnyUserData::wrap(Shared::new(Vec3::splat(val))))
        })?,
    )?;
    lua.globals().set("Vec3", table)?;

    Ok(())
}

fn register_transform(lua: &Lua) -> Result<()> {
    lua.register_userdata_type::<Shared<Transform>>(|reg| {
        register_getters_shared!(reg, {}, any: { pos, scale });
        register_setters_shared!(reg, {}, 
            any: { pos: Shared<Vec3>, scale: Shared<Vec3> });
        register_meta_string!(reg);
        reg.add_method_mut("rotate_x", |_, this, angle: f32| {
            this.borrow_mut().rotate_x(angle);
            Ok(())
        });
        reg.add_method_mut("rotate_y", |_, this, angle: f32| {
            this.borrow_mut().rotate_y(angle);
            Ok(())
        });
        reg.add_method_mut("rotate_z", |_, this, angle: f32| {
            this.borrow_mut().rotate_z(angle);
            Ok(())
        });
    })?;

    let table = lua.create_table()?;
    table.set(
        "new",
        lua.create_function(|_, pos: UserDataRef<Shared<Vec3>>| {
            Ok(AnyUserData::wrap(Shared::new(Transform::from_pos(
                pos.clone(),
            ))))
        })?,
    )?;
    lua.globals().set("Transform", table)?;

    Ok(())
}

fn register_camera(lua: &Lua) -> Result<()> {
    lua.register_userdata_type::<Shared<Camera>>(|reg| {
        register_getters_shared!(reg, { fovy }, any: { transform });
            register_setters_shared!(reg, { fovy }, any: { transform: Shared<Transform> });
    })?;
    Ok(())
}

fn register_meshes(lua: &Lua) -> Result<()> {
    lua.register_userdata_type::<MeshAssets>(|reg| {
        reg.add_method_mut("load", |_, this, mesh_id: String| {
            this.load(&mesh_id);
            Ok(())
        });
    })?;
    Ok(())
}

fn register_game(
    lua: &Lua,
    render_state: Shared<RenderState>,
    scene: Shared<Scene>,
) -> Result<()> {
    let game = lua.create_table()?;

    game.set(
        "load_mesh",
        lua.create_function(move |_, mesh_id: String| {
            render_state.borrow_mut().meshes.load(&mesh_id);
            Ok(())
        })?,
    )?;

    let scene_cloned = scene.clone();
    game.set(
        "render_model",
        lua.create_function(
            move |_,
                  (mesh_id, transform): (
                String,
                UserDataRef<Shared<Transform>>,
            )| {
                let t = transform.borrow();
                scene_cloned.borrow_mut().model_batches.add_model(
                    mesh_id,
                    model::Instance::new(t.build_matrix(), t.rot),
                );
                Ok(())
            },
        )?,
    )?;

    game.set("camera", AnyUserData::wrap(scene.borrow().camera.clone()))?;

    lua.globals().set("game", game)?;

    Ok(())
}

fn register_entities(lua: &Lua) -> Result<()> {
    lua.globals().set("entities", lua.create_table()?)?;
    lua.globals().set(
        "entity",
        lua.create_function(|lua, id: String| {
            let entities = lua.globals().raw_get::<_, Table>("entities")?;
            if !entities.contains_key(id.clone())? {
                entities.raw_set(id.clone(), lua.create_table()?.clone())?;
            }
            entities.raw_get::<_, Table>(id)
        })?,
    )?;

    Ok(())
}

pub fn register_types_globals(
    lua: &Lua,
    render_state: Shared<RenderState>,
    scene: Shared<Scene>,
) -> Result<()> {
    register_vec3(lua)?;
    register_transform(lua)?;
    register_camera(lua)?;
    register_meshes(lua)?;

    register_game(lua, render_state, scene)?;
    register_entities(lua)?;

    lua.globals().set(
        "print",
        Function::wrap(|_, vals: Variadic<mlua::Value>| {
            let args: Vec<String> =
                vals.iter().map(|val| val.to_string().unwrap()).collect();
            info!("{}", args.join(", "));
            Ok(())
        }),
    )?;

    Ok(())
}
