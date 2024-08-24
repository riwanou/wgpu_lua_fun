# WGPU with Lua

Simple lua embedding with wgpu to render stuffs.

## Luau

Roblox [Luau](https://luau.org) is used to get native type annotations. 
Type declarations can be found in `definition.d.lua`, to make it work with [luau-lsp](https://github.com/JohnnyMorganz/luau-lsp), add `--definitions=definition.d.lua` to the lsp args.

## Xcode debugging with the graphic debugger 

- Create a new XCODE project, select external build tool, add the executable (more details in the [wgpu docs](https://github.com/gfx-rs/wgpu/wiki/Debugging-with-Xcode))
- Edit scheme -> env variables
  - `DYLD_LIBRARY_PATH=rustc --print target-libdir`: rpath issues
  - `IDEPreferLogStreaming=YES`: logs
  - `NOT_ON_TOP=1`: do not show window on top
- Edit scheme -> Options -> Use custom working directory -> project directory


