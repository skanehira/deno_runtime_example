use deno_core::anyhow::Context;
use deno_core::error::AnyError;
use deno_core::v8;
use deno_runtime::permissions::PermissionsContainer;
use deno_runtime::{deno_core, BootstrapOptions};
use futures::executor::block_on;
use std::{rc::Rc, sync::Arc};
mod module_loader;

#[derive(Debug, Default, serde::Deserialize)]
pub struct Object {
    pub name: Option<String>,
}

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
            locale: "".into(),
        },
        extensions: vec![],
        extensions_with_js: vec![],
        unsafely_ignore_certificate_errors: None,
        root_cert_store: None,
        seed: None,
        module_loader: Rc::new(module_loader::FsModuleLoader),
        npm_resolver: None,
        create_web_worker_cb: Arc::new(|_| unreachable!()),
        web_worker_preload_module_cb: Arc::new(|_| unreachable!()),
        web_worker_pre_execute_module_cb: Arc::new(|_| unreachable!()),
        format_js_error_fn: None,
        source_map_getter: None,
        maybe_inspector_server: None,
        should_break_on_first_statement: false,
        get_error_class_fn: None,
        cache_storage_dir: None,
        origin_storage_dir: None,
        blob_store: deno_runtime::deno_web::BlobStore::default(),
        broadcast_channel: deno_runtime::deno_broadcast_channel::InMemoryBroadcastChannel::default(
        ),
        shared_array_buffer_store: None,
        compiled_wasm_module_store: None,
        stdio: Default::default(),
        startup_snapshot: None,
        should_wait_for_inspector_session: false,
    };

    let mut runtime = deno_runtime::worker::MainWorker::bootstrap_from_options(
        main_module.clone(),
        PermissionsContainer::allow_all(),
        options,
    )
    .js_runtime;

    let module_id = runtime
        .load_main_module(
            &main_module,
            Some(
                r#"
const obj = {name: "gorilla"};
export { obj };
"#
                .into(),
            ),
        )
        .await?;

    let _ = runtime.mod_evaluate(module_id);
    runtime.run_event_loop(false).await?;

    let module_handle_scope = runtime.get_module_namespace(module_id)?;

    let global_handle_scope = &mut runtime.handle_scope();
    let local_handle_scope = v8::Local::<v8::Object>::new(global_handle_scope, module_handle_scope);

    let export_name = v8::String::new(global_handle_scope, "obj").context("failed to get obj")?;
    let binding = local_handle_scope.get(global_handle_scope, export_name.into());
    let object = binding.context("not found obj")?;
    let obj: Object = serde_v8::from_v8(global_handle_scope, object)?;
    println!("{}", obj.name.unwrap());
    Ok(())
}

fn main() {
    _ = block_on(run_js("./main.js"));
}
