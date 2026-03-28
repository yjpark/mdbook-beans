use crate::bean::{Bean, BeanStatus, BeanType};

/// Format a status label for display.
pub fn status_label(status: &BeanStatus) -> &'static str {
    match status {
        BeanStatus::Draft => "Draft",
        BeanStatus::Todo => "Todo",
        BeanStatus::InProgress => "In Progress",
        BeanStatus::Done | BeanStatus::Completed => "Done",
        BeanStatus::Archived => "Archived",
    }
}

/// Format a type label for display.
pub fn type_label(bean_type: &BeanType) -> &'static str {
    match bean_type {
        BeanType::Epic => "Epic",
        BeanType::Feature => "Feature",
        BeanType::Task => "Task",
        BeanType::Bug => "Bug",
        BeanType::Spike => "Spike",
        BeanType::Chore => "Chore",
    }
}

/// Render a compact card for use in Kanban view.
pub fn render_bean_card(bean: &Bean, children: &[&Bean]) -> String {
    let mut card = format!("### {}", bean.frontmatter.title);

    // Type and priority badges
    card.push_str(&format!(
        "\n\n`{}` · `{}`",
        type_label(&bean.frontmatter.bean_type),
        bean.frontmatter.priority
    ));

    // Epic progress badge
    if bean.frontmatter.bean_type == BeanType::Epic && !children.is_empty() {
        let done_count = children
            .iter()
            .filter(|c| {
                c.frontmatter.status == BeanStatus::Done
                    || c.frontmatter.status == BeanStatus::Completed
            })
            .count();
        card.push_str(&format!(" · ({}/{} done)", done_count, children.len()));
    }

    // Subtask epic indicator
    if let Some(parent_id) = &bean.frontmatter.parent {
        card.push_str(&format!(" · ↑ {parent_id}"));
    }

    // Link to bean's own page
    card.push_str(&format!("\n\n[View →](beans/{}.md)\n", bean.id));

    card
}

/// Render a bean as its own page.
pub fn render_bean_section(bean: &Bean, all_beans: &[Bean]) -> String {
    let mut page = format!("# {} (`{}`)\n\n", bean.frontmatter.title, bean.id);

    // Metadata table
    page.push_str("| | |\n|---|---|\n");
    page.push_str(&format!(
        "| **Status** | {} |\n",
        status_label(&bean.frontmatter.status)
    ));
    page.push_str(&format!(
        "| **Type** | {} |\n",
        type_label(&bean.frontmatter.bean_type)
    ));
    page.push_str(&format!(
        "| **Priority** | {} |\n",
        bean.frontmatter.priority
    ));

    if !bean.frontmatter.tags.is_empty() {
        let tags = bean.frontmatter.tags.join(", ");
        page.push_str(&format!("| **Tags** | {tags} |\n"));
    }

    if let Some(parent_id) = &bean.frontmatter.parent {
        page.push_str(&format!(
            "| **Parent** | [\\<{parent_id}\\>]({parent_id}.md) |\n"
        ));
    }

    if !bean.frontmatter.blocked_by.is_empty() {
        let links: Vec<String> = bean
            .frontmatter
            .blocked_by
            .iter()
            .map(|id| format!("[\\<{id}\\>]({id}.md)"))
            .collect();
        page.push_str(&format!("| **Blocked by** | {} |\n", links.join(", ")));
    }

    // Subtasks list (for epics)
    let children: Vec<&Bean> = all_beans
        .iter()
        .filter(|b| b.frontmatter.parent.as_deref() == Some(&bean.id))
        .collect();

    if !children.is_empty() {
        page.push_str(&format!(
            "| **Subtasks** | {} |\n",
            children
                .iter()
                .map(|c| format!("[\\<{}\\>]({}.md)", c.id, c.id))
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }

    page.push('\n');

    // Body
    if !bean.body.is_empty() {
        page.push_str(&bean.body);
        page.push('\n');
    }

    page
}
