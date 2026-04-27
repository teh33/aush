//! The `aush.*` Lua API surface.
//!
//! Every function callable from Lua lives here. They are registered onto a
//! global Lua table named `aush` by [`register_aush_api`].
//!
//! # Available functions
//!
//! | Lua call                              | Description                                    |
//! |---------------------------------------|------------------------------------------------|
//! | `aush.exec(cmd)`                      | Run a shell command, return stdout as string   |
//! | `aush.exec_structured(cmd)`           | Run command, return structured data            |
//! | `aush.json_parse(str)`                | Decode JSON string to Lua table                |
//! | `aush.json_encode(val)`               | Encode Lua value to JSON string                |
//! | `aush.env.get(name)`                  | Read an environment variable                   |
//! | `aush.env.set(name, value)`           | Write an environment variable                  |
//! | `aush.cwd()`                          | Return current working directory               |
//! | `aush.register_builtin(name, spec)`   | Register a custom Lua builtin                  |
//! | `aush.register_prompt(name, fn)`      | Register a prompt segment function             |
//! | `aush.register_completion(name, fn)`  | Register a completion function                 |
//! | `aush.on(event, fn)`                  | Register a shell event hook                    |

use std::process::Command;

use mlua::{Function, Lua, Table, Value as LuaValue};

use crate::lua::bridge::lua_to_value;

/// Register the full `aush.*` API table into the Lua globals.
///
/// Called once by [`LuaRuntime::new`].
pub fn register_aush_api(lua: &Lua) -> mlua::Result<()> {
    let aush = lua.create_table()?;

    register_exec(lua, &aush)?;
    register_exec_structured(lua, &aush)?;
    register_json(lua, &aush)?;
    register_env(lua, &aush)?;
    register_cwd(lua, &aush)?;
    register_hooks(lua, &aush)?;
    register_builtins_api(lua, &aush)?;
    register_prompt_api(lua, &aush)?;
    register_completion_api(lua, &aush)?;

    lua.globals().set("aush", aush)?;
    Ok(())
}

// ---------------------------------------------------------------------------
// exec / exec_structured
// ---------------------------------------------------------------------------

fn register_exec(lua: &Lua, aush: &Table) -> mlua::Result<()> {
    let exec = lua.create_function(|_, cmd: String| {
        let output = Command::new("sh")
            .arg("-c")
            .arg(&cmd)
            .output()
            .map_err(|e| mlua::Error::RuntimeError(e.to_string()))?;
        let stdout = String::from_utf8_lossy(&output.stdout)
            .trim_end()
            .to_string();
        Ok(stdout)
    })?;
    aush.set("exec", exec)?;
    Ok(())
}

fn register_exec_structured(lua: &Lua, aush: &Table) -> mlua::Result<()> {
    let exec_structured = lua.create_function(|lua_ctx, cmd: String| {
        let output = Command::new("sh")
            .arg("-c")
            .arg(&cmd)
            .output()
            .map_err(|e| mlua::Error::RuntimeError(e.to_string()))?;

        let stdout = String::from_utf8_lossy(&output.stdout)
            .trim_end()
            .to_string();

        // Try to parse as JSON → aush Value → Lua table; fall back to string.
        match crate::value::Value::from_json(&stdout) {
            Ok(aush_val) => crate::lua::bridge::value_to_lua(lua_ctx, &aush_val),
            Err(_) => Ok(LuaValue::String(lua_ctx.create_string(&stdout)?)),
        }
    })?;
    aush.set("exec_structured", exec_structured)?;
    Ok(())
}

// ---------------------------------------------------------------------------
// json_parse / json_encode
// ---------------------------------------------------------------------------

fn register_json(lua: &Lua, aush: &Table) -> mlua::Result<()> {
    let json_parse = lua.create_function(|lua_ctx, s: String| {
        let aush_val = crate::value::Value::from_json(&s)
            .map_err(|e| mlua::Error::RuntimeError(format!("json_parse: {}", e)))?;
        crate::lua::bridge::value_to_lua(lua_ctx, &aush_val)
    })?;

    let json_encode = lua.create_function(|_, val: LuaValue| {
        let aush_val = lua_to_value(val);
        Ok(aush_val.to_json())
    })?;

    aush.set("json_parse", json_parse)?;
    aush.set("json_encode", json_encode)?;
    Ok(())
}

