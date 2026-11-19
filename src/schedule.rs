#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Status {
    Done,
    Open,
    Waiting,
    Canceled,
    Other,
}

impl Status {
    pub fn label(self) -> &'static str {
        match self {
            Self::Done => "Done",
            Self::Open => "Open",
            Self::Waiting => "Waiting",
            Self::Canceled => "Canceled",
            Self::Other => "—",
        }
    }
}

pub struct Task<'a> {
    pub status: Status,
    pub label: &'a str,
    pub range: Option<Range<'a>>,
    pub invalid: bool,
}

pub struct Range<'a> {
    pub start: u32,
    pub end: u32,
    pub from: &'a str,
    pub to: &'a str,
}

fn seconds(value: &str) -> Option<u32> {
    let mut parts = value.split(':');
    let [Some(h), Some(m), s, None] = [parts.next(), parts.next(), parts.next(), parts.next()] else {
        return None;
    };
    let parse = |part: &str| {
        (part.len() == 2 && part.bytes().all(|b| b.is_ascii_digit())).then(|| part.parse::<u32>().unwrap())
    };
    let (h, m, s) = (parse(h)?, parse(m)?, parse(s.unwrap_or("00"))?);
    (m < 60 && s < 60 && (h < 24 || (h == 24 && m == 0 && s == 0))).then_some(h * 3600 + m * 60 + s)
}

fn word(text: &str) -> (&str, &str) {
    let at = text.find(char::is_whitespace).unwrap_or(text.len());
    (&text[..at], text[at..].trim_start())
}

fn range(text: &str) -> Option<(Range<'_>, &str)> {
    let (from, rest) = word(text);
    let (dash, rest) = word(rest);
    let (to, label) = word(rest);
    let label = label.strip_prefix('|').unwrap_or(label).trim();
    let (start, end) = (seconds(from)?, seconds(to)?);
    (dash == "-" && start < end && !label.is_empty()).then_some((Range { start, end, from, to }, label))
}

fn list_item(line: &str) -> Option<&str> {
    let (marker, text) = word(line);
    let ordered = marker.strip_suffix(['.', ')']).is_some_and(|n| !n.is_empty() && n.bytes().all(|b| b.is_ascii_digit()));
    (!text.is_empty() && (matches!(marker, "-" | "+" | "*") || ordered)).then_some(text)
}

pub fn parse(markdown: &str) -> Vec<Task<'_>> {
    let mut tasks = Vec::new();
    let mut section = "";
    let mut fence = None;
    for original in markdown.lines() {
        let line = original.trim_start();
        if original.len() - line.len() <= 3 && let Some(marker @ (b'`' | b'~')) = line.bytes().next() {
            let len = line.bytes().take_while(|b| *b == marker).count();
            if len >= 3 {
                match fence {
                    None => fence = Some((marker, len)),
                    Some((open, n)) if open == marker && len >= n && line[len..].trim().is_empty() => fence = None,
                    _ => {}
                }
                continue;
            }
        }
        if fence.is_some() {
            continue;
        }
        let (heading, title) = word(line);
        if original.len() - line.len() <= 3 && matches!(heading, "#" | "##") {
            section = if heading == "##" {
                let trimmed = title.trim_end();
                let without = trimmed.trim_end_matches('#');
                if without.ends_with(char::is_whitespace) { without.trim_end() } else { trimmed }
            } else { "" };
            continue;
        }
        if !matches!(section, "Schedule" | "Other tasks") {
            continue;
        }
        let Some(item) = list_item(line) else { continue };
        let (status, text) = match item.as_bytes() {
            [b'[', mark, b']', ..] if mark.is_ascii() => {
                let status = match mark {
                    b'x' | b'X' => Status::Done,
                    b' ' => Status::Open,
                    b'?' => Status::Waiting,
                    b'-' => Status::Canceled,
                    _ => Status::Other,
                };
                (status, item[3..].trim())
            }
            _ => (Status::Other, item.trim()),
        };
        let time_like = section == "Schedule"
            && text.split_once(':').is_some_and(|(h, _)| !h.is_empty() && h.bytes().all(|b| b.is_ascii_digit()));
        if time_like && let Some((range, label)) = range(text) {
            tasks.push(Task { status, label, range: Some(range), invalid: false });
        } else {
            tasks.push(Task { status, label: text, range: None, invalid: time_like });
        }
    }
    tasks.sort_by_key(|task| task.range.as_ref().map_or(u32::MAX, |range| range.start));
    tasks
}
