use std::{
    fmt,
    ops::{Add, Div, Mul, Sub},
    sync::Arc,
};

use glam::{Quat, Vec3};
use log::info;
use mlua::{
    AnyUserData, Error, Function, Lua, MetaMethod, Result, Scope, Table,
    UserDataFields, UserDataMethods, UserDataRef, UserDataRegistry, Value,
    Variadic,
};
use winit::window::{CursorGrabMode, Window};

use crate::{
    input::Inputs,
    register_fields, register_getters, register_methods_mut,
    register_to_string,
    render::{
        bundle::{lights, model},
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
            move |_, (this, other): (UserDataRef<T>, Value)| {
                let this = this.borrow();
                let to_vec3 = |value: f32| Vec3::splat(value);
                let result = match other {
                    Value::UserData(other) => {
                        let other = match other.borrow::<Vec3>() {
                            Ok(borrowed) => *borrowed,
                            Err(_) => **other.borrow::<&mut Vec3>()?,
                        };
                        op(*this, other)
                    }
                    Value::Number(other) => op(*this, to_vec3(other as f32)),
                    Value::Integer(other) => op(*this, to_vec3(other as f32)),
                    _ => {
                        return Err(Error::runtime(
                            "Invalid operand type for Vec3",
                        ))
                    }
                };
                Ok(AnyUserData::wrap(result))
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
    lua.globals().set("Vec3", table)
}

fn register_quat(lua: &Lua) -> Result<()> {
    let table = lua.create_table()?;
    table.set(
        "default",
        lua.create_function(|_, _: ()| Ok(AnyUserData::wrap(Quat::default())))?,
    )?;
    lua.globals().set("Quat", table)
}

fn register_transform_methods_mut<
    T: std::borrow::BorrowMut<Transform> + fmt::Debug,
>(
    reg: &mut UserDataRegistry<T>,
) {
    register_fields!(reg, T, {}, userdata: { pos: Vec3, rot: Quat, scale: Vec3 });
    register_to_string!(reg);
    reg.add_method("forward", |_, this, _: ()| {
        Ok(AnyUserData::wrap(this.borrow().forward()))
    });
    reg.add_method("right", |_, this, _: ()| {
        Ok(AnyUserData::wrap(this.borrow().right()))
    });
    reg.add_method_mut(
        "rotate",
        |_, this, (axis, angle): (UserDataRef<Vec3>, f32)| {
            this.borrow_mut().rotate(*axis, angle);
            Ok(())
        },
    );
    reg.add_method_mut(
        "rotate_local",
        |_, this, (axis, angle): (UserDataRef<Vec3>, f32)| {
            this.borrow_mut().rotate_local(*axis, angle);
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
    lua.globals().set("Transform", table)
}

fn register_camera_methods_mut<
    T: std::borrow::BorrowMut<Camera> + fmt::Debug,
>(
    reg: &mut UserDataRegistry<T>,
) {
    register_fields!(reg, T, { fovy }, userdata: { transform: Transform });
}

fn register_camera(lua: &Lua) -> Result<()> {
    register_methods_mut!(lua, Camera, register_camera_methods_mut);
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
         (mesh_id, texture_id, shader_id, transform): (
            String,
            Option<String>,
            Option<String>,
            UserDataRef<Transform>,
        )| {
            this.borrow_mut().model_batches.add_model(
                mesh_id,
                texture_id
                    .unwrap_or(model::DEFAULT_DIFFUSE_TEXTURE.to_string()),
                shader_id.unwrap_or(model::DEFAULT_SHADER.to_string()),
                model::Instance::new(transform.build_matrix(), transform.rot),
            );
            Ok(())
        },
    );
    reg.add_method_mut(
        "point_light",
        |_, this, (pos, radius): (UserDataRef<Vec3>, f32)| {
            this.borrow_mut()
                .point_lights
                .push(lights::PointLight { pos: *pos, radius });
            Ok(())
        },
    );
}

fn register_scene(lua: &Lua) -> Result<()> {
    register_methods_mut!(lua, Scene, register_scene_methods_mut);
    Ok(())
}

fn register_inputs(lua: &Lua) -> Result<()> {
    lua.register_userdata_type::<Inputs>(|reg| {
        reg.add_method("cursor_in_window", |_, this, _: ()| {
            Ok(this.cursor_in_window)
        });
        reg.add_method("focused", |_, this, _: ()| Ok(this.focused));
        reg.add_method("pressed", |_, this, action: String| {
            Ok(this.action_pressed(&action))
        });
        reg.add_method("just_pressed", |_, this, action: String| {
            Ok(this.action_just_pressed(&action))
        });
        reg.add_method("mouse_pressed", |_, this, button: String| {
            let state = match button.as_bytes() {
                b"left" => this.mouse_pressed(0),
                b"right" => this.mouse_pressed(1),
                _ => return Err(Error::runtime("Invalid mouse button")),
            };
            Ok(state)
        });
        reg.add_method("mouse_just_pressed", |_, this, button: String| {
            let state = match button.as_bytes() {
                b"left" => this.mouse_just_pressed(0),
                b"right" => this.mouse_just_pressed(1),
                _ => return Err(Error::runtime("Invalid mouse button")),
            };
            Ok(state)
        });
        reg.add_method("mouse_delta", |lua, this, _: ()| {
            let delta = this.mouse_delta;
            let table = lua.create_table_from(
                vec![("x", delta.x), ("y", delta.y)].into_iter(),
            )?;
            Ok(table)
        });
    })
}

fn register_window(lua: &Lua) -> Result<()> {
    lua.register_userdata_type::<Arc<Window>>(|reg| {
        reg.add_method("grab_cursor", |_, this, _: ()| {
            this.set_cursor_grab(CursorGrabMode::Locked)
                .map_err(Error::runtime)?;
            this.set_cursor_visible(false);
            Ok(())
        });
        reg.add_method("release_cursor", |_, this, _: ()| {
            this.set_cursor_grab(CursorGrabMode::None)
                .map_err(Error::runtime)?;
            this.set_cursor_visible(true);
            Ok(())
        });
    })
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
        reg.add_method_mut("load_shader", |_, this, shader_id: String| {
            this.shaders.load(&shader_id);
            this.bundles.model.register_shader(&shader_id);
            Ok(())
        });
    })
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
    )
}

pub fn create_scoped_context<'scope>(
    lua: &'scope Lua,
    scope: &Scope<'_, 'scope>,
    scene: &'scope mut Scene,
    inputs: &'scope Inputs,
    window: Arc<Window>,
    render_state: &'scope mut RenderState,
) -> Result<Table<'scope>> {
    let ctx = lua.create_table()?;
    ctx.set("scene", scope.create_any_userdata_ref_mut(scene)?)?;
    ctx.set("inputs", scope.create_any_userdata_ref(inputs)?)?;
    ctx.set("window", scope.create_any_userdata(window)?)?;
    ctx.set("graphics", scope.create_any_userdata_ref_mut(render_state)?)?;
    Ok(ctx)
}

pub fn register_types_globals(lua: &Lua) -> Result<()> {
    register_vec3(lua)?;
    register_quat(lua)?;
    register_transform(lua)?;
    register_camera(lua)?;
    register_scene(lua)?;
    register_inputs(lua)?;
    register_window(lua)?;
    register_render_state(lua)?;
    register_cached_tables(lua)?;

    lua.globals().set(
        "print",
        Function::wrap(|_, vals: Variadic<Value>| {
            let args: Vec<String> =
                vals.iter().map(|val| val.to_string().unwrap()).collect();
            info!("{}", args.join(", "));
            Ok(())
        }),
    )
}
