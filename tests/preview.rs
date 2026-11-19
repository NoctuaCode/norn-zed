use std::{fs, path::{Path, PathBuf}, process::{Command, Output}};

struct Fixture(PathBuf);

impl Fixture {
    fn new(case: &str) -> Self {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("target").join(format!("preview-{case}-{}", std::process::id()));
        fs::create_dir(&path).unwrap();
        fs::create_dir(path.join("notes")).unwrap();
        Self(path)
    }

    fn run(&self, args: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_zed-timeline"))
            .current_dir(&self.0)
            .args(["--notes", "notes"])
            .args(args)
            .output().unwrap()
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn week_and_day_preserve_overlaps_and_ignore_code_samples() {
    let f = Fixture::new("period");
    fs::write(f.0.join("notes/2024-02-29.md"), concat!(
        "# Leap day\n## Schedule\n",
        "- [x] 08:00:00 - 09:00:00 | Team: early\n",
        "- [?] 08:30 - 09:00 | Team: overlap\n",
        "- [ ] 09:00 - 08:00 | Team: invalid\n",
        "~~~md\n- [x] 10:00 - 11:00 | ignored example\n~~~\n",
        "## Other tasks\n- Call back\n",
    )).unwrap();

    let week = f.run(&["--view", "week", "--active-file", "notes/2024-02-29.md"]);
    assert!(week.status.success(), "{}", String::from_utf8_lossy(&week.stderr));
    let week = String::from_utf8(week.stdout).unwrap();
    assert!(week.contains("week of 2024-02-26") && week.contains("2024-03-03"));
    let mut lines = week.lines().skip_while(|line| !line.trim_start().starts_with("2024-02-29"));
    assert!(lines.next().unwrap().contains("████  │ 1/4 done"));
    assert_eq!(lines.next(), Some("                  ██"));

    let day = f.run(&["--date", "2024-02-29"]);
    let day = String::from_utf8(day.stdout).unwrap();
    assert!(day.contains("Waiting   Team: overlap"));
    assert!(day.contains("Invalid schedule:") && day.contains("Call back"));
    assert!(!day.contains("ignored example"));

    let month = f.run(&["--view", "month", "--date", "2024-02-29"]);
    let month = String::from_utf8(month.stdout).unwrap();
    assert!(month.contains("2024-02-29") && !month.contains("2024-03-01"));
    assert!(month.lines().any(|line| line.contains("2024-02-28") && line.contains("No task yet")));
}

#[test]
fn output_replaces_only_generated_previews_outside_the_notes() {
    let f = Fixture::new("output");
    fs::write(f.0.join("ordinary.md"), "handwritten\n").unwrap();
    assert!(!f.run(&["--date", "2024-02-29", "--output", "ordinary.md"]).status.success());
    assert_eq!(fs::read_to_string(f.0.join("ordinary.md")).unwrap(), "handwritten\n");
    assert!(!f.run(&["--date", "2024-02-29", "--output", "notes/2024-02-29.md"]).status.success());
    assert!(!f.0.join("notes/2024-02-29.md").exists());
    assert!(f.run(&["--date", "2024-02-29", "--output", "preview.md"]).status.success());
    assert!(f.run(&["--date", "2024-03-01", "--output", "preview.md"]).status.success());
    assert!(fs::read_to_string(f.0.join("preview.md")).unwrap().contains("Daily timeline · 2024-03-01\n\nNo task yet"));
}

#[test]
fn groups_task_durations_by_project_and_normalized_description() {
    let f = Fixture::new("project-totals");
    fs::write(f.0.join("notes/2024-02-28.md"), concat!(
        "## Schedule\n",
        "- [x] 08:00:00 - 08:15:12 | Alpha: Review PR\n",
        "- [-] 08:30 - 09:30 | Alpha: Review PR\n",
        "- [x] 08:00 - 08:15 | Beta: review pr\n",
    )).unwrap();
    fs::write(f.0.join("notes/2024-02-29.md"),
        "## Schedule\n- [ ] 08:00 - 08:30 | Alpha: review   pr\n").unwrap();
    let report = f.run(&["--view", "month", "--date", "2024-02-29"]);
    assert!(report.status.success());
    let report = String::from_utf8(report.stdout).unwrap();
    let (_, totals) = report.split_once("## Project time (scheduled / marked done)\n\n").unwrap();
    assert_eq!(totals, concat!(
        "    00:45:12 / 00:15:12  Alpha\n",
        "      └─ 00:45:12 / 00:15:12  Review PR\n",
        "    00:15:00 / 00:15:00  Beta\n",
        "      └─ 00:15:00 / 00:15:00  review pr\n",
    ));
}
