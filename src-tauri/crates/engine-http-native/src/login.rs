use engine_model::{Error, HttpResponse, JsRequest, Result};

use crate::url::UrlContext;

pub async fn check_login(
    js: &str,
    response: HttpResponse,
    ctx: &UrlContext<'_>,
) -> Result<HttpResponse> {
    if js.trim().is_empty() {
        return Ok(response);
    }
    let Some(evaluator) = ctx.js else {
        return Err(Error::Js("loginCheckJs 需要 JsEvaluator".into()));
    };
    evaluator
        .eval_response(JsRequest {
            script: js,
            source: ctx.source,
            base_url: ctx.base_url,
            page: ctx.page,
            key: ctx.key,
            result: Some(&response.text),
            result_bytes: None,
            src: None,
            response: Some(&response),
            book: None,
            chapter: None,
            variables: None,
        })
        .await
}
