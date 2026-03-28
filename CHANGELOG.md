# Changelog

## [0.2.0] - 2026-03-28

### Added
- Load beans from `archive/` subdirectory — archived beans now appear in All Tasks under their normal type sections (#2)

### Fixed
- Clippy warnings: needless lifetime, collapsible if-let, ptr_arg, dead_code on config structs

## [0.1.0] - 2026-03-27

### Added
- mdBook preprocessor with `{{#beans-active-tasks}}` and `{{#beans-all-tasks}}` markers
- Active Tasks page with In Progress / Todo sections and card styling
- Done tasks as a separate sub-page under Active Tasks
- All Tasks page with type sections (Epics, Features, Tasks, Bugs, Spikes, Chores, Drafts)
- Bean detail pages with metadata table, subtask listing, parent/blocked-by links
- Epic cards with progress badges and subtask status icons
- CSS for card styling, sidebar number hiding, and full-width metadata tables
- Config loading from `.beans.yml` with upward directory search
