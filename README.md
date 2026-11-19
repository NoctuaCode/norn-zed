# Norn for Zed (`norn-zed`)

A Rust renderer for dated Markdown task notes. It generates daily, Monday–Sunday weekly, and calendar-month timelines for Zed's built-in Markdown Preview. It is **not** an installable Zed extension: the [public Rust extension API](https://docs.rs/zed_extension_api/latest/zed_extension_api/trait.Extension.html) cannot register custom views or actions.

## Set up in Zed

1. Install Rust 1.88+ and run `cargo install --path . --offline --locked` from this checkout (Bash or Fish). Make sure `zed-timeline` is on the `PATH` used by Zed's task shell.
2. Open the workspace where you keep your notes. Run **zed: open project tasks**, then copy the [example tasks](examples/tasks.json) into `.zed/tasks.json` (merge the entries if you already have tasks). In each entry, set `--notes` to your daily notes directory, either relative to the workspace root or as an absolute path.
3. Run **task: spawn** and choose **Timeline: day**, **Timeline: week**, or **Timeline: month**. The task writes `.zed/timeline-preview.md`. If a dated daily note is active, the view uses its date; otherwise it uses today. Save the source note first if you want its latest edits included.
4. Open `.zed/timeline-preview.md`, then run **markdown: open preview to the side** to open it on the right. For another placement, open **markdown: open preview** in the current pane and use Zed's pane controls to position it, or drag the preview tab to an edge of an editor pane. The split and its width belong to Zed, not the renderer; there is no dedicated timeline dock or global dock-side setting for this preview. Rerun a task to regenerate the same preview file.

## Command palette

No keybinding is required. In the Command Palette, run **task: spawn**, then search for `Timeline` and choose day, week, or month. Run **markdown: open preview to the side** from the generated Markdown file to open the preview.

For shorter commands, run **zed: open settings file** and merge these [command aliases](https://zed.dev/docs/reference/all-settings#command-aliases) into your user `settings.json`:

```json
{
  "command_aliases": {
    "norn": "task: spawn",
    "norn preview": "markdown: open preview to the side"
  }
}
```

Type `norn` in the Command Palette to open the task picker, or `norn preview` with the generated Markdown file active. Stock Zed [does not list individual custom tasks directly in the Command Palette](https://github.com/zed-industries/zed/issues/14505). The alias opens the picker; it does not run a particular view.

## Optional keybindings

No keys are installed. In **zed: open keymap file**, add the following entries inside a `Workspace` binding group, replacing the example keys with your own. Zed uses [`task::Spawn`](https://zed.dev/docs/tasks#custom-keybindings-for-tasks) to start the renderer and its built-in Markdown action to open the preview.

```json
{
  "context": "Workspace",
  "bindings": {
    "alt-1": ["task::Spawn", { "task_name": "Timeline: day" }],
    "alt-2": ["task::Spawn", { "task_name": "Timeline: week" }],
    "alt-3": ["task::Spawn", { "task_name": "Timeline: month" }],
    "alt-4": "markdown::OpenPreviewToTheSide"
  }
}
```

## Command line

From the workspace root, in either Bash or Fish:

```sh
zed-timeline --notes notes/Daily --view week --date 2026-11-19
```

Without `--output`, the command prints Markdown. With `--output FILE.md`, it writes a generated file outside the notes directory; it refuses to overwrite ordinary Markdown files. Use `--date YYYY-MM-DD` to browse a specific period, or omit it to use `--active-file` or today's local date (`date` on `PATH`).

| Option | Purpose |
| --- | --- |
| `--notes DIR` | Required. Directory containing `YYYY-MM-DD.md` notes |
| `--view day\|week\|month` | Period to render; defaults to `day` |
| `--date YYYY-MM-DD` | Explicit date to anchor the period |
| `--active-file PATH` | Infer the date from a file in the notes directory if `--date` is omitted |
| `--output FILE.md` | Write a preview instead of printing to standard output |
| `--help` | Show command usage |

## Note format

```markdown
# 2026-11-19

## Schedule

- [x] 08:00 - 09:00 | Project: Review changes
- [ ] 09:00:00 - 09:30:00 | Project: Follow up

## Other tasks

- Update the project notes
```

Each `█` represents a 15-minute slot, and overlaps occupy separate rows. Empty days show **No task yet**. Untimed items are listed in daily view and counted in weekly/monthly totals. Bars use the preview's theme text colour, with task status shown as text in daily view; Markdown Preview does not expose semantic per-bar highlights. This reads saved notes, not live editor buffers, and does not edit tasks or install shortcuts.

The project breakdown beneath each view shows exact scheduled time and time marked done. Rows are nested by `Project: Description`, with descriptions grouped by capitalization and whitespace. Untimed and canceled tasks are excluded, and overlapping blocks are summed separately.
