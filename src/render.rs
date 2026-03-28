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

/// Format a bean reference link: `Title (id)`
fn bean_ref(id: &str, title: &str) -> String {
    format!("[{title} ({id})]({id}.md)")
}

/// Look up a bean by ID and format a reference link.
fn bean_ref_by_id<'a>(id: &str, all_beans: &'a [Bean]) -> String {
    if let Some(b) = all_beans.iter().find(|b| b.id == id) {
        bean_ref(id, &b.frontmatter.title)
    } else {
        format!("[({id})]({id}.md)")
    }
}

/// Render a bean as its own page.
pub fn render_bean_section(bean: &Bean, all_beans: &[Bean]) -> String {
    let mut page = format!("# {} ({})\n\n", bean.frontmatter.title, bean.id);

    // Metadata table (full width via HTML style)
    page.push_str("<div class=\"bean-meta\">\n\n");
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
            "| **Parent** | {} |\n",
            bean_ref_by_id(parent_id, all_beans)
        ));
    }

    if !bean.frontmatter.blocked_by.is_empty() {
        let links: Vec<String> = bean
            .frontmatter
            .blocked_by
            .iter()
            .map(|id| bean_ref_by_id(id, all_beans))
            .collect();
        page.push_str(&format!("| **Blocked by** | {} |\n", links.join(", ")));
    }

    // Subtasks list (for epics) — one per row
    let children: Vec<&Bean> = all_beans
        .iter()
        .filter(|b| b.frontmatter.parent.as_deref() == Some(&bean.id))
        .collect();

    if !children.is_empty() {
        for (i, child) in children.iter().enumerate() {
            let label = if i == 0 { "**Subtasks**" } else { "" };
            page.push_str(&format!(
                "| {} | {} |\n",
                label,
                bean_ref(&child.id, &child.frontmatter.title)
            ));
        }
    }

    page.push_str("\n</div>\n\n");

    // Body
    if !bean.body.is_empty() {
        page.push_str(&bean.body);
        page.push('\n');
    }

    page
}
