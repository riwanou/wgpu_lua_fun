use std::{ops::Deref, time::Instant};

use anyhow::Result;
use assets_manager::{
    loader::{self},
    Asset, AssetCache,
};
use log::info;
use mlua::{Compiler, Function, Lua};
use scene::register_types;

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
}

impl LuaState {
    /// Load lua script entrypoint, will get hot-reloaded.
    /// This should contains a global update and init function.
    pub fn new(entry_point: &str) -> Self {
        let lua = Lua::new();
        lua.set_compiler(Compiler::new().set_type_info_level(1));

        let cache = AssetCache::new("assets/scripts").unwrap();

        {
            info!("Loading luau script from {}.luau", entry_point);
            let handle = cache.load_expect::<LuauScript>(entry_point);
            Self::load_entry_point(&lua, handle.read().0.deref()).unwrap();
        }

        register_types(&lua).unwrap();

        Self {
            cache,
            entry_point: entry_point.to_string(),
            last_reload: Instant::now(),
            lua,
        }
    }

    pub fn init(&mut self, render_state: &mut RenderState) -> Result<()> {
        let globals = self.lua.globals();

        info!("Running luau init function");
        self.lua.scope(|scope| {
            let camera = scope
                .create_any_userdata_ref_mut(&mut render_state.scene.camera)?;
            let init_fn = globals.get::<_, Function>("init")?;
            init_fn.call::<_, ()>(camera)
        })?;

        Ok(())
    }

    fn load_entry_point(lua: &Lua, data: &str) -> Result<()> {
        lua.load(data).set_name("entry_point").exec()?;
        Ok(())
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
                info!("Reloading luau script from {}.luau", self.entry_point);
                Self::load_entry_point(&self.lua, handle.read().0.deref())?;
            }
        }

        let globals = self.lua.globals();
        self.lua.scope(|scope| {
            let camera = scope
                .create_any_userdata_ref_mut(&mut render_state.scene.camera)?;
            let update_fn = globals.get::<_, Function>("update")?;
            update_fn.call::<_, ()>((delta_sec, camera))
        })?;

        Ok(())
    }
}
