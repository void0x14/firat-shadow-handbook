use crate::domain::collab::{CollabSnapshot, CourseEntry, CourseSchedule, PlaybackEntry};
use crate::domain::ports::scraper_port::{ScrapeRequest, ScraperError, ScraperPort};

const ALLOWED_HOSTS: [&str; 2] = ["eu.bbcollab.com", "debsis.firat.edu.tr"];

pub struct CollabScraperAdapter;

impl CollabScraperAdapter {
    pub fn new() -> Self {
        Self
    }
}

impl Default for CollabScraperAdapter {
    fn default() -> Self {
        Self
    }
}

impl ScraperPort for CollabScraperAdapter {
    fn scrape_collab_html(&self, request: ScrapeRequest) -> Result<CollabSnapshot, ScraperError> {
        if request.moodle_session.trim().is_empty() {
            return Err(ScraperError::Unauthorized);
        }

        let courses = parse_course_entries(&request.html)?;
        let playbacks = parse_playback_entries(&request.html)?;

        Ok(CollabSnapshot { courses, playbacks })
    }
}

fn parse_course_entries(html: &str) -> Result<Vec<CourseEntry>, ScraperError> {
    let mut courses = Vec::new();
    let mut cursor = 0usize;

    while let Some(pos) = html[cursor..].find("data-course-id=") {
        let abs = cursor + pos;
        let tag_start = find_tag_start(html, abs).ok_or_else(|| {
            ScraperError::ParseError("Failed to locate course tag start".to_string())
        })?;
        let (tag_text, tag_end) = extract_tag_text(html, tag_start).ok_or_else(|| {
            ScraperError::ParseError("Malformed course tag around data-course-id".to_string())
        })?;
        let fallback_scope_end = next_course_scope_end(html, tag_end);
        let fallback_scope = &html[tag_end..fallback_scope_end];

        let course_id = parse_attr(tag_text, "data-course-id");
        let title = parse_attr(tag_text, "data-course-title").or_else(|| {
            extract_text_between(
                fallback_scope,
                "<span class=\"course-title\">",
                "</span>",
            )
        });
        let instructor = parse_attr(tag_text, "data-instructor").or_else(|| {
            extract_text_between(
                fallback_scope,
                "<span class=\"course-instructor\">",
                "</span>",
            )
        });
        let schedule_raw = parse_attr(tag_text, "data-schedule").or_else(|| {
            extract_text_between(
                fallback_scope,
                "<span class=\"course-schedule\">",
                "</span>",
            )
        });

        let title = title
            .map(|v| html_unescape(v.trim()))
            .filter(|v| !v.is_empty())
            .ok_or_else(|| ScraperError::ParseError("Course title missing".to_string()))?;

        let schedule = schedule_raw
            .map(|s| normalize_schedule(&html_unescape(s.trim())))
            .filter(|s| s.start_iso.is_some() || s.end_iso.is_some() || s.timezone.is_some());

        courses.push(CourseEntry {
            course_id: course_id.map(|v| v.trim().to_string()),
            title,
            instructor: instructor
                .map(|v| html_unescape(v.trim()))
                .filter(|v| !v.is_empty()),
            schedule,
        });

        cursor = tag_end;
    }

    if courses.is_empty() {
        return Err(ScraperError::ParseError(
            "No course entries found in HTML".to_string(),
        ));
    }

    Ok(courses)
}

fn parse_playback_entries(html: &str) -> Result<Vec<PlaybackEntry>, ScraperError> {
    let mut playbacks = Vec::new();
    let mut cursor = 0usize;

    while let Some(tag_start) = find_next_anchor_tag(html, cursor) {
        let (tag_text, tag_end) = extract_tag_text(html, tag_start)
            .ok_or_else(|| ScraperError::ParseError("Malformed <a> tag".to_string()))?;
        let href = parse_attr(tag_text, "href");
        let class_name = parse_attr(tag_text, "class").unwrap_or_default();
        let explicit_playback = parse_attr(tag_text, "data-playback")
            .map(|v| v.eq_ignore_ascii_case("true"))
            .unwrap_or(false);
        let href_marker = href
            .as_deref()
            .map(|h| {
                let lower = h.to_ascii_lowercase();
                lower.contains("recording") || lower.contains("playback")
            })
            .unwrap_or(false);
        let class_marker = class_name.to_ascii_lowercase().contains("playback");
        let is_playback = explicit_playback || href_marker || class_marker;

        if is_playback {
            let href_value = href.ok_or_else(|| {
                ScraperError::ParseError("Playback anchor missing href".to_string())
            })?;

            validate_allowed_url(&href_value)?;

            let label = parse_attr(tag_text, "data-label")
                .or_else(|| extract_anchor_inner_text(html, tag_end));

            playbacks.push(PlaybackEntry {
                course_title: parse_attr(tag_text, "data-course-title")
                    .map(|v| html_unescape(v.trim()))
                    .filter(|v| !v.is_empty()),
                url: href_value,
                label: label
                    .map(|v| html_unescape(v.trim()))
                    .filter(|v| !v.is_empty()),
            });
        }

        cursor = tag_end;
    }

    Ok(playbacks)
}

