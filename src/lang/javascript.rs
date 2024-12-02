#![allow(unused_imports)]

use anyhow::{anyhow, Error, Result};
use async_trait::async_trait;
use log::{debug, error, info};
use std::{fs, path::Path, process::Command};

#[cfg(feature = "javascript")]
use v8::Module;

use super::{Dev, RunStatus};

#[cfg(feature = "javascript")]
static LOG_TARGET: &str = "javascript";

#[derive(Debug, Clone)]
pub struct JavaScriptLanguage {}

impl JavaScriptLanguage {
    pub fn new() -> Self {
        Self {}
    }

    #[cfg(feature = "javascript")]
    fn init(&self) -> Result<(), anyhow::Error> {
        let platform = v8::new_default_platform(0, false).make_shared();
        v8::V8::initialize_platform(platform);
        v8::V8::initialize();
        Ok(())
    }
}

impl Default for JavaScriptLanguage {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl super::LanguageFunctions for JavaScriptLanguage {
    #[allow(unused_variables)]
    async fn run_file(
        &self,
        dev: Dev,
        file: &str,
        args: Vec<&str>,
    ) -> Result<RunStatus, anyhow::Error> {
        #[cfg(not(feature = "javascript"))]
        return Err(anyhow!("JavaScript support is not enabled"));

        #[cfg(feature = "javascript")]
        return self.run_file(dev, file, args).await;
    }

    #[allow(unused_variables)]
    async fn load_file(&self, file: &str) -> Result<(), anyhow::Error> {
        #[cfg(not(feature = "javascript"))]
        return Err(anyhow!("JavaScript support is not enabled"));

        #[cfg(feature = "javascript")]
        return self.load_file(file).await;
    }

    #[allow(unused_variables)]
    async fn run_shell(&self, command: &str, args: Vec<&str>) -> Result<RunStatus, anyhow::Error> {
        #[cfg(not(feature = "javascript"))]
        return Err(anyhow!("JavaScript support is not enabled"));

        #[cfg(feature = "javascript")]
        return self.run_shell(command, args).await;
    }
}

#[cfg(feature = "javascript")]
impl JavaScriptLanguage {
    async fn run_file(
        &self,
        dev: Dev,
        file: &str,
        _args: Vec<&str>,
    ) -> Result<RunStatus, anyhow::Error> {
        self.init()?;

        let isolate = &mut v8::Isolate::new(Default::default());
        let handle_scope = &mut v8::HandleScope::new(isolate);
        let context = v8::Context::new(handle_scope, Default::default());
        let scope = &mut v8::ContextScope::new(handle_scope, context);
        let global = context.global(scope);

        {
            let key = v8::String::new(scope, "Dev").unwrap();
            let value = v8::External::new(scope, &dev as *const _ as *mut std::ffi::c_void);
            global.set(scope, key.into(), value.into());

            let maybe_module = load_file(file, scope)?;
            let tc_scope = &mut v8::TryCatch::new(scope);

            ensure_module_instantiated(tc_scope, maybe_module)
                .ok_or(anyhow!("Failed to ensure module is instantiated"))?;

            maybe_module
                .evaluate(tc_scope)
                .ok_or(anyhow!("Failed to evaluate module"))?;

            if tc_scope.has_caught() {
                let exception = tc_scope.exception().unwrap();
                return Err(anyhow::anyhow!(exception.to_rust_string_lossy(tc_scope)));
            }

            let module_namespace = maybe_module
                .get_module_namespace()
                .to_object(tc_scope)
                .ok_or(anyhow!("Failed to convert module namespace to object"))?;

            let default_key = v8::String::new(tc_scope, "default")
                .ok_or(anyhow!("Failed to create default key string"))?;
            let default_export = module_namespace
                .get(tc_scope, default_key.into())
                .ok_or(anyhow!("Failed to get default export"))?;

            match serde_v8::from_v8::<Dev>(tc_scope, default_export) {
                Ok(dev) => debug!(target: LOG_TARGET, "{:?}", dev),
                Err(e) => error!(target: LOG_TARGET, "Error deserializing: {:?}", e),
            }
        }

        // unsafe {
        //     v8::V8::dispose();
        // }
        // v8::V8::dispose_platform();

        Ok(RunStatus {
            exit_code: Some(0),
            message: None,
        })
    }

    async fn load_file(&self, file: &str) -> Result<(), anyhow::Error> {
        self.init()?;

        let isolate = &mut v8::Isolate::new(Default::default());
        let handle_scope = &mut v8::HandleScope::new(isolate);
        let context = v8::Context::new(handle_scope, Default::default());
        let scope = &mut v8::ContextScope::new(handle_scope, context);
        load_file(file, scope)?;
        Ok(())
    }

