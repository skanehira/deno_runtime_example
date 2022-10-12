use deno_core::anyhow::Context;
use deno_core::error::AnyError;
use deno_core::v8;
use deno_runtime::permissions::Permissions;
use deno_runtime::{deno_core, BootstrapOptions};
use std::{rc::Rc, sync::Arc};
mod module_loader;

async fn run_js(file_path: &str) -> Result<(), AnyError> {
    let main_module = deno_core::resolve_path(file_path).unwrap();
    let options = deno_runtime::worker::WorkerOptions {
        bootstrap: BootstrapOptions {
            args: vec![],
            cpu_count: 1,
            debug_flag: false,
            enable_testing_features: false,
            location: None,
            no_color: true,
            is_tty: false,
            runtime_version: "x".to_string(),
            ts_version: "x".to_string(),
            unstable: true,
            user_agent: "x".to_string(),
            inspect: false,
        },
        extensions: vec![],
        unsafely_ignore_certificate_errors: None,
        root_cert_store: None,
        seed: None,
        format_js_error_fn: None,
        source_map_getter: None,
        web_worker_preload_module_cb: Arc::new(|_| unreachable!()),
        web_worker_pre_execute_module_cb: Arc::new(|_| unreachable!()),
        create_web_worker_cb: Arc::new(|_| unreachable!()),
        maybe_inspector_server: None,
        should_break_on_first_statement: false,
        module_loader: Rc::new(module_loader::FsModuleLoader),
        npm_resolver: None,
        get_error_class_fn: None,
        cache_storage_dir: None,
        origin_storage_dir: None,
        blob_store: deno_runtime::deno_web::BlobStore::default(),
        broadcast_channel: deno_runtime::deno_broadcast_channel::InMemoryBroadcastChannel::default(
        ),
        shared_array_buffer_store: None,
        compiled_wasm_module_store: None,
        stdio: Default::default(),
    };

    let mut runtime = deno_runtime::worker::MainWorker::bootstrap_from_options(
        main_module.clone(),
        Permissions::allow_all(),
        options,
    )
    .js_runtime;

    let module_id = runtime
        .load_main_module(
            &main_module,
            Some(r#"export const obj = {name: "gorilla"}; console.log("name:", name); "#.into()),
        )
        .await?;

    let global_scope = runtime.get_module_namespace(module_id).unwrap();
    {
        {
            let handle_scope = &mut runtime.handle_scope();
            let local_scope = v8::Local::<v8::Object>::new(handle_scope, &global_scope);

            let key = serde_v8::to_v8(handle_scope, "name")?;
            let value = serde_v8::to_v8(handle_scope, "gorilla")?;
            local_scope.set(handle_scope, key, value);
        }

        let _ = runtime.mod_evaluate(module_id);
        runtime.run_event_loop(false).await?;
    }

    let scope = &mut runtime.handle_scope();
    let local_object = v8::Local::<v8::Object>::new(scope, global_scope);

    let default_export_name = v8::String::new(scope, "obj").unwrap();
    let binding = local_object.get(scope, default_export_name.into());
    let object = binding.context("not found obj")?;

    #[derive(Debug, Default, serde::Deserialize)]
    pub struct Object {
        pub name: Option<String>,
    }

    let obj: Object = serde_v8::from_v8(scope, object).unwrap();
    dbg!(obj);
    Ok(())
}

fn main() {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    if let Err(error) = runtime.block_on(run_js("./main.js")) {
        eprintln!("error: {}", error);
    }
}
