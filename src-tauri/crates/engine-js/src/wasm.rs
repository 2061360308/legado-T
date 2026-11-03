use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;
use std::time::Duration;

use engine_model::{
    BookSource, CacheStore, CookieStore, HttpFetcher, JsRequest, JsVariables, Result, RuleRunner,
    Source,
};
use rquickjs::{CatchResultExt as _, Context, Runtime};

use crate::host::{Binding, Host};
use crate::worker::{self, Deps, Evaluated};

pub struct WasmEngine {
    host: Rc<Host>,
    runtime: Runtime,
    context: Context,
    loaded_libs: RefCell<HashMap<String, String>>,
    source_variables: RefCell<HashMap<String, HashMap<String, String>>>,
}

impl WasmEngine {
    pub fn new(
        http: Option<Arc<dyn HttpFetcher>>,
        android_id: impl Into<String>,
    ) -> Result<Self> {
        Self::with_deps(http, None, None, None, android_id)
    }

    pub fn with_deps(
        http: Option<Arc<dyn HttpFetcher>>,
        cache: Option<Arc<dyn CacheStore>>,
        cookies: Option<Arc<dyn CookieStore>>,
        rules: Option<Arc<dyn RuleRunner>>,
        android_id: impl Into<String>,
    ) -> Result<Self> {
        let runtime = Runtime::new()
            .map_err(|error| engine_model::Error::Js(format!("QuickJS runtime: {error}")))?;
        runtime.set_memory_limit(256 * 1024 * 1024);
        runtime.set_max_stack_size(1024 * 1024);
        let host = Host::new(
            Deps {
                http,
                cache,
                cookies,
                rules,
            },
            android_id.into(),
        )
        .map_err(|error| engine_model::Error::Js(format!("JS host: {error}")))?;
        let context = worker::build_context(&runtime, &host, None)?;
        Ok(Self {
            host,
            runtime,
            context,
            loaded_libs: RefCell::new(HashMap::new()),
            source_variables: RefCell::new(HashMap::new()),
        })
    }

    pub fn set_source_variables(
        &self,
        source_key: &str,
        variables: HashMap<String, String>,
    ) {
        if let Some(store) = self.host.cache.as_ref()
            && let Ok(value) = serde_json::to_vec(&variables)
        {
            let key = engine_model::CacheNamespace::JsCache
                .key(&format!("sourceVariable_{source_key}"));
            let _ = futures::executor::block_on(store.put(&key, value.into(), None));
        }
        self.source_variables
            .borrow_mut()
            .insert(source_key.to_string(), variables);
    }

    pub fn eval(
        &self,
        script: &str,
        base_url: &str,
        result: &str,
        source_json: Option<&str>,
    ) -> Result<String> {
        let source: Option<Box<dyn Source>> = source_json
            .and_then(|json| serde_json::from_str::<BookSource>(json).ok())
            .map(|source| Box::new(source) as Box<dyn Source>);
        let request = JsRequest {
            script,
            source: source.as_deref(),
            base_url,
            page: None,
            key: None,
            result: Some(result),
            result_bytes: None,
            src: None,
            response: None,
            book: None,
            chapter: None,
            variables: None,
        };
        self.eval_request(&request, None)
            .map(|value| value.value)
    }

    pub fn eval_request(
        &self,
        request: &JsRequest<'_>,
        result_items: Option<&[String]>,
    ) -> Result<Evaluated> {
        let source_key = request.source.map(Source::key).unwrap_or_default();
        if let Some(source) = request.source
            && let Some(lib) = source.js_lib().filter(|lib| !lib.trim().is_empty())
        {
            let needs_load = self
                .loaded_libs
                .borrow()
                .get(source_key)
                .is_none_or(|loaded| loaded != lib);
            if needs_load {
                self.context.with(|ctx| {
                    let mut options = rquickjs::context::EvalOptions::default();
                    options.strict = false;
                    match ctx
                        .eval_with_options::<rquickjs::Value, _>(lib, options)
                        .catch(&ctx)
                    {
                        Ok(_) => Ok(()),
                        Err(caught) => Err(engine_model::Error::Js(format!(
                            "jsLib 执行失败: {}",
                            worker::format_caught(caught)
                        ))),
                    }
                })?;
                self.loaded_libs
                    .borrow_mut()
                    .insert(source_key.to_string(), lib.to_string());
            }
        }

        let mut binding = Binding::from_request(request, result_items);
        if request.variables.is_none()
            && let Some(variables) = self.source_variables.borrow().get(source_key)
        {
            binding.variables = JsVariables::new(None, None, Some(variables.clone()));
        }
        let saved = self.host.take_binding();
        self.host.set_binding(Some(binding));
        let evaluated = self
            .context
            .with(|ctx| worker::eval_in_ctx(&ctx, &self.host, request.script));
        self.host.set_binding(saved);
        evaluated
    }

    pub fn timeout(&self) -> Duration {
        Duration::from_secs(70)
    }

    pub fn keep_alive(&self) -> &Runtime {
        &self.runtime
    }
}
