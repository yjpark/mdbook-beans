use mdbook_preprocessor::book::{BookItem, Chapter, SectionNumber};

use crate::bean::{Bean, BeanStatus, BeanType};
use crate::render;

/// Check if a bean is done (Done or Completed).
fn is_done(bean: &Bean) -> bool {
    bean.frontmatter.status == BeanStatus::Done
        || bean.frontmatter.status == BeanStatus::Completed
}

/// Check if a bean is active (not Draft/Archived).
fn is_active(bean: &Bean) -> bool {
    !matches!(
        bean.frontmatter.status,
        BeanStatus::Draft | BeanStatus::Archived
    )
}

/// Collect children for a given bean.
fn children_of<'a>(bean: &Bean, beans: &'a [Bean]) -> Vec<&'a Bean> {
    beans
        .iter()
        .filter(|b| b.frontmatter.parent.as_deref() == Some(&bean.id))
        .collect()
}

/// IDs of beans that are subtasks (have a parent).
fn subtask_ids(beans: &[Bean]) -> Vec<String> {
    beans
        .iter()
        .filter(|b| b.frontmatter.parent.is_some())
        .map(|b| b.id.clone())
        .collect()
}

/// Render cards for a list of beans.
fn render_cards(beans_list: &[&&Bean], all_beans: &[Bean], path_prefix: &str) -> String {
    let mut out = String::new();
    for bean in beans_list {
        let children = children_of(bean, all_beans);
        out.push_str(&render::render_kanban_card(bean, &children, path_prefix));
        out.push('\n');
    }
    out
}

