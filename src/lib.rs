mod bean;
mod config;
mod kanban;
mod render;
mod tasks;

use anyhow::Result;
use mdbook_preprocessor::book::{Book, BookItem};
use mdbook_preprocessor::{Preprocessor, PreprocessorContext};

pub struct BeansPreprocessor;

impl Preprocessor for BeansPreprocessor {
    fn name(&self) -> &str {
        "beans"
    }

    fn run(&self, ctx: &PreprocessorContext, mut book: Book) -> Result<Book> {
        let root = &ctx.root;
        let config = config::BeansConfig::load(root)?;
        let beans = bean::load_beans(root, &config)?;

        book.for_each_mut(|item| {
            if let BookItem::Chapter(chapter) = item {
                if chapter.content.contains("{{#beans-kanban}}") {
                    chapter.content = kanban::render(&beans);
                } else if chapter.content.contains("{{#beans-tasks}}") {
                    let (content, sub_items) = tasks::render(&beans);
                    chapter.content = content;
                    chapter.sub_items = sub_items;
                }
            }
        });

        Ok(book)
    }

    fn supports_renderer(&self, renderer: &str) -> Result<bool> {
        Ok(renderer != "not-supported")
    }
}
