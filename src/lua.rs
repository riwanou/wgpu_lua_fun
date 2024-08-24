use std::{ops::Deref, time::Instant};

use anyhow::Result;
use assets_manager::{
    loader::{self},
    Asset, AssetCache,
};
use log::info;
use mlua::{Compiler, Function, Lua};

use crate::app::RELOAD_DEBOUNCE;

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
    /// This should contains a global update function.
    pub fn new(entry_point: &str) -> Result<Self> {
        let lua = Lua::new();
        lua.set_compiler(Compiler::new().set_type_info_level(1));

        let my_table = lua.create_table()?;
        my_table.set(1, "one")?;
        my_table.set("two", 2)?;
        lua.globals().set("my_table", my_table)?;

        let cache = AssetCache::new("assets/scripts")?;
        {
            info!("Loading luau script from {}.luau", entry_point);
            let handle = cache.load_expect::<LuauScript>(entry_point);
            Self::load_entry_point(&lua, handle.read().0.deref())?;
        }

        Ok(Self {
            cache,
            entry_point: entry_point.to_string(),
            last_reload: Instant::now(),
            lua,
        })
    }

    fn load_entry_point(lua: &Lua, data: &str) -> Result<()> {
        lua.load(data).set_name("entry_point").exec()?;
        Ok(())
    }

    pub fn update(&mut self, delta_sec: f32) -> Result<()> {
        self.cache.hot_reload();

        let handle = self.cache.load_expect::<LuauScript>(&self.entry_point);
        if self.last_reload.elapsed() >= RELOAD_DEBOUNCE {
            self.last_reload = Instant::now();
            if handle.reloaded_global() {
                info!("Reloading luau script from {}.luau", self.entry_point);
                Self::load_entry_point(&self.lua, handle.read().0.deref())?;
            }
        }

        let update_fn = self.lua.globals().get::<_, Function>("update")?;
        update_fn.call::<_, ()>(delta_sec)?;

        Ok(())
    }
}
