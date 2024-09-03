use std::{ops::Deref, sync::Arc, time::Instant};

use anyhow::Result;
use assets_manager::{loader, Asset, AssetCache};
use log::error;
use mlua::{Compiler, Function, Lua};
use register::{create_scoped_context, register_types_globals};
use winit::window::Window;

use crate::{
    app::RELOAD_DEBOUNCE, input::Inputs, render::state::RenderState,
    scene::Scene,
};

mod register;
mod utils;

const SCRIPTS_DIR: &str = "assets/scripts";

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

        let cache = AssetCache::new(SCRIPTS_DIR).unwrap();
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

    pub fn init(
        &mut self,
        scene: &mut Scene,
        inputs: &Inputs,
        window: Arc<Window>,
        render_state: &mut RenderState,
    ) -> Result<()> {
        let result = self.lua.scope(|scope| {
            let init_fn = self.lua.globals().get::<_, Function>("init")?;
            let ctx = create_scoped_context(
                &self.lua,
                scope,
                scene,
                inputs,
                window,
                render_state,
            )?;
            init_fn.call::<_, ()>(ctx)?;
            Ok(())
        });
        if let Err(err) = result {
            error!("init\n{}", err);
        }
        Ok(())
    }

    fn load_entry_point(lua: &Lua, data: &str) {
        if let Err(err) = lua.load(data).set_name("entry_point").exec() {
            error!("entry_point\n{}", err.to_string());
        }
    }

    fn any_script_reloaded(&self) -> Result<Option<String>> {
        let scripts_dir = self.cache.load_dir::<LuauScript>("")?.read();
        for script_id in scripts_dir.ids() {
            let reloaded = self
                .cache
                .load_expect::<LuauScript>(script_id)
                .reloaded_global();
            if reloaded {
                return Ok(Some(script_id.to_string()));
            }
        }
        Ok(None)
    }

    pub fn update(
        &mut self,
        scene: &mut Scene,
        inputs: &Inputs,
        window: Arc<Window>,
        render_state: &mut RenderState,
        delta_sec: f32,
        elapsed_sec: f32,
    ) -> Result<()> {
        self.cache.hot_reload();
        let handle = self.cache.load_expect::<LuauScript>(&self.entry_point);

        if self.last_reload.elapsed() >= RELOAD_DEBOUNCE {
            self.last_reload = Instant::now();
            if let Some(script_id) = self.any_script_reloaded()? {
                let mod_name = format!("{}/{}", SCRIPTS_DIR, script_id);
                self.lua.unload(&mod_name)?;
                self.update_got_error = false;
                Self::load_entry_point(&self.lua, handle.read().0.deref());
            }
        }

        if self.update_got_error {
            return Ok(());
        }

        let result = self.lua.scope(|scope| {
            let update_fn = self.lua.globals().get::<_, Function>("update")?;
            let ctx = create_scoped_context(
                &self.lua,
                scope,
                scene,
                inputs,
                window,
                render_state,
            )?;
            update_fn.call::<_, ()>((ctx, delta_sec, elapsed_sec))?;
            Ok(())
        });
        if let Err(err) = result {
            if !self.update_got_error {
                self.update_got_error = true;
                error!("update\n{}", err);
            }
        }

        Ok(())
    }
}