/// Render the Active Tasks chapter.
/// Returns (content, sub_items) where sub_items contains the Done page.
pub fn render(beans: &[Bean], parent_number: &[u32]) -> (String, Vec<BookItem>) {
    let mut content = String::from("# Active Tasks\n\n");

    let sub_ids = subtask_ids(beans);
    let top_level: Vec<&Bean> = beans
        .iter()
        .filter(|b| is_active(b) && !sub_ids.contains(&b.id))
        .collect();

    let statuses = [
        (BeanStatus::InProgress, "In Progress"),
        (BeanStatus::Todo, "Todo"),
    ];

    for (status, label) in &statuses {
        content.push_str(&format!("## {label}\n\n"));

        let matching: Vec<&&Bean> = top_level
            .iter()
            .filter(|b| {
                &b.frontmatter.status == status
                    || (b.frontmatter.bean_type == BeanType::Epic
                        && children_of(b, beans)
                            .iter()
                            .any(|c| &c.frontmatter.status == status))
            })
            .collect();

        if matching.is_empty() {
            content.push_str("*No tasks*\n\n");
            continue;
        }

        content.push_str(&render_cards(&matching, beans, ""));
    }

    // Done page as sub_item
    let done: Vec<&&Bean> = top_level.iter().filter(|b| is_done(b)).collect();
    let done_count = done.len();

    let mut done_content = format!("# Done ({done_count})\n\n");
    if done.is_empty() {
        done_content.push_str("*No tasks*\n\n");
    } else {
        done_content.push_str(&render_cards(&done, beans, ""));
    }

    let mut done_chapter = Chapter::new(
        &format!("Done ({done_count})"),
        done_content,
        "beans/done.md",
        vec!["Active Tasks".to_string()],
    );
    done_chapter.source_path = None;
    let mut num = parent_number.to_vec();
    num.push(1);
    done_chapter.number = Some(SectionNumber::new(num));

    (content, vec![BookItem::Chapter(done_chapter)])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bean::{BeanFrontmatter, BeanType};

    fn make_bean(id: &str, title: &str, status: BeanStatus, bean_type: BeanType) -> Bean {
        Bean {
            id: id.to_string(),
            frontmatter: BeanFrontmatter {
                title: title.to_string(),
                status,
                bean_type,
                priority: "normal".to_string(),
                tags: vec![],
                parent: None,
                blocked_by: vec![],
            },
            body: String::new(),
        }
    }

    #[test]
    fn kanban_excludes_draft_and_archived() {
        let beans = vec![
            make_bean("b-1", "Active", BeanStatus::Todo, BeanType::Task),
            make_bean("b-2", "Draft", BeanStatus::Draft, BeanType::Task),
            make_bean("b-3", "Archived", BeanStatus::Archived, BeanType::Task),
        ];
        let (content, _) = render(&beans, &[7]);
        assert!(content.contains("Active"));
        assert!(!content.contains("Draft"));
        assert!(!content.contains("Archived"));
    }

    #[test]
    fn kanban_groups_by_status() {
        let beans = vec![
            make_bean("b-1", "Todo task", BeanStatus::Todo, BeanType::Task),
            make_bean("b-2", "WIP task", BeanStatus::InProgress, BeanType::Feature),
            make_bean("b-3", "Done task", BeanStatus::Done, BeanType::Bug),
        ];
        let (content, sub_items) = render(&beans, &[7]);

        let ip_pos = content.find("## In Progress").unwrap();
        let todo_pos = content.find("## Todo").unwrap();
        assert!(ip_pos < todo_pos);

        assert!(content.contains("WIP task"));
        assert!(content.contains("Todo task"));
        // Done is in sub_item, not in main content
        assert!(!content.contains("Done task"));

        assert_eq!(sub_items.len(), 1);
        if let BookItem::Chapter(ch) = &sub_items[0] {
            assert!(ch.content.contains("Done task"));
        }
    }

    #[test]
    fn kanban_epic_shows_progress_badge() {
        let beans = vec![
            make_bean("b-epic", "My Epic", BeanStatus::InProgress, BeanType::Epic),
            {
                let mut b = make_bean("b-sub1", "Sub 1", BeanStatus::Done, BeanType::Task);
                b.frontmatter.parent = Some("b-epic".to_string());
                b
            },
            {
                let mut b = make_bean("b-sub2", "Sub 2", BeanStatus::Todo, BeanType::Task);
                b.frontmatter.parent = Some("b-epic".to_string());
                b
            },
        ];
        let (content, _) = render(&beans, &[7]);
        assert!(content.contains("(1/2 done)"));
    }

    #[test]
    fn kanban_subtasks_not_shown_independently() {
        let beans = vec![
            make_bean("b-epic", "My Epic", BeanStatus::InProgress, BeanType::Epic),
            {
                let mut b = make_bean("b-sub1", "Sub 1", BeanStatus::Todo, BeanType::Task);
                b.frontmatter.parent = Some("b-epic".to_string());
                b
            },
            make_bean("b-standalone", "Standalone", BeanStatus::Todo, BeanType::Task),
        ];
        let (content, _) = render(&beans, &[7]);
        // Sub 1 appears under the epic card, not as a standalone card
        assert!(content.contains("Sub 1"));
        // Only the standalone task has its own card in Todo
        let todo_section = &content[content.find("## Todo").unwrap()..];
        assert!(todo_section.contains("Standalone"));
        // Sub 1 does NOT have its own H3 card — only appears in epic's subtask list
        assert!(!content.contains("### [Sub 1"));
    }

    #[test]
    fn kanban_done_is_separate_page() {
        let beans = vec![
            make_bean("b-1", "Done task", BeanStatus::Done, BeanType::Task),
        ];
        let (content, sub_items) = render(&beans, &[7]);
        // Main content should NOT contain done tasks
        assert!(!content.contains("Done task"));
        // Done page is a sub_item
        assert_eq!(sub_items.len(), 1);
        if let BookItem::Chapter(ch) = &sub_items[0] {
            assert_eq!(ch.name, "Done (1)");
            assert!(ch.content.contains("Done task"));
            assert_eq!(ch.path.as_ref().unwrap().to_string_lossy(), "beans/done.md");
        }
    }

    #[test]
    fn kanban_card_has_linked_title() {
        let beans = vec![
            make_bean("b-1", "My Task", BeanStatus::Todo, BeanType::Task),
        ];
        let (content, _) = render(&beans, &[7]);
        assert!(content.contains("[My Task (b-1)](b-1.md)"));
    }

    #[test]
    fn kanban_epic_subtasks_show_status_icons() {
        let beans = vec![
            make_bean("b-epic", "My Epic", BeanStatus::InProgress, BeanType::Epic),
            {
                let mut b = make_bean("b-done", "Done sub", BeanStatus::Done, BeanType::Task);
                b.frontmatter.parent = Some("b-epic".to_string());
                b
            },
            {
                let mut b = make_bean("b-wip", "WIP sub", BeanStatus::InProgress, BeanType::Task);
                b.frontmatter.parent = Some("b-epic".to_string());
                b
            },
            {
                let mut b = make_bean("b-todo", "Todo sub", BeanStatus::Todo, BeanType::Task);
                b.frontmatter.parent = Some("b-epic".to_string());
                b
            },
        ];
        let (content, _) = render(&beans, &[7]);
        assert!(content.contains("✓ [Done sub"));
        assert!(content.contains("▶ [WIP sub"));
        assert!(content.contains("○ [Todo sub"));
    }
}
