use mdbook_preprocessor::book::{BookItem, Chapter};

use crate::bean::{Bean, BeanStatus, BeanType};
use crate::render;

/// Type sections in display order.
const TYPE_SECTIONS: &[(BeanType, &str)] = &[
    (BeanType::Epic, "Epics"),
    (BeanType::Feature, "Features"),
    (BeanType::Task, "Tasks"),
    (BeanType::Bug, "Bugs"),
    (BeanType::Spike, "Spikes"),
    (BeanType::Chore, "Chores"),
];

fn make_bean_chapter(bean: &Bean, all_beans: &[Bean]) -> BookItem {
    let page_content = render::render_bean_page(bean, all_beans);
    let path = format!("beans/{}.md", bean.id);
    let mut chapter = Chapter::new(&bean.frontmatter.title, page_content, &path, vec![]);
    chapter.source_path = None;
    BookItem::Chapter(chapter)
}

/// Render the All Tasks chapter content and sub_items.
/// Returns (index page content, sub_item chapters for each bean).
pub fn render(beans: &[Bean]) -> (String, Vec<BookItem>) {
    let mut content = String::from("# All Tasks\n\n");
    let mut sub_items: Vec<BookItem> = Vec::new();

    // Collect epic-to-children mapping
    let epic_children = |epic_id: &str| -> Vec<&Bean> {
        beans
            .iter()
            .filter(|b| b.frontmatter.parent.as_deref() == Some(epic_id))
            .collect()
    };

    for (bean_type, section_label) in TYPE_SECTIONS {
        let matching: Vec<&Bean> = beans
            .iter()
            .filter(|b| &b.frontmatter.bean_type == bean_type)
            .collect();

        if matching.is_empty() {
            continue;
        }

        content.push_str(&format!("## {section_label}\n\n"));

        for bean in &matching {
            content.push_str(&format!(
                "- [{}]({}.md)",
                bean.frontmatter.title, bean.id
            ));

            // Show status inline
            content.push_str(&format!(
                " — *{}*",
                render::status_label(&bean.frontmatter.status)
            ));

            content.push('\n');

            // For epics, also list subtasks inline
            if *bean_type == BeanType::Epic {
                let children = epic_children(&bean.id);
                for child in &children {
                    content.push_str(&format!(
                        "  - [{}]({}.md) — *{}*\n",
                        child.frontmatter.title,
                        child.id,
                        render::status_label(&child.frontmatter.status)
                    ));
                }
            }

            sub_items.push(make_bean_chapter(bean, beans));
        }

        content.push('\n');
    }

    // Add draft beans at the end
    let drafts: Vec<&Bean> = beans
        .iter()
        .filter(|b| b.frontmatter.status == BeanStatus::Draft)
        .collect();

    if !drafts.is_empty() {
        content.push_str("## Drafts\n\n");
        for bean in &drafts {
            content.push_str(&format!(
                "- [{}]({}.md)\n",
                bean.frontmatter.title, bean.id
            ));
            sub_items.push(make_bean_chapter(bean, beans));
        }
        content.push('\n');
    }

    (content, sub_items)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bean::BeanFrontmatter;

    fn make_bean(
        id: &str,
        title: &str,
        status: BeanStatus,
        bean_type: BeanType,
        parent: Option<&str>,
    ) -> Bean {
        Bean {
            id: id.to_string(),
            frontmatter: BeanFrontmatter {
                title: title.to_string(),
                status,
                bean_type,
                priority: "normal".to_string(),
                tags: vec![],
                parent: parent.map(|s| s.to_string()),
                blocked_by: vec![],
            },
            body: format!("Body of {title}"),
        }
    }

    #[test]
    fn tasks_groups_by_type() {
        let beans = vec![
            make_bean("b-1", "A feature", BeanStatus::Todo, BeanType::Feature, None),
            make_bean("b-2", "A bug", BeanStatus::Todo, BeanType::Bug, None),
            make_bean("b-3", "A task", BeanStatus::Todo, BeanType::Task, None),
        ];
        let (content, _) = render(&beans);
        assert!(content.contains("## Features"));
        assert!(content.contains("## Tasks"));
        assert!(content.contains("## Bugs"));
    }

    #[test]
    fn tasks_creates_sub_items_with_stable_paths() {
        let beans = vec![
            make_bean("b-1", "Task one", BeanStatus::Todo, BeanType::Task, None),
            make_bean("b-2", "Task two", BeanStatus::Done, BeanType::Task, None),
        ];
        let (_, sub_items) = render(&beans);
        assert_eq!(sub_items.len(), 2);

        for item in &sub_items {
            if let BookItem::Chapter(ch) = item {
                let path = ch.path.as_ref().unwrap().to_string_lossy();
                assert!(path.starts_with("beans/"));
                assert!(path.ends_with(".md"));
            }
        }
    }

    #[test]
    fn tasks_epic_lists_subtasks_inline() {
        let beans = vec![
            make_bean("b-epic", "My Epic", BeanStatus::InProgress, BeanType::Epic, None),
            make_bean("b-sub1", "Sub 1", BeanStatus::Done, BeanType::Task, Some("b-epic")),
            make_bean("b-sub2", "Sub 2", BeanStatus::Todo, BeanType::Feature, Some("b-epic")),
        ];
        let (content, _) = render(&beans);
        assert!(content.contains("Sub 1"));
        assert!(content.contains("Sub 2"));
    }

    #[test]
    fn tasks_drafts_appear_at_end() {
        let beans = vec![
            make_bean("b-1", "Active", BeanStatus::Todo, BeanType::Task, None),
            make_bean("b-2", "Draft one", BeanStatus::Draft, BeanType::Task, None),
        ];
        let (content, _) = render(&beans);
        let tasks_pos = content.find("## Tasks").unwrap();
        let drafts_pos = content.find("## Drafts").unwrap();
        assert!(tasks_pos < drafts_pos);
    }
}
