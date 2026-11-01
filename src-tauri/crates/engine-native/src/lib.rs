pub mod corpus;
pub mod error;
pub mod evaluate;
pub mod model;
pub mod pipeline;
pub mod rule;
pub mod upstream;

pub use corpus::{analyze, render_markdown, Corpus, FeatureStats};
pub use error::{EngineError, Result};
pub use evaluate::{eval_html, eval_html_with_base, eval_list, eval_text};
pub use engine_flow::{classify_pay_action, PayActionResult};
pub use engine_model::BookReview;
pub use model::{
    Book, BookChapter, BookInfoRule, BookListRule, BookSource, ContentRule, ExploreRule,
    ReviewRule, SearchBook, SearchRule, TocRule,
};
pub use pipeline::{parse_sources, Pipeline};
pub use rule::{Mode, SourceRule};