    async fn run_shell(
        &self,
        _command: &str,
        _args: Vec<&str>,
    ) -> Result<RunStatus, anyhow::Error> {
        self.init()?;

        let isolate = &mut v8::Isolate::new(v8::CreateParams::default());
        let handle_scope = &mut v8::HandleScope::new(isolate);

        let context = v8::Context::new(handle_scope, Default::default());

        let context_scope = &mut v8::ContextScope::new(handle_scope, context);
        let scope = &mut v8::HandleScope::new(context_scope);

        run_shell(scope)?;

        Ok(RunStatus {
            exit_code: Some(0),
            message: None,
        })
    }
}

#[cfg(feature = "javascript")]
fn load_file<'a>(
    file: &str,
    scope: &mut v8::HandleScope<'a>,
) -> Result<v8::Local<'a, v8::Module>, Error> {
    let file_contents = fs::read_to_string(Path::new(file))?;
    let code = v8::String::new(scope, &file_contents).ok_or(anyhow!("Failed to create code"))?;
    let file_name = v8::String::new(scope, file).ok_or(anyhow!("Failed to create file name"))?;
    let origin = v8::ScriptOrigin::new(
        scope,
        file_name.into(),
        0,
        0,
        false,
        0,
        None,
        false,
        false,
        true,
        None,
    );
    let mut source = v8::script_compiler::Source::new(code, Some(&origin));
    let maybe_module = v8::script_compiler::compile_module(scope, &mut source);

    maybe_module.ok_or(anyhow!("Failed to compile module"))
}

#[inline]
#[cfg(feature = "javascript")]
fn get_version(
    scope: &mut v8::HandleScope,
    _args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let global = scope.get_current_context().global(scope);
    let key = v8::String::new(scope, "Dev").unwrap();
    let value = global.get(scope, key.into()).unwrap();
    let ext = v8::Local::<v8::External>::try_from(value).unwrap();
    let dev = unsafe { &*(ext.value() as *const Dev) };
    let result = v8::String::new(scope, &dev.version).unwrap();
    retval.set(result.into());
}

#[inline]
#[cfg(feature = "javascript")]
fn get_work_dir(
    scope: &mut v8::HandleScope,
    _args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let working_dir =
        String::from_utf8_lossy(&Command::new("pwd").output().unwrap().stdout).to_string();
    let result = v8::String::new(scope, working_dir.as_str()).unwrap();
    retval.set(result.into());
}

#[cfg(feature = "javascript")]
fn ensure_module_instantiated<'a>(
    scope: &'a mut v8::HandleScope,
    module: v8::Local<'a, v8::Module>,
) -> Option<v8::Local<'a, v8::Module>> {
    match module.get_status() {
        v8::ModuleStatus::Instantiated => None,
        v8::ModuleStatus::Evaluated => None,
        v8::ModuleStatus::Errored => None,
        v8::ModuleStatus::Uninstantiated => {
            module.instantiate_module(scope, module_callback)?;
            Some(module)
        }
        _ => None,
    }
}

#[inline]
#[cfg(feature = "javascript")]
fn module_callback<'a>(
    context: v8::Local<'a, v8::Context>,
    specifier: v8::Local<'a, v8::String>,
    _import_assertions: v8::Local<'a, v8::FixedArray>,
    _referrer: v8::Local<'a, v8::Module>,
) -> Option<v8::Local<'a, v8::Module>> {
    let scope = &mut unsafe { v8::CallbackScope::new(context) };
    let specifier_str = specifier.to_rust_string_lossy(scope);

    if specifier_str == "dev" {
        let module_name = v8::String::new(scope, "dev").unwrap();
        let export_names = [
            v8::String::new(scope, "getVersion").unwrap(),
            v8::String::new(scope, "getWorkDir").unwrap(),
        ];

        let dev_module =
            Module::create_synthetic_module(scope, module_name, &export_names, evaluate_module);
        ensure_module_instantiated(scope, dev_module).unwrap();
        let _ = dev_module.evaluate(scope);
        Some(dev_module)
    } else {
        None
    }
}