// ---------------------------------------------------------------------------
// env table
// ---------------------------------------------------------------------------

fn register_env(lua: &Lua, aush: &Table) -> mlua::Result<()> {
    let env = lua.create_table()?;

    let get = lua.create_function(|_, name: String| Ok(std::env::var(&name).ok()))?;

    let set = lua.create_function(|_, (name, value): (String, String)| {
        std::env::set_var(&name, &value);
        Ok(())
    })?;

    env.set("get", get)?;
    env.set("set", set)?;
    aush.set("env", env)?;
    Ok(())
}

// ---------------------------------------------------------------------------
// cwd
// ---------------------------------------------------------------------------

fn register_cwd(lua: &Lua, aush: &Table) -> mlua::Result<()> {
    let cwd = lua.create_function(|_, ()| {
        let path = std::env::current_dir()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default();
        Ok(path)
    })?;
    aush.set("cwd", cwd)?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Event hooks  (aush.on)
// ---------------------------------------------------------------------------

/// Registry key for the hook table: `{ event_name -> [fn, ...] }`.
const HOOKS_KEY: &str = "__aush_hooks__";

fn register_hooks(lua: &Lua, aush: &Table) -> mlua::Result<()> {
    let hooks_store: Table = lua.create_table()?;
    lua.set_named_registry_value(HOOKS_KEY, hooks_store)?;

    let on_fn = lua.create_function(|lua_ctx, (event, func): (String, Function)| {
        let store: Table = lua_ctx.named_registry_value(HOOKS_KEY)?;
        let list: Table = match store.get::<Table>(event.as_str()) {
            Ok(t) => t,
            Err(_) => {
                let t = lua_ctx.create_table()?;
                store.set(event.as_str(), t.clone())?;
                t
            }
        };
        list.raw_set(list.raw_len() + 1, func)?;
        Ok(())
    })?;

    aush.set("on", on_fn)?;
    Ok(())
}

pub(crate) fn hooks_key() -> &'static str {
    HOOKS_KEY
}

// ---------------------------------------------------------------------------
// register_builtin
// ---------------------------------------------------------------------------

/// Registry key for custom builtins: `{ name -> spec_table }`.
const BUILTINS_KEY: &str = "__aush_builtins__";

fn register_builtins_api(lua: &Lua, aush: &Table) -> mlua::Result<()> {
    let store: Table = lua.create_table()?;
    lua.set_named_registry_value(BUILTINS_KEY, store)?;

    let register = lua.create_function(|lua_ctx, (name, spec): (String, Table)| {
        let store: Table = lua_ctx.named_registry_value(BUILTINS_KEY)?;
        store.set(name.as_str(), spec)?;
        Ok(())
    })?;

    aush.set("register_builtin", register)?;
    Ok(())
}

pub(crate) fn builtins_key() -> &'static str {
    BUILTINS_KEY
}

// ---------------------------------------------------------------------------
// register_prompt
// ---------------------------------------------------------------------------

const PROMPT_KEY: &str = "__aush_prompts__";

fn register_prompt_api(lua: &Lua, aush: &Table) -> mlua::Result<()> {
    let store: Table = lua.create_table()?;
    lua.set_named_registry_value(PROMPT_KEY, store)?;

    let register = lua.create_function(|lua_ctx, (name, func): (String, Function)| {
        let store: Table = lua_ctx.named_registry_value(PROMPT_KEY)?;
        store.set(name.as_str(), func)?;
        Ok(())
    })?;

    aush.set("register_prompt", register)?;
    Ok(())
}

pub(crate) fn prompt_key() -> &'static str {
    PROMPT_KEY
}

// ---------------------------------------------------------------------------
// register_completion
// ---------------------------------------------------------------------------

const COMPLETION_KEY: &str = "__aush_completions__";

fn register_completion_api(lua: &Lua, aush: &Table) -> mlua::Result<()> {
    let store: Table = lua.create_table()?;
    lua.set_named_registry_value(COMPLETION_KEY, store)?;

    let register = lua.create_function(|lua_ctx, (name, func): (String, Function)| {
        let store: Table = lua_ctx.named_registry_value(COMPLETION_KEY)?;
        store.set(name.as_str(), func)?;
        Ok(())
    })?;

    aush.set("register_completion", register)?;
    Ok(())
}

pub(crate) fn completion_key() -> &'static str {
    COMPLETION_KEY
}
