#![allow(unused_imports)]

use std::fs;

use anyhow::{anyhow, Result};
use async_trait::async_trait;

#[cfg(feature = "lua")]
use mlua::prelude::*;

use super::{Dev, RunStatus};

#[derive(Debug, Clone)]
pub struct LuaLanguage {}

impl LuaLanguage {
    pub fn new() -> Self {
        Self {}
    }

    fn init(&self, dev: &Dev) -> Result<Lua, anyhow::Error> {
        // I guess we set with rust :(
        for (key, value) in dev.get_env() {
            std::env::set_var(key, value);
        }
        let lua = Lua::new();
        Ok(lua)
    }
}

impl Default for LuaLanguage {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl super::LanguageFunctions for LuaLanguage {
    #[allow(unused_variables)]
    async fn run_file(
        &self,
        dev: Dev,
        file: &str,
        args: Vec<&str>,
    ) -> Result<RunStatus, anyhow::Error> {
        #[cfg(not(feature = "lua"))]
        return Err(anyhow!("lua support is not enabled"))?;

        #[cfg(feature = "lua")]
        return self.run_file(dev, file, args).await;
    }

    #[allow(unused_variables)]
    async fn load_file(&self, file: &str) -> Result<(), anyhow::Error> {
        #[cfg(not(feature = "lua"))]
        return Err(anyhow!("lua support is not enabled"))?;

        #[cfg(feature = "lua")]
        return self.load_file(file).await;
    }

    #[allow(unused_variables)]
    async fn run_shell(&self, command: &str, args: Vec<&str>) -> Result<RunStatus, anyhow::Error> {
        #[cfg(not(feature = "lua"))]
        return Err(anyhow!("lua support is not enabled"))?;

        #[cfg(feature = "lua")]
        return self.run_shell(command, args).await;
    }
}

#[cfg(feature = "lua")]
impl LuaLanguage {
    async fn run_file(
        &self,
        dev: Dev,
        file: &str,
        _args: Vec<&str>,
    ) -> Result<RunStatus, anyhow::Error> {
        let lua = self.init(&dev)?;
        let globals = lua.globals();

        // let load = lua.create_function(move |lua, modname: String| {
        //     let rectangle = Rectangle {
        //         name: "Rectangle".to_string(),
        //         length: 0,
        //         width: 0,
        //     };
        //     let m = lua.create_table()?;
        //     m.set("__name", modname)?;
        //     m.set("rec", rectangle)?;
        //     m.set("v", "1.0")?;
        //     Ok(m)
        // })?;
        // let t: mlua::Table = lua.load_from_function("test", load.clone())?;

        // globals.set("test", t)?;
        globals.set("dev", lua.create_ser_userdata(dev)?)?;

        let lua_code = fs::read_to_string(file)?;
        let m: mlua::Table = lua.load(&lua_code).eval()?;

        let dev: Dev = lua.from_value(m.get("Out")?)?;
        println!("{}", dev);

        let init: String = m.get::<mlua::Function>("init")?.call(())?;
        println!("{}", init);

        Ok(RunStatus {
            exit_code: Some(0),
            message: None,
        })
    }

    async fn load_file(&self, _file: &str) -> Result<(), anyhow::Error> {
        todo!()
    }

    async fn run_shell(
        &self,
        _command: &str,
        _args: Vec<&str>,
    ) -> Result<RunStatus, anyhow::Error> {
        todo!();
    }
}

#[cfg(feature = "lua")]
impl LuaUserData for Dev {
    fn add_methods<'lua, M: LuaUserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("get_version", |_, this, ()| Ok(this.get_version()));
        methods.add_method("get_dir", |_, this, ()| Ok(this.get_dir()));

        methods.add_meta_method(LuaMetaMethod::Index, |lua, this, key: String| {
            match key.as_str() {
                "version" => Ok(lua.create_string(&this.version)?),
                _ => Err(mlua::Error::RuntimeError("Attribute not found".to_string())),
            }
        });

        methods.add_meta_method_mut(
            LuaMetaMethod::NewIndex,
            |_, this, (key, value): (String, String)| match key.as_str() {
                "version" => {
                    this.version = value;
                    Ok(())
                }
                _ => Err(mlua::Error::RuntimeError(
                    "Cannot set this attribute".to_string(),
                )),
            },
        );
    }
}

// #[derive(Default)]
// struct Rectangle {
//     name: String,
//     length: u32,
//     width: u32,
// }

// #[cfg(feature = "lua")]
// impl mlua::UserData for Rectangle {
//     fn add_fields<'lua, F: mlua::UserDataFields<Self>>(fields: &mut F) {
//         fields.add_field_method_get("name", |_, this| Ok(this.name.clone()));
//         fields.add_field_method_get("length", |_, this| Ok(this.length));
//         fields.add_field_method_set("length", |_, this, val| {
//             this.length = val;
//             Ok(())
//         });
//         fields.add_field_method_get("width", |_, this| Ok(this.width));
//         fields.add_field_method_set("width", |_, this, val| {
//             this.width = val;
//             Ok(())
//         });
//     }

//     fn add_methods<'lua, M: mlua::UserDataMethods<Self>>(methods: &mut M) {
//         methods.add_method("area", |_, this, ()| Ok(this.length * this.width));
//         methods.add_method("diagonal", |_, this, ()| {
//             Ok((this.length.pow(2) as f64 + this.width.pow(2) as f64).sqrt())
//         });

//         // Constructor
//         methods.add_meta_function(mlua::MetaMethod::Call, |_, ()| Ok(Rectangle::default()));
//     }
// }