fn normalize_schedule(raw: &str) -> CourseSchedule {
    let cleaned = raw.trim();
    if cleaned.is_empty() {
        return CourseSchedule {
            start_iso: None,
            end_iso: None,
            timezone: None,
        };
    }

    if cleaned.contains('|') {
        let mut parts = cleaned.split('|').map(|s| s.trim());
        return CourseSchedule {
            start_iso: parts
                .next()
                .map(|s| s.to_string())
                .filter(|s| !s.is_empty()),
            end_iso: parts
                .next()
                .map(|s| s.to_string())
                .filter(|s| !s.is_empty()),
            timezone: parts
                .next()
                .map(|s| s.to_string())
                .filter(|s| !s.is_empty()),
        };
    }

    let timezone = extract_parenthesized(cleaned);
    let no_tz = timezone
        .as_ref()
        .and_then(|tz| cleaned.strip_suffix(&format!(" ({})", tz)))
        .unwrap_or(cleaned)
        .trim();

    let (start_iso, end_iso) = if let Some((start, end)) = no_tz.split_once(" - ") {
        (
            Some(start.trim().to_string()).filter(|s| !s.is_empty()),
            Some(end.trim().to_string()).filter(|s| !s.is_empty()),
        )
    } else {
        (Some(no_tz.to_string()), None)
    };

    CourseSchedule {
        start_iso,
        end_iso,
        timezone,
    }
}

fn validate_allowed_url(url: &str) -> Result<(), ScraperError> {
    if !url.starts_with("https://") {
        return Err(ScraperError::UnsupportedFormat(format!(
            "Playback URL must use https: {}",
            url
        )));
    }

    let without_scheme = &url["https://".len()..];
    let host_port = without_scheme.split('/').next().unwrap_or("");
    let host = host_port.split(':').next().unwrap_or("");

    if ALLOWED_HOSTS.iter().any(|allowed| host.eq_ignore_ascii_case(allowed)) {
        return Ok(());
    }

    Err(ScraperError::UnsupportedFormat(format!(
        "Playback URL host is not allowlisted: {}",
        host
    )))
}

fn find_tag_start(html: &str, from: usize) -> Option<usize> {
    html[..from].rfind('<')
}

fn next_course_scope_end(html: &str, from: usize) -> usize {
    match html[from..].find("data-course-id=") {
        Some(rel) => {
            let next_marker = from + rel;
            find_tag_start(html, next_marker).unwrap_or(html.len())
        }
        None => html.len(),
    }
}

fn extract_tag_text(html: &str, from: usize) -> Option<(&str, usize)> {
    let rest = html.get(from..)?;
    let mut in_single_quote = false;
    let mut in_double_quote = false;

    for (idx, ch) in rest.char_indices() {
        match ch {
            '\'' if !in_double_quote => in_single_quote = !in_single_quote,
            '"' if !in_single_quote => in_double_quote = !in_double_quote,
            '>' if !in_single_quote && !in_double_quote => {
                let end_abs = from + idx + 1;
                return Some((&html[from..end_abs], end_abs));
            }
            _ => {}
        }
    }

    None
}

fn find_next_anchor_tag(html: &str, from: usize) -> Option<usize> {
    let bytes = html.as_bytes();
    let mut idx = from;

    while idx < bytes.len() {
        if bytes[idx] != b'<' {
            idx += 1;
            continue;
        }

        let next = idx + 1;
        if next >= bytes.len() {
            return None;
        }

        let tag_char = bytes[next];
        if !tag_char.eq_ignore_ascii_case(&b'a') {
            idx += 1;
            continue;
        }

        let delimiter = bytes.get(next + 1).copied().unwrap_or(b'>');
        if delimiter.is_ascii_whitespace() || delimiter == b'>' || delimiter == b'/' {
            return Some(idx);
        }

        idx += 1;
    }

    None
}

fn extract_anchor_inner_text(html: &str, from: usize) -> Option<String> {
    let rest = html.get(from..)?;
    let lower = rest.to_ascii_lowercase();
    let close_rel = lower.find("</a>")?;
    Some(rest[..close_rel].to_string())
}

