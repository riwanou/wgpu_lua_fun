use std::{ops::Deref, time::Instant};

use anyhow::Result;
use assets_manager::{
    loader::{self},
    Asset, AssetCache,
};
use log::error;
use mlua::{Compiler, Function, Lua};
use scene::{create_scene, register_types_globals};

use crate::{app::RELOAD_DEBOUNCE, render::state::RenderState};

mod scene;

struct LuauScript(String);

impl From<String> for LuauScript {
    fn from(value: String) -> Self {
        LuauScript(value)
    }
}

impl Asset for LuauScript {
    const EXTENSION: &'static str = "luau";
    type Loader = loader::LoadFrom<String, loader::StringLoader>;
}

pub struct LuaState {
    cache: AssetCache,
    entry_point: String,
    last_reload: Instant,
    lua: Lua,
    update_got_error: bool,
}

impl LuaState {
    /// Load lua script entrypoint, will get hot-reloaded.
    /// This should contains a global update and init function.
    pub fn new(entry_point: &str) -> Self {
        let lua = Lua::new();
        lua.set_compiler(Compiler::new().set_type_info_level(1));

        register_types_globals(&lua).unwrap();

        let cache = AssetCache::new("assets/scripts").unwrap();
        {
            let handle = cache.load_expect::<LuauScript>(entry_point);
            Self::load_entry_point(&lua, handle.read().0.deref());
        }

        Self {
            cache,
            entry_point: entry_point.to_string(),
            last_reload: Instant::now(),
            lua,
            update_got_error: false,
        }
    }

    pub fn init(&mut self, render_state: &mut RenderState) -> Result<()> {
        let globals = self.lua.globals();

        self.lua.scope(|scope| {
            let scene = create_scene(&self.lua, scope, render_state)?;
            let result: mlua::Result<()> = (|| {
                let init_fn = globals.get::<_, Function>("init")?;
                init_fn.call::<_, ()>(scene)?;
                Ok(())
            })();
            if let Err(err) = result {
                error!("init\n{}", err);
            }
            Ok(())
        })?;

        Ok(())
    }

    fn load_entry_point(lua: &Lua, data: &str) {
        if let Err(err) = lua.load(data).set_name("entry_point").exec() {
            error!("entry_point\n{}", err.to_string());
        }
    }

    pub fn update(
        &mut self,
        render_state: &mut RenderState,
        delta_sec: f32,
    ) -> Result<()> {
        self.cache.hot_reload();
        let handle = self.cache.load_expect::<LuauScript>(&self.entry_point);
        if self.last_reload.elapsed() >= RELOAD_DEBOUNCE {
            self.last_reload = Instant::now();
            if handle.reloaded_global() {
                self.update_got_error = false;
                Self::load_entry_point(&self.lua, handle.read().0.deref());
            }
        }

        if self.update_got_error {
            return Ok(());
        }

        let globals = self.lua.globals();
        self.lua.scope(|scope| {
            let scene = create_scene(&self.lua, scope, render_state)?;
            let result: mlua::Result<()> = (|| {
                let update_fn = globals.get::<_, Function>("update")?;
                update_fn.call::<_, ()>((delta_sec, scene))?;
                Ok(())
            })();
            if let Err(err) = result {
                if !self.update_got_error {
                    self.update_got_error = true;
                    error!("update\n{}", err);
                }
            }
            Ok(())
        })?;

        Ok(())
    }
}
