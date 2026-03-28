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
    let page_content = render::render_bean_section(bean, all_beans);
    let path = format!("beans/{}.md", bean.id);
    let mut chapter = Chapter::new(&bean.frontmatter.title, page_content, &path, vec![]);
    chapter.source_path = None;
    BookItem::Chapter(chapter)
}

fn make_type_section(label: &str, beans: &[&Bean], all_beans: &[Bean]) -> BookItem {
    let slug = label.to_lowercase().replace(' ', "-");
    let path = format!("beans/type-{slug}.md");

    // Type section index page: list of beans with links
    let mut content = format!("# {label}\n\n");
    for bean in beans {
        content.push_str(&format!(
            "- [{}]({}.md) — *{}*\n",
            bean.frontmatter.title,
            bean.id,
            render::status_label(&bean.frontmatter.status)
        ));
    }

    let mut chapter = Chapter::new(label, content, &path, vec![]);
    chapter.source_path = None;

    // Each bean is a sub_item of its type section -> own page
    chapter.sub_items = beans
        .iter()
        .map(|bean| make_bean_chapter(bean, all_beans))
        .collect();

    BookItem::Chapter(chapter)
}

/// Render the All Tasks chapter with type sections and individual bean pages.
/// Returns (index page content, sub_item chapters).
pub fn render(beans: &[Bean]) -> (String, Vec<BookItem>) {
    let mut content = String::from("# All Tasks\n\n");
    let mut sub_items: Vec<BookItem> = Vec::new();

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

        // Index page summary
        content.push_str(&format!("## {section_label}\n\n"));

        for bean in &matching {
            content.push_str(&format!(
                "- {} — *{}*",
                bean.frontmatter.title,
                render::status_label(&bean.frontmatter.status)
            ));
            content.push('\n');

            if *bean_type == BeanType::Epic {
                let children = epic_children(&bean.id);
                for child in &children {
                    content.push_str(&format!(
                        "  - {} — *{}*\n",
                        child.frontmatter.title,
                        render::status_label(&child.frontmatter.status)
                    ));
                }
            }
        }

        content.push('\n');

        sub_items.push(make_type_section(section_label, &matching, beans));
    }

    // Drafts
    let drafts: Vec<&Bean> = beans
        .iter()
        .filter(|b| b.frontmatter.status == BeanStatus::Draft)
        .collect();

    if !drafts.is_empty() {
        content.push_str("## Drafts\n\n");
        for bean in &drafts {
            content.push_str(&format!(
                "- {} — *Draft*\n",
                bean.frontmatter.title
            ));
        }
        content.push('\n');

        sub_items.push(make_type_section("Drafts", &drafts, beans));
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
    fn tasks_creates_type_sections_with_bean_sub_items() {
        let beans = vec![
            make_bean("b-1", "Feat one", BeanStatus::Todo, BeanType::Feature, None),
            make_bean("b-2", "Feat two", BeanStatus::Done, BeanType::Feature, None),
            make_bean("b-3", "A bug", BeanStatus::Todo, BeanType::Bug, None),
        ];
        let (_, sub_items) = render(&beans);
        assert_eq!(sub_items.len(), 2); // Features, Bugs

        // Features section should have 2 bean sub_items
        if let BookItem::Chapter(features_ch) = &sub_items[0] {
            assert_eq!(features_ch.name, "Features");
            assert_eq!(features_ch.sub_items.len(), 2);

            // Each bean has its own path
            if let BookItem::Chapter(bean_ch) = &features_ch.sub_items[0] {
                let path = bean_ch.path.as_ref().unwrap().to_string_lossy();
                assert_eq!(path, "beans/b-1.md");
            }
        }
    }

    #[test]
    fn tasks_bean_pages_have_stable_paths() {
        let beans = vec![
            make_bean("b-1", "Task one", BeanStatus::Todo, BeanType::Task, None),
        ];
        let (_, sub_items) = render(&beans);
        if let BookItem::Chapter(type_ch) = &sub_items[0] {
            if let BookItem::Chapter(bean_ch) = &type_ch.sub_items[0] {
                assert_eq!(
                    bean_ch.path.as_ref().unwrap().to_string_lossy(),
                    "beans/b-1.md"
                );
                assert!(bean_ch.content.contains("Body of Task one"));
            }
        }
    }

    #[test]
    fn tasks_drafts_appear_at_end() {
        let beans = vec![
            make_bean("b-1", "Active", BeanStatus::Todo, BeanType::Task, None),
            make_bean("b-2", "Draft one", BeanStatus::Draft, BeanType::Task, None),
        ];
        let (content, sub_items) = render(&beans);
        let tasks_pos = content.find("## Tasks").unwrap();
        let drafts_pos = content.find("## Drafts").unwrap();
        assert!(tasks_pos < drafts_pos);

        let last = sub_items.last().unwrap();
        if let BookItem::Chapter(ch) = last {
            assert_eq!(ch.name, "Drafts");
        }
    }
}
