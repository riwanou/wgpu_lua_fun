use std::ops::{Add, Sub};

use anyhow::Result;
use glam::{Mat4, Quat, Vec3};
use log::info;
use mlua::{
    AnyUserData, Function, Lua, MetaMethod, Scope, Table, UserDataFields,
    UserDataMethods, Variadic,
};

use crate::{
    render::{
        bundle::model, camera::Camera, mesh::MeshAssets, state::RenderState,
    },
    scene::Scene,
};

macro_rules! register_getters {
    ($reg:expr, { $( $field:ident ),* } $( ,any: { $( $any_field:ident ),* } )?) => {
        $(
            $reg.add_field_method_get(stringify!($field), |_, this|
                Ok(this.$field)
            );
        )*
        $(
            $(
                $reg.add_field_method_get(stringify!($any_field), |_, this| {
                    Ok(AnyUserData::wrap(this.$any_field))
                });
            )*
        )?
    };
}

macro_rules! register_setters {
    ($reg:expr, { $( $field:ident ),* } $( ,any: { $( $any_field:ident : $any_field_type:ty ),* } )?) => {
        $(
            $reg.add_field_method_set(stringify!($field), |_, this, val| {
                this.$field = val;
                Ok(())
            });
        )*
        $(
            $(
                $reg.add_field_method_set(stringify!($any_field), |_, this, val: AnyUserData| {
                    this.$any_field = *val.borrow::<$any_field_type>()?;
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

macro_rules! register_meta_binary {
    ($reg:expr, $meta_method:ident, $type:ty, $op:expr) => {
        $reg.add_meta_function(
            MetaMethod::$meta_method,
            |_, (lhs, rhs): (AnyUserData, AnyUserData)| {
                let lhs = *lhs.borrow_mut::<$type>()?;
                let rhs = *rhs.borrow_mut::<$type>()?;
                Ok(AnyUserData::wrap($op(lhs, rhs)))
            },
        );
    };
}

fn register_vec3(lua: &Lua) -> Result<()> {
    lua.register_userdata_type::<Vec3>(|reg| {
        register_getters!(reg, { x, y, z });
        register_meta_binary!(reg, Add, Vec3, |lhs: Vec3, rhs: Vec3| {
            lhs.add(rhs)
        });
        register_meta_binary!(reg, Sub, Vec3, |lhs: Vec3, rhs: Vec3| {
            lhs.sub(rhs)
        });
        register_meta_string!(reg);
    })?;

    let table = lua.create_table()?;
    table.set(
        "new",
        lua.create_function(|_, (x, y, z): (f32, f32, f32)| {
            Ok(AnyUserData::wrap(Vec3::new(x, y, z)))
        })?,
    )?;
    lua.globals().set("Vec3", table)?;

    Ok(())
}

fn register_camera(lua: &Lua) -> Result<()> {
    lua.register_userdata_type::<Camera>(|reg| {
        register_getters!(reg, { fovy }, any: { pos });
        register_setters!(reg, { fovy }, any: { pos: Vec3 });
        register_meta_string!(reg);
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

pub fn register_types_globals(lua: &Lua) -> Result<()> {
    lua.globals().set(
        "print",
        Function::wrap(|_, vals: Variadic<mlua::Value>| {
            let args: Vec<String> =
                vals.iter().map(|val| val.to_string().unwrap()).collect();
            info!("{}", args.join(", "));
            Ok(())
        }),
    )?;

    register_vec3(lua)?;
    register_camera(lua)?;
    register_meshes(lua)?;

    register_entities(lua)?;

    Ok(())
}

pub fn create_scene<'lua>(
    lua: &'lua Lua,
    scope: &Scope<'_, 'lua>,
    render_state: &'lua mut RenderState,
    scene: &'lua mut Scene,
) -> mlua::Result<Table<'lua>> {
    let table = lua.create_table()?;

    table.set(
        "camera",
        scope.create_any_userdata_ref_mut(&mut scene.camera)?,
    )?;
    table.set(
        "meshes",
        scope.create_any_userdata_ref_mut(&mut render_state.meshes)?,
    )?;
    table.set(
        "render_model",
        scope.create_function_mut(
            |_, (mesh_id, pos): (String, AnyUserData)| {
                let pos = pos.borrow::<Vec3>()?;
                scene.model_batches.add_model(
                    mesh_id,
                    model::Instance::new(
                        Mat4::from_translation(*pos),
                        Quat::IDENTITY,
                    ),
                );
                Ok(())
            },
        )?,
    )?;

    Ok(table)
}
