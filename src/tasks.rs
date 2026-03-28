use mdbook_preprocessor::book::{BookItem, Chapter, SectionNumber};

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

fn bean_link(bean: &Bean) -> String {
    format!(
        "[{} ({})]({}.md)",
        bean.frontmatter.title, bean.id, bean.id
    )
}

fn make_bean_chapter(
    bean: &Bean,
    all_beans: &[Bean],
    parent_number: &[u32],
    index: u32,
    parent_names: Vec<String>,
) -> BookItem {
    let page_content = render::render_bean_section(bean, all_beans);
    let path = format!("beans/{}.md", bean.id);
    let name = format!("{} ({})", bean.frontmatter.title, bean.id);
    let mut chapter = Chapter::new(&name, page_content, &path, parent_names);
    chapter.source_path = None;
    let mut num = parent_number.to_vec();
    num.push(index);
    chapter.number = Some(SectionNumber::new(num));
    BookItem::Chapter(chapter)
}

fn make_type_section(
    label: &str,
    beans: &[&Bean],
    all_beans: &[Bean],
    parent_number: &[u32],
    section_index: u32,
) -> BookItem {
    let slug = label.to_lowercase().replace(' ', "-");
    let path = format!("beans/type-{slug}.md");
    let parent_names = vec!["All Tasks".to_string()];

    let mut section_number = parent_number.to_vec();
    section_number.push(section_index);

    // Type section content: list of beans with links
    let mut content = format!("# {label}\n\n");
    for bean in beans {
        content.push_str(&format!(
            "- {} — *{}*\n",
            bean_link(bean),
            render::status_label(&bean.frontmatter.status)
        ));
    }

    let mut chapter = Chapter::new(label, content, &path, parent_names.clone());
    chapter.source_path = None;
    chapter.number = Some(SectionNumber::new(section_number.clone()));

    // Each bean is a sub_item of its type section
    let bean_parents = {
        let mut p = parent_names;
        p.push(label.to_string());
        p
    };
    chapter.sub_items = beans
        .iter()
        .enumerate()
        .map(|(i, bean)| {
            make_bean_chapter(bean, all_beans, &section_number, (i + 1) as u32, bean_parents.clone())
        })
        .collect();

    BookItem::Chapter(chapter)
}

/// Render the All Tasks chapter with type sections and bean pages.
/// Returns (index page content, sub_item chapters).
pub fn render(beans: &[Bean], parent_number: &[u32]) -> (String, Vec<BookItem>) {
    let mut content = String::from("# All Tasks\n\n");
    let mut sub_items: Vec<BookItem> = Vec::new();
    let mut section_index: u32 = 1;

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
                bean_link(bean),
                render::status_label(&bean.frontmatter.status)
            ));
            content.push('\n');

            if *bean_type == BeanType::Epic {
                let children = epic_children(&bean.id);
                for child in &children {
                    content.push_str(&format!(
                        "  - {} — *{}*\n",
                        bean_link(child),
                        render::status_label(&child.frontmatter.status)
                    ));
                }
            }
        }

        content.push('\n');

        sub_items.push(make_type_section(
            section_label,
            &matching,
            beans,
            parent_number,
            section_index,
        ));
        section_index += 1;
    }

    // Drafts
    let drafts: Vec<&Bean> = beans
        .iter()
        .filter(|b| b.frontmatter.status == BeanStatus::Draft)
        .collect();

    if !drafts.is_empty() {
        content.push_str("## Drafts\n\n");
        for bean in &drafts {
            content.push_str(&format!("- {} — *Draft*\n", bean_link(bean)));
        }
        content.push('\n');

        sub_items.push(make_type_section(
            "Drafts",
            &drafts,
            beans,
            parent_number,
            section_index,
        ));
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
        let (content, _) = render(&beans, &[8]);
        assert!(content.contains("## Features"));
        assert!(content.contains("## Tasks"));
        assert!(content.contains("## Bugs"));
    }

    #[test]
    fn tasks_content_has_clickable_links() {
        let beans = vec![
            make_bean("b-1", "Feat one", BeanStatus::Todo, BeanType::Feature, None),
            make_bean("b-2", "A bug", BeanStatus::Todo, BeanType::Bug, None),
        ];
        let (content, _) = render(&beans, &[8]);
        assert!(content.contains("[Feat one (b-1)](b-1.md)"));
        assert!(content.contains("[A bug (b-2)](b-2.md)"));
    }

    #[test]
    fn tasks_type_sections_contain_beans() {
        let beans = vec![
            make_bean("b-1", "Feat one", BeanStatus::Todo, BeanType::Feature, None),
            make_bean("b-2", "Feat two", BeanStatus::Done, BeanType::Feature, None),
            make_bean("b-3", "A bug", BeanStatus::Todo, BeanType::Bug, None),
        ];
        let (_, sub_items) = render(&beans, &[8]);
        // Two type sections: Features, Bugs
        assert_eq!(sub_items.len(), 2);

        if let BookItem::Chapter(features_ch) = &sub_items[0] {
            assert_eq!(features_ch.name, "Features");
            assert_eq!(features_ch.number.as_ref().unwrap().as_slice(), &[8, 1]);
            assert_eq!(features_ch.sub_items.len(), 2);

            if let BookItem::Chapter(bean_ch) = &features_ch.sub_items[0] {
                assert_eq!(bean_ch.path.as_ref().unwrap().to_string_lossy(), "beans/b-1.md");
                assert_eq!(bean_ch.number.as_ref().unwrap().as_slice(), &[8, 1, 1]);
                assert_eq!(bean_ch.parent_names, vec!["All Tasks", "Features"]);
            }
        }
    }

    #[test]
    fn tasks_bean_pages_have_stable_paths() {
        let beans = vec![
            make_bean("b-1", "Task one", BeanStatus::Todo, BeanType::Task, None),
        ];
        let (_, sub_items) = render(&beans, &[8]);
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
        let (content, sub_items) = render(&beans, &[8]);
        let tasks_pos = content.find("## Tasks").unwrap();
        let drafts_pos = content.find("## Drafts").unwrap();
        assert!(tasks_pos < drafts_pos);

        let last = sub_items.last().unwrap();
        if let BookItem::Chapter(ch) = last {
            assert_eq!(ch.name, "Drafts");
        }
    }
}
