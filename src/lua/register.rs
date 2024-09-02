use std::{
    fmt,
    ops::{Add, Div, Mul, Sub},
};

use anyhow::Result;
use glam::Vec3;
use log::info;
use mlua::{
    AnyUserData, Function, Lua, MetaMethod, Scope, Table, UserDataFields,
    UserDataMethods, UserDataRef, UserDataRegistry, Variadic,
};

use crate::{
    register_fields, register_getters, register_methods_mut,
    register_to_string,
    render::{
        bundle::model::{self},
        camera::Camera,
        state::RenderState,
    },
    scene::Scene,
    transform::Transform,
};

fn register_vec3_methods_mut<T: std::borrow::BorrowMut<Vec3> + fmt::Debug>(
    reg: &mut UserDataRegistry<T>,
) {
    register_fields!(reg, T, { x, y, z });
    register_to_string!(reg);
    let mut reg_meta_op = |method: MetaMethod, op: fn(Vec3, Vec3) -> Vec3| {
        reg.add_meta_function(
            method,
            move |_, (this, other): (UserDataRef<Vec3>, UserDataRef<Vec3>)| {
                Ok(AnyUserData::wrap(op(*this, *other)))
            },
        );
    };
    reg_meta_op(MetaMethod::Add, Vec3::add);
    reg_meta_op(MetaMethod::Sub, Vec3::sub);
    reg_meta_op(MetaMethod::Mul, Vec3::mul);
    reg_meta_op(MetaMethod::Div, Vec3::div);
}

fn register_vec3(lua: &Lua) -> Result<()> {
    register_methods_mut!(lua, Vec3, register_vec3_methods_mut);
    let table = lua.create_table()?;
    table.set(
        "new",
        lua.create_function(|_, (x, y, z): (f32, f32, f32)| {
            Ok(AnyUserData::wrap(Vec3::new(x, y, z)))
        })?,
    )?;
    table.set(
        "splat",
        lua.create_function(|_, val: f32| {
            Ok(AnyUserData::wrap(Vec3::splat(val)))
        })?,
    )?;
    table.set("X", AnyUserData::wrap(Vec3::X))?;
    table.set("Y", AnyUserData::wrap(Vec3::Y))?;
    table.set("Z", AnyUserData::wrap(Vec3::Z))?;
    lua.globals().set("Vec3", table)?;
    Ok(())
}

fn register_transform_methods_mut<
    T: std::borrow::BorrowMut<Transform> + fmt::Debug,
>(
    reg: &mut UserDataRegistry<T>,
) {
    register_fields!(reg, T, {}, userdata: { pos: Vec3, scale: Vec3 });
    register_to_string!(reg);
    reg.add_method_mut(
        "rotate",
        |_, this, (axis, angle): (UserDataRef<Vec3>, f32)| {
            this.borrow_mut().rotate(*axis, angle);
            Ok(())
        },
    );
}

fn register_transform(lua: &Lua) -> Result<()> {
    register_methods_mut!(lua, Transform, register_transform_methods_mut);
    let table = lua.create_table()?;
    table.set(
        "new",
        lua.create_function(|_, pos: UserDataRef<Vec3>| {
            Ok(AnyUserData::wrap(Transform::from_pos(*pos)))
        })?,
    )?;
    lua.globals().set("Transform", table)?;
    Ok(())
}

fn register_camera_methods_mut<
    T: std::borrow::BorrowMut<Camera> + fmt::Debug,
>(
    reg: &mut UserDataRegistry<T>,
) {
    register_fields!(reg, T, { fovy }, userdata: { transform: Transform });
}

fn register_camera(lua: &Lua) -> Result<()> {
    {
        lua.register_userdata_type::<Camera>(register_camera_methods_mut)?;
        lua.register_userdata_type::<&mut Camera>(register_camera_methods_mut)?;
    };
    Ok(())
}

fn register_scene_methods_mut<T: std::borrow::BorrowMut<Scene> + fmt::Debug>(
    reg: &mut UserDataRegistry<T>,
) {
    register_getters!(reg, T, {}, userdata: { camera: Camera });
    reg.add_method_mut(
        "batch_model",
        |_,
         this,
         (mesh_id, texture_id, transform): (
            String,
            Option<String>,
            UserDataRef<Transform>,
        )| {
            this.borrow_mut().model_batches.add_model(
                mesh_id,
                texture_id
                    .unwrap_or(model::DEFAULT_DIFFUSE_TEXTURE.to_string()),
                model::Instance::new(transform.build_matrix(), transform.rot),
            );
            Ok(())
        },
    );
}

fn register_scene(lua: &Lua) -> Result<()> {
    register_methods_mut!(lua, Scene, register_scene_methods_mut);
    Ok(())
}

fn register_render_state(lua: &Lua) -> Result<()> {
    lua.register_userdata_type::<RenderState>(|reg| {
        reg.add_method_mut("load_mesh", |_, this, mesh_id: String| {
            this.meshes.load(&mesh_id);
            Ok(())
        });
        reg.add_method_mut("load_texture", |_, this, texture_id: String| {
            this.textures.load(&texture_id);
            Ok(())
        });
    })?;
    Ok(())
}

fn register_cached_tables(lua: &Lua) -> Result<()> {
    lua.set_named_registry_value("cached_tables", lua.create_table()?)?;
    lua.globals().set(
        "cached_table",
        lua.create_function(|lua, id: String| {
            let cached_tables =
                lua.named_registry_value::<Table>("cached_tables")?;
            if !cached_tables.contains_key(id.clone())? {
                cached_tables
                    .raw_set(id.clone(), lua.create_table()?.clone())?;
            }
            cached_tables.raw_get::<_, Table>(id)
        })?,
    )?;

    Ok(())
}

pub fn create_scoped_context<'scope>(
    lua: &'scope Lua,
    scope: &Scope<'_, 'scope>,
    render_state: &'scope mut RenderState,
    scene: &'scope mut Scene,
) -> mlua::Result<Table<'scope>> {
    let ctx = lua.create_table()?;
    ctx.set("graphics", scope.create_any_userdata_ref_mut(render_state)?)?;
    ctx.set("scene", scope.create_any_userdata_ref_mut(scene)?)?;
    Ok(ctx)
}

pub fn register_types_globals(lua: &Lua) -> Result<()> {
    register_vec3(lua)?;
    register_transform(lua)?;
    register_camera(lua)?;
    register_scene(lua)?;
    register_render_state(lua)?;
    register_cached_tables(lua)?;

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
