mod bean;
mod config;
mod kanban;
mod render;
mod tasks;

use anyhow::Result;
use mdbook_preprocessor::book::{Book, BookItem};
use mdbook_preprocessor::{Preprocessor, PreprocessorContext};

/// Walk book items and replace markers with rendered content.
/// Uses direct iteration instead of `for_each_mut` to ensure sub_items
/// are properly nested in the book structure.
fn replace_markers(items: &mut Vec<BookItem>, beans: &[bean::Bean]) {
    for item in items.iter_mut() {
        if let BookItem::Chapter(chapter) = item {
            if chapter.content.contains("{{#beans-kanban}}") {
                chapter.content = kanban::render(beans);
            } else if chapter.content.contains("{{#beans-tasks}}") {
                let parent_number = chapter
                    .number
                    .as_ref()
                    .map(|n| n.as_slice().to_vec())
                    .unwrap_or_default();
                let (content, sub_items) = tasks::render(beans, &parent_number);
                chapter.content = content;
                chapter.sub_items = sub_items;
            } else {
                replace_markers(&mut chapter.sub_items, beans);
            }
        }
    }
}

pub struct BeansPreprocessor;

impl Preprocessor for BeansPreprocessor {
    fn name(&self) -> &str {
        "beans"
    }

    fn run(&self, ctx: &PreprocessorContext, mut book: Book) -> Result<Book> {
        let root = &ctx.root;
        let project_root = config::BeansConfig::project_root(root)?;
        let config = config::BeansConfig::load(root)?;
        let beans = bean::load_beans(&project_root, &config)?;

        replace_markers(&mut book.items, &beans);

        Ok(book)
    }

    fn supports_renderer(&self, renderer: &str) -> Result<bool> {
        Ok(renderer != "not-supported")
    }
}
