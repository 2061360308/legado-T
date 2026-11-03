use std::sync::Arc;

use engine_flow::{BookEngine, PageFetcher};
use engine_model::JsEvaluator;
use engine_rss::RssEngine;
use engine_rule::RuleEngine;

pub struct EnginePipeline {
    rules: Arc<RuleEngine>,
    js: Arc<dyn JsEvaluator>,
    fetcher: Arc<dyn PageFetcher>,
    books: BookEngine,
    rss: RssEngine,
}

impl EnginePipeline {
    pub fn new(
        rules: Arc<RuleEngine>,
        js: Arc<dyn JsEvaluator>,
        fetcher: Arc<dyn PageFetcher>,
    ) -> Self {
        rules.set_js(js.clone());
        let books = BookEngine::new(rules.clone(), js.clone(), fetcher.clone());
        let rss = RssEngine::new(rules.clone(), js.clone(), fetcher.clone());
        Self {
            rules,
            js,
            fetcher,
            books,
            rss,
        }
    }

    pub fn rules(&self) -> &Arc<RuleEngine> {
        &self.rules
    }

    pub fn js(&self) -> &Arc<dyn JsEvaluator> {
        &self.js
    }

    pub fn fetcher(&self) -> &Arc<dyn PageFetcher> {
        &self.fetcher
    }

    pub fn books(&self) -> &BookEngine {
        &self.books
    }

    pub fn rss(&self) -> &RssEngine {
        &self.rss
    }
}
