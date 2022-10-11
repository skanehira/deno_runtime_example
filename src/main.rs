use deno_core::error::AnyError;
use deno_runtime::permissions::Permissions;
use deno_runtime::{deno_core, BootstrapOptions};
use std::{rc::Rc, sync::Arc};
mod module;

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
        module_loader: Rc::new(module::FsModuleLoader),
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

    let mut worker = deno_runtime::worker::MainWorker::bootstrap_from_options(
        main_module.clone(),
        Permissions::allow_all(),
        options,
    );

    let result = worker.execute_main_module(&main_module).await;
    dbg!(&result);
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
