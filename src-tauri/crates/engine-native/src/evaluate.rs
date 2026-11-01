use std::sync::Arc;
use std::time::Duration;

use engine_js::JsEngine;
use engine_rule::{Content, EvalContext, RuleEngine};

use crate::{EngineError, Result};

fn engine_with_js() -> (RuleEngine, Option<Arc<JsEngine>>) {
    let engine = RuleEngine::new();
    let js = JsEngine::builder()
        .timeout(Duration::from_secs(70))
        .build()
        .ok()
        .map(Arc::new);
    if let Some(js) = &js {
        engine.set_js(js.clone());
    }
    (engine, js)
}

async fn eval_content(rule: &str, content: Content, base_url: Option<&str>) -> Result<String> {
    let (engine, js) = engine_with_js();
    let mut context = EvalContext::new(content);
    if let Some(js) = &js {
        context = context.with_js(js.as_ref());
    }
    if let Some(base_url) = base_url {
        context = context.with_base_url(base_url);
    }
    engine
        .eval_string(rule, &mut context)
        .await
        .map_err(|error| EngineError::Rule(error.to_string()))
}

pub async fn eval_html(rule: &str, html: &str) -> Result<String> {
    eval_content(rule, Content::html(html), None).await
}

pub async fn eval_html_with_base(rule: &str, html: &str, base_url: &str) -> Result<String> {
    eval_content(rule, Content::html(html), Some(base_url)).await
}

pub async fn eval_text(rule: &str, text: &str) -> Result<String> {
    eval_content(rule, Content::text(text), None).await
}

pub async fn eval_list(rule: &str, html: &str) -> Result<Vec<String>> {
    let (engine, js) = engine_with_js();
    let mut context = EvalContext::new(Content::html(html));
    if let Some(js) = &js {
        context = context.with_js(js.as_ref());
    }
    engine
        .eval_list(rule, &mut context)
        .await
        .map_err(|error| EngineError::Rule(error.to_string()))
}
