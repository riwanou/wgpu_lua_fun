use std::{
    borrow::Cow,
    ops::Deref,
    time::{Duration, Instant},
};

use assets_manager::{loader::Loader, Asset, AssetCache, BoxedError};
use log::info;
use mlua::{Compiler, Function, Lua};

struct LuauScriptLoader;
impl Loader<LuauScript> for LuauScriptLoader {
    #[inline]
    fn load(content: Cow<[u8]>, _: &str) -> Result<LuauScript, BoxedError> {
        Ok(LuauScript(String::from_utf8(content.into_owned())?))
    }
}

struct LuauScript(String);
impl Asset for LuauScript {
    const EXTENSION: &'static str = "luau";
    type Loader = LuauScriptLoader;
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
    pub fn new(entry_point: &str) -> mlua::Result<Self> {
        let lua = Lua::new();
        lua.set_compiler(Compiler::new().set_type_info_level(1));

        let my_table = lua.create_table()?;
        my_table.set(1, "one")?;
        my_table.set("two", 2)?;
        lua.globals().set("my_table", my_table)?;

        let cache = AssetCache::new("assets/scripts").unwrap();
        {
            info!("Loading luau script from {}.luau", entry_point);
            let data = cache.load_expect::<LuauScript>(entry_point).read();
            Self::load_entry_point(&lua, data.0.deref()).unwrap();
        }

        Ok(Self {
            cache,
            entry_point: entry_point.to_string(),
            last_reload: Instant::now(),
            lua,
        })
    }

    fn load_entry_point(lua: &Lua, data: &str) -> mlua::Result<()> {
        lua.load(data).set_name("entry_point").exec()
    }

    pub fn update(&mut self, delta_sec: f32) -> mlua::Result<()> {
        self.cache.hot_reload();
        let handle = self
            .cache
            .get_cached::<LuauScript>(&self.entry_point)
            .unwrap();
        if handle.reloaded_global() {
            // small debounce to avoid falsy reload (happens on my machine)
            if self.last_reload.elapsed() >= Duration::from_millis(100) {
                self.last_reload = Instant::now();
                info!("Reloading luau script from {}.luau", self.entry_point);
                let data = handle.read();
                Self::load_entry_point(&self.lua, data.0.deref()).unwrap();
            }
        }

        let update_fn = self.lua.globals().get::<_, Function>("update")?;
        update_fn.call::<_, ()>(delta_sec)?;

        Ok(())
    }
}
