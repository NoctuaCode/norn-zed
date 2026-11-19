mod calendar;
mod render;
mod schedule;

use calendar::{Date, View};
use std::env;
use std::error::Error;
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

const HELP: &str = "Usage: zed-timeline --notes DIR [--view day|week|month] [--date YYYY-MM-DD]\n                    [--active-file PATH] [--output FILE]\n\nReads saved notes only. If --date is omitted, use the active daily note\nor today's local date. Prints Markdown unless --output is supplied.";
type Result<T, E = Box<dyn Error>> = std::result::Result<T, E>;

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("zed-timeline: {err}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<()> {
    let mut notes = None;
    let mut view = View::Day;
    let mut date = None;
    let mut active = None;
    let mut output = None;
    let mut args = env::args_os().skip(1);
    while let Some(arg) = args.next() {
        let flag = arg.to_str().ok_or("option names must be UTF-8")?;
        if flag == "--help" || flag == "-h" {
            println!("{HELP}");
            return Ok(());
        }
        let value = args.next().ok_or_else(|| format!("missing value for {flag}"))?;
        match flag {
            "--notes" => notes = Some(PathBuf::from(value)),
            "--view" => view = value.to_str().ok_or("view must be UTF-8")?.parse()?,
            "--date" => date = Some(value.to_str().ok_or("date must be UTF-8")?.parse()?),
            "--active-file" => active = Some(PathBuf::from(value)),
            "--output" => output = Some(PathBuf::from(value)),
            _ => return Err(format!("unknown option: {flag}").into()),
        }
    }
    let notes = notes.ok_or("--notes DIR is required; see --help")?;
    let notes = notes.canonicalize().map_err(|e| format!("{}: {e}", notes.display()))?;
    if !notes.is_dir() {
        return Err(format!("{} is not a directory", notes.display()).into());
    }
    let active_date = active.as_deref().and_then(|path| date_for_file(path, &notes));
    let date = match date.or(active_date) {
        Some(date) => date,
        None => local_date()?,
    };
    let (title, period) = view.period(date)?;
    let mut documents = Vec::with_capacity(period.len());
    for day in period {
        let path = notes.join(format!("{day}.md"));
        let text = match fs::read_to_string(&path) {
            Ok(text) => text,
            Err(err) if err.kind() == io::ErrorKind::NotFound => String::new(),
            Err(err) => return Err(format!("{}: {err}", path.display()).into()),
        };
        documents.push((day, text));
    }
    let markdown = render::document(&title, &documents);
    if let Some(path) = output {
        save_preview(&path, &notes, &markdown)?;
        println!("Timeline written to {}", path.display());
    } else {
        io::stdout().lock().write_all(markdown.as_bytes())?;
    }
    Ok(())
}

fn date_for_file(path: &Path, notes: &Path) -> Option<Date> {
    if path.extension()? != "md" || path.parent()?.canonicalize().ok()? != notes {
        return None;
    }
    path.file_stem()?.to_str()?.parse().ok()
}

fn local_date() -> Result<Date> {
    let output = Command::new("date").arg("+%F").output()?;
    if !output.status.success() {
        return Err("could not determine the local date; pass --date".into());
    }
    Ok(std::str::from_utf8(&output.stdout)?.trim().parse()?)
}

fn save_preview(path: &Path, notes: &Path, markdown: &str) -> Result<()> {
    let filename = path.file_name().ok_or("output must be a Markdown file")?;
    if path.extension().is_none_or(|ext| ext != "md") {
        return Err("output must have an .md extension".into());
    }
    let parent = path.parent().filter(|p| !p.as_os_str().is_empty()).unwrap_or(Path::new("."));
    let parent = parent.canonicalize().map_err(|e| format!("{}: {e}", parent.display()))?;
    if parent.starts_with(notes) {
        return Err("keep the generated preview outside the notes directory".into());
    }
    let path = parent.join(filename);
    match path.symlink_metadata() {
        Ok(meta) if meta.is_file() => {
            use std::io::BufRead as _;
            let mut reader = io::BufReader::new(fs::File::open(&path)?);
            let mut first_line = String::new();
            reader.read_line(&mut first_line)?;
            if first_line.trim_end() != render::MARKER {
                return Err(format!("refusing to overwrite {}: not a generated preview", path.display()).into());
            }
        }
        Ok(_) => return Err(format!("refusing to replace {}: not a regular file", path.display()).into()),
        Err(e) if e.kind() == io::ErrorKind::NotFound => {}
        Err(e) => return Err(e.into()),
    }
    // Keep the previous preview intact if rendering or writing fails.
    let tmp = parent.join(format!(".zed-timeline-{}.tmp", std::process::id()));
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt as _;
        options.mode(0o600);
    }
    let mut file = options.open(&tmp)?;
    let result = file.write_all(markdown.as_bytes());
    drop(file);
    if let Err(err) = result.and_then(|()| fs::rename(&tmp, &path)) {
        let _ = fs::remove_file(&tmp);
        return Err(err.into());
    }
    Ok(())
}
