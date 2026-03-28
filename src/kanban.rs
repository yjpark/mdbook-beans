use crate::bean::{Bean, BeanStatus};
use crate::render;

/// Render the Kanban chapter content.
/// Groups beans by status (excluding Draft and Archived) into columns.
pub fn render(beans: &[Bean]) -> String {
    let mut content = String::from("# Kanban\n\n");

    let statuses = [
        (BeanStatus::InProgress, "In Progress"),
        (BeanStatus::Todo, "Todo"),
        (BeanStatus::Done, "Done"),
    ];

    for (status, label) in &statuses {
        content.push_str(&format!("## {label}\n\n"));

        let matching: Vec<&Bean> = beans
            .iter()
            .filter(|b| &b.frontmatter.status == status)
            .collect();

        if matching.is_empty() {
            content.push_str("*No tasks*\n\n");
            continue;
        }

        for bean in &matching {
            let children: Vec<&Bean> = beans
                .iter()
                .filter(|b| b.frontmatter.parent.as_deref() == Some(&bean.id))
                .collect();
            content.push_str(&render::render_bean_card(bean, &children));
            content.push('\n');
        }
    }

    content
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
        let output = render(&beans);
        assert!(output.contains("Active"));
        assert!(!output.contains("Draft"));
        assert!(!output.contains("Archived"));
    }

    #[test]
    fn kanban_groups_by_status() {
        let beans = vec![
            make_bean("b-1", "Todo task", BeanStatus::Todo, BeanType::Task),
            make_bean("b-2", "WIP task", BeanStatus::InProgress, BeanType::Feature),
            make_bean("b-3", "Done task", BeanStatus::Done, BeanType::Bug),
        ];
        let output = render(&beans);

        // Verify ordering: In Progress before Todo before Done
        let ip_pos = output.find("## In Progress").unwrap();
        let todo_pos = output.find("## Todo").unwrap();
        let done_pos = output.find("## Done").unwrap();
        assert!(ip_pos < todo_pos);
        assert!(todo_pos < done_pos);

        assert!(output.contains("WIP task"));
        assert!(output.contains("Todo task"));
        assert!(output.contains("Done task"));
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
        let output = render(&beans);
        assert!(output.contains("(1/2 done)"));
    }
}
