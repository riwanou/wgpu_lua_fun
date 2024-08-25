use std::ops::{Add, Sub};

use anyhow::Result;
use glam::Vec3;
use mlua::{AnyUserData, Lua, MetaMethod, UserDataFields, UserDataMethods};

use crate::render::camera::Camera;

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
                    Ok(AnyUserData::wrap(this.$any_field.clone()))
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
    let table = lua.create_table()?;

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

pub fn register_types(lua: &Lua) -> Result<()> {
    register_vec3(lua)?;
    register_camera(lua)?;
    Ok(())
}