#[inline]
#[cfg(feature = "javascript")]
fn evaluate_module<'a>(
    context: v8::Local<'a, v8::Context>,
    module: v8::Local<v8::Module>,
) -> Option<v8::Local<'a, v8::Value>> {
    let scope = &mut unsafe { v8::CallbackScope::new(context) };

    let get_version = v8::Function::new(scope, get_version).unwrap();
    let get_version_key = v8::String::new(scope, "getVersion").unwrap();
    let _ = module.set_synthetic_module_export(scope, get_version_key, get_version.into());

    let get_work_dir = v8::Function::new(scope, get_work_dir).unwrap();
    let get_work_dir_key = v8::String::new(scope, "getWorkDir").unwrap();
    let _ = module.set_synthetic_module_export(scope, get_work_dir_key, get_work_dir.into());

    // Seems like it doesn't matter what we return
    // here it just has to be something
    let obj = v8::Object::new(scope);
    Some(obj.into())
}

/// Process remaining command line arguments and execute files
#[cfg(feature = "javascript")]
fn run_shell(scope: &mut v8::HandleScope) -> Result<(), anyhow::Error> {
    use std::io::{self, Write};

    println!("V8 version {} [sample shell]", v8::V8::get_version());

    loop {
        print!("> ");
        io::stdout().flush().unwrap();

        let mut buf = String::new();
        match io::stdin().read_line(&mut buf) {
            Ok(n) => {
                if n == 0 {
                    println!();
                    return Ok(());
                }

                execute_string(scope, &buf, "(shell)", true, true);
            }
            Err(error) => println!("error: {}", error),
        }
    }
}

#[cfg(feature = "javascript")]
fn execute_string(
    scope: &mut v8::HandleScope,
    script: &str,
    filename: &str,
    print_result: bool,
    report_exceptions_flag: bool,
) {
    let mut scope = v8::TryCatch::new(scope);

    let filename = v8::String::new(&mut scope, filename).unwrap();
    let script = v8::String::new(&mut scope, script).unwrap();
    let origin = v8::ScriptOrigin::new(
        &mut scope,
        filename.into(),
        0,
        0,
        false,
        0,
        None,
        false,
        false,
        false,
        None,
    );

    let script = if let Some(script) = v8::Script::compile(&mut scope, script, Some(&origin)) {
        script
    } else {
        assert!(scope.has_caught());

        if report_exceptions_flag {
            report_exceptions(scope);
        }
        return;
    };

    if let Some(result) = script.run(&mut scope) {
        if print_result {
            println!(
                "{}",
                result
                    .to_string(&mut scope)
                    .unwrap()
                    .to_rust_string_lossy(&mut scope)
            );
        }
    } else {
        assert!(scope.has_caught());
        if report_exceptions_flag {
            report_exceptions(scope);
        }
    }
}

#[cfg(feature = "javascript")]
fn report_exceptions(mut try_catch: v8::TryCatch<v8::HandleScope>) {
    let exception = try_catch.exception().unwrap();
    let exception_string = exception
        .to_string(&mut try_catch)
        .unwrap()
        .to_rust_string_lossy(&mut try_catch);
    let message = if let Some(message) = try_catch.message() {
        message
    } else {
        eprintln!("{}", exception_string);
        return;
    };

    // Print (filename):(line number): (message).
    let filename = message
        .get_script_resource_name(&mut try_catch)
        .map_or_else(
            || "(unknown)".into(),
            |s| {
                s.to_string(&mut try_catch)
                    .unwrap()
                    .to_rust_string_lossy(&mut try_catch)
            },
        );
    let line_number = message.get_line_number(&mut try_catch).unwrap_or_default();

    eprintln!("{}:{}: {}", filename, line_number, exception_string);

    // Print line of source code.
    let source_line = message
        .get_source_line(&mut try_catch)
        .map(|s| {
            s.to_string(&mut try_catch)
                .unwrap()
                .to_rust_string_lossy(&mut try_catch)
        })
        .unwrap();
    eprintln!("{}", source_line);

    // Print wavy underline (GetUnderline is deprecated).
    let start_column = message.get_start_column();
    let end_column = message.get_end_column();

    for _ in 0..start_column {
        eprint!(" ");
    }

    for _ in start_column..end_column {
        eprint!("^");
    }

    eprintln!();

    // Print stack trace
    let stack_trace = if let Some(stack_trace) = try_catch.stack_trace() {
        stack_trace
    } else {
        return;
    };
    let stack_trace = unsafe { v8::Local::<v8::String>::cast_unchecked(stack_trace) };
    let stack_trace = stack_trace
        .to_string(&mut try_catch)
        .map(|s| s.to_rust_string_lossy(&mut try_catch));

    if let Some(stack_trace) = stack_trace {
        eprintln!("{}", stack_trace);
    }
}
