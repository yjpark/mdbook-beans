# mdbook-beans

An [mdBook](https://rust-lang.github.io/mdBook/) preprocessor that injects [beans](https://github.com/hmans/beans) task data into your book — turning documentation into a project dashboard.

## What it does

`mdbook-beans` reads your project's `.beans/` markdown files and generates two chapters in your book:

- **Kanban** — A board view with status columns (Todo, In Progress, Done), excluding drafts and archived tasks. Epics show subtask progress badges; subtasks indicate their parent epic.
- **All Tasks** — A structured reference of every bean with stable URLs (`/beans/<bean-id>`), organized by type (Epics, Features, Tasks, Bugs, Drafts).

No beans runtime is required — the preprocessor reads the markdown files and `.beans.yml` config directly.

## Usage

Add the preprocessor to your `book.toml`:

```toml
[preprocessor.beans]
```

Create stub markdown files with markers, then reference them in `SUMMARY.md`:

```markdown
<!-- src/beans/kanban.md -->
{{#beans-kanban}}
```

```markdown
<!-- src/beans/tasks.md -->
{{#beans-tasks}}
```

```markdown
# Summary

- [Introduction](./intro.md)
- [Kanban](beans/kanban.md)
- [All Tasks](beans/tasks.md)
```

The preprocessor finds the markers and replaces the content with generated chapters.

## Requirements

- A `.beans.yml` configuration file at the project root
- Bean markdown files in the configured beans directory (default `.beans/`)

## License

MIT