fn parse_attr<'a>(tag: &'a str, name: &str) -> Option<String> {
    for quote in ['"', '\''] {
        let pattern = format!("{}={}", name, quote);
        if let Some(pos) = tag.find(&pattern) {
            let start = pos + pattern.len();
            let rest = tag.get(start..)?;
            let end = rest.find(quote)?;
            return Some(rest[..end].to_string());
        }
    }
    None
}

fn extract_text_between<'a>(input: &'a str, start: &str, end: &str) -> Option<String> {
    let start_pos = input.find(start)? + start.len();
    let rest = input.get(start_pos..)?;
    let end_pos = rest.find(end)?;
    Some(rest[..end_pos].to_string())
}

fn extract_parenthesized(input: &str) -> Option<String> {
    let start = input.rfind('(')?;
    let end = input.rfind(')')?;
    if end <= start + 1 {
        return None;
    }
    Some(input[start + 1..end].trim().to_string())
}

fn html_unescape(input: &str) -> String {
    input
        .replace("&amp;", "&")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_course_and_playback_successfully() {
        let html = r#"
<div data-course-id="42" data-course-title="Yazilim Muhendisligi" data-instructor="Dr. Ada" data-schedule="2026-02-27T10:00:00+03:00|2026-02-27T11:00:00+03:00|Europe/Istanbul"></div>
<a class="playback-link" data-playback="true" data-course-title="Yazilim Muhendisligi" href="https://eu.bbcollab.com/recording/abc" data-label="Kaydi Ac">Kayıt</a>
"#;

        let courses = parse_course_entries(html).expect("course parse should succeed");
        let playbacks = parse_playback_entries(html).expect("playback parse should succeed");

        assert_eq!(courses.len(), 1);
        assert_eq!(courses[0].title, "Yazilim Muhendisligi");
        assert_eq!(courses[0].schedule.as_ref().and_then(|s| s.timezone.clone()), Some("Europe/Istanbul".to_string()));

        assert_eq!(playbacks.len(), 1);
        assert_eq!(playbacks[0].url, "https://eu.bbcollab.com/recording/abc");
    }

    #[test]
    fn returns_parse_error_on_missing_course_title() {
        let html = r#"<div data-course-id="42"></div>"#;
        let err = parse_course_entries(html).expect_err("missing title should fail");
        assert!(matches!(err, ScraperError::ParseError(_)));
    }

    #[test]
    fn course_fallback_does_not_leak_into_next_course() {
        let html = r#"
<div data-course-id="1"></div>
<div data-course-id="2"></div>
<span class="course-title">Second Title</span>
"#;
        let err = parse_course_entries(html).expect_err("first course should not steal next course title");
        assert!(matches!(err, ScraperError::ParseError(_)));
    }

    #[test]
    fn rejects_non_allowlisted_playback_url() {
        let html = r#"
<div data-course-id="42" data-course-title="Algoritma"></div>
<a class="playback-link" href="http://evil.local/recording/steal">X</a>
"#;
        let err = parse_playback_entries(html).expect_err("unsafe url should fail");
        assert!(matches!(err, ScraperError::UnsupportedFormat(_)));
    }

    #[test]
    fn parses_uppercase_anchor_tags_for_playback() {
        let html = r#"
<div data-course-id="42" data-course-title="Algoritma"></div>
<A class="playback-link" href="https://eu.bbcollab.com/recording/ok">KAYIT</A>
"#;
        let playbacks = parse_playback_entries(html).expect("uppercase anchor should parse");
        assert_eq!(playbacks.len(), 1);
        assert_eq!(playbacks[0].url, "https://eu.bbcollab.com/recording/ok");
    }

    #[test]
    fn handles_gt_inside_quoted_attributes() {
        let html = r#"
<div data-course-id="42" data-course-title="Algoritma"></div>
<a class="playback-link" title="A > B" href="https://eu.bbcollab.com/recording/ok">Kayit</a>
"#;
        let playbacks = parse_playback_entries(html).expect("quoted '>' should not break parsing");
        assert_eq!(playbacks.len(), 1);
    }

    #[test]
    fn uses_anchor_text_when_data_label_missing() {
        let html = r#"
<div data-course-id="42" data-course-title="Algoritma"></div>
<a class="playback-link" href="https://eu.bbcollab.com/recording/ok"> Kayit Ac </a>
"#;
        let playbacks = parse_playback_entries(html).expect("playback parse should succeed");
        assert_eq!(playbacks[0].label.as_deref(), Some("Kayit Ac"));
    }
}
