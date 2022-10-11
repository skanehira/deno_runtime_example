use deno_ast::parse_module;
use deno_ast::EmitOptions;
use deno_ast::ParseParams;
use deno_ast::SourceTextInfo;
use deno_core::anyhow::Error;
use deno_core::ModuleLoader;
use deno_core::ModuleSource;
use deno_core::ModuleSourceFuture;
use deno_core::ModuleSpecifier;
use deno_core::ModuleType;
use deno_core::OpState;
use deno_fetch::create_http_client;
use futures::future::FutureExt;
use std::cell::RefCell;
use std::future::Future;
use std::pin::Pin;

use std::rc::Rc;

pub struct FsModuleLoader;

impl ModuleLoader for FsModuleLoader {
    fn load(
        &self,
        module_specifier: &ModuleSpecifier,
        _maybe_referrer: Option<ModuleSpecifier>,
        _is_dyn_import: bool,
    ) -> Pin<Box<ModuleSourceFuture>> {
        let module_specifier = module_specifier.clone();
        async move {
            let code;
            if module_specifier.scheme().starts_with("http") {
                println!("Downloading: {}", module_specifier);
                let client = create_http_client("deno".into(), None, vec![], None, None, None)?;
                let resp = client.get(module_specifier.to_string()).send().await?;
                code = resp.bytes().await?.to_vec();
            } else {
                let path = module_specifier.to_file_path().map_err(|_| {
                    deno_core::error::generic_error(format!(
                        "Provided module specifier \"{}\" is not a file URL.",
                        module_specifier
                    ))
                })?;
                code = std::fs::read(path)?;
            }

            let module_type = if module_specifier.to_string().ends_with(".json") {
                ModuleType::Json
            } else {
                ModuleType::JavaScript
            };

            let source = String::from_utf8(code).unwrap();
            let parsed_source = parse_module(ParseParams {
                specifier: module_specifier.clone().into(),
                media_type: deno_ast::MediaType::TypeScript,
                text_info: SourceTextInfo::new(source.into()),
                capture_tokens: false,
                maybe_syntax: None,
                scope_analysis: false,
            })
            .unwrap();

            let options = EmitOptions::default();
            let source = parsed_source.transpile(&options).unwrap();
            let code: Vec<u8> = source.text.as_bytes().to_vec();

            let module = ModuleSource {
                code: code.into_boxed_slice(),
                module_type,
                module_url_specified: module_specifier.to_string(),
                module_url_found: module_specifier.to_string(),
            };
            Ok(module)
        }
        .boxed_local()
    }

    fn resolve(
        &self,
        specifier: &str,
        referrer: &str,
        _is_main: bool,
    ) -> Result<ModuleSpecifier, Error> {
        Ok(deno_core::resolve_import(specifier, referrer)?)
    }

    fn prepare_load(
        &self,
        _op_state: Rc<RefCell<OpState>>,
        _module_specifier: &ModuleSpecifier,
        _maybe_referrer: Option<String>,
        _is_dyn_import: bool,
    ) -> std::pin::Pin<Box<dyn Future<Output = Result<(), Error>>>> {
        async { Ok(()) }.boxed_local()
    }
}
