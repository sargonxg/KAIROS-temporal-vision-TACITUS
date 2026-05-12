use crate::source::SourceSpan;
use chrono::{DateTime, Datelike, Duration, NaiveDate, TimeZone, Utc};
use regex::Regex;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DateMention {
    pub text: String,
    pub char_start: usize,
    pub char_end: usize,
    pub resolved: Option<DateTime<Utc>>,
    pub fuzziness_secs: u32,
    pub kind: DateMentionKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_span: Option<SourceSpan>,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DateMentionKind {
    Absolute,
    YearOnly,
    MonthYear,
    Iso,
    Relative,
    Duration,
}

pub struct DateExtractor {
    rx_full: Regex,
    rx_month_year: Regex,
    rx_year: Regex,
    rx_iso: Regex,
    rx_relative: Regex,
    rx_within_days: Regex,
}

impl Default for DateExtractor {
    fn default() -> Self {
        Self::new()
    }
}

impl DateExtractor {
    pub fn new() -> Self {
        let month = r"(January|February|March|April|May|June|July|August|September|October|November|December|Jan|Feb|Mar|Apr|Jun|Jul|Aug|Sep|Sept|Oct|Nov|Dec)";
        Self {
            rx_full: Regex::new(&format!(r"(?i)\b{}\s+(\d{{1,2}}),?\s+(\d{{4}})\b", month))
                .unwrap(),
            rx_month_year: Regex::new(&format!(r"(?i)\b{}\s+(\d{{4}})\b", month)).unwrap(),
            rx_year: Regex::new(r"(?i)\b(?:in|by|since|during|until)\s+(20\d{2}|19\d{2})\b")
                .unwrap(),
            rx_iso: Regex::new(r"\b(\d{4})-(\d{2})-(\d{2})\b").unwrap(),
            rx_relative: Regex::new(
                r"(?i)\b(yesterday|tomorrow|last week|next week|next quarter|two days later|two days before|the following morning)\b",
            )
            .unwrap(),
            rx_within_days: Regex::new(r"(?i)\bwithin\s+(\d{1,3})\s+days?\b").unwrap(),
        }
    }

    pub fn extract(&self, text: &str) -> Vec<DateMention> {
        self.extract_with_context(text, None, None)
    }

    pub fn extract_with_context(
        &self,
        text: &str,
        doc_id: Option<&str>,
        document_created_at: Option<DateTime<Utc>>,
    ) -> Vec<DateMention> {
        let mut out = Vec::new();
        let mut taken: Vec<(usize, usize)> = Vec::new();

        for cap in self.rx_iso.captures_iter(text) {
            let m = cap.get(0).unwrap();
            if overlaps(m.start(), m.end(), &taken) {
                continue;
            }
            let resolved = ymd(&cap[1], &cap[2], &cap[3]);
            out.push(DateMention {
                text: m.as_str().to_string(),
                char_start: m.start(),
                char_end: m.end(),
                resolved,
                fuzziness_secs: 0,
                kind: DateMentionKind::Iso,
                source_span: Some(SourceSpan::new(doc_id, m.start(), m.end(), text)),
            });
            taken.push((m.start(), m.end()));
        }

        for cap in self.rx_full.captures_iter(text) {
            let m = cap.get(0).unwrap();
            if overlaps(m.start(), m.end(), &taken) {
                continue;
            }
            let resolved = month_day_year(&cap[1], &cap[2], &cap[3]);
            out.push(DateMention {
                text: m.as_str().to_string(),
                char_start: m.start(),
                char_end: m.end(),
                resolved,
                fuzziness_secs: 0,
                kind: DateMentionKind::Absolute,
                source_span: Some(SourceSpan::new(doc_id, m.start(), m.end(), text)),
            });
            taken.push((m.start(), m.end()));
        }

        for cap in self.rx_month_year.captures_iter(text) {
            let m = cap.get(0).unwrap();
            if overlaps(m.start(), m.end(), &taken) {
                continue;
            }
            let resolved = month_year(&cap[1], &cap[2]);
            out.push(DateMention {
                text: m.as_str().to_string(),
                char_start: m.start(),
                char_end: m.end(),
                resolved,
                fuzziness_secs: 60 * 60 * 24 * 31,
                kind: DateMentionKind::MonthYear,
                source_span: Some(SourceSpan::new(doc_id, m.start(), m.end(), text)),
            });
            taken.push((m.start(), m.end()));
        }

        for cap in self.rx_year.captures_iter(text) {
            let m = cap.get(1).unwrap();
            if overlaps(m.start(), m.end(), &taken) {
                continue;
            }
            let resolved = ymd(m.as_str(), "01", "01");
            out.push(DateMention {
                text: m.as_str().to_string(),
                char_start: m.start(),
                char_end: m.end(),
                resolved,
                fuzziness_secs: 60 * 60 * 24 * 365,
                kind: DateMentionKind::YearOnly,
                source_span: Some(SourceSpan::new(doc_id, m.start(), m.end(), text)),
            });
            taken.push((m.start(), m.end()));
        }

        for cap in self.rx_relative.captures_iter(text) {
            let m = cap.get(0).unwrap();
            if overlaps(m.start(), m.end(), &taken) {
                continue;
            }
            let resolved = document_created_at.and_then(|dct| resolve_relative(m.as_str(), dct));
            out.push(DateMention {
                text: m.as_str().to_string(),
                char_start: m.start(),
                char_end: m.end(),
                resolved,
                fuzziness_secs: 60 * 60 * 24,
                kind: DateMentionKind::Relative,
                source_span: Some(SourceSpan::new(doc_id, m.start(), m.end(), text)),
            });
            taken.push((m.start(), m.end()));
        }

        for cap in self.rx_within_days.captures_iter(text) {
            let m = cap.get(0).unwrap();
            if overlaps(m.start(), m.end(), &taken) {
                continue;
            }
            let days = cap
                .get(1)
                .and_then(|n| n.as_str().parse::<i64>().ok())
                .unwrap_or(0);
            let resolved = document_created_at.map(|dct| dct + Duration::days(days));
            out.push(DateMention {
                text: m.as_str().to_string(),
                char_start: m.start(),
                char_end: m.end(),
                resolved,
                fuzziness_secs: (60 * 60 * 24 * days.max(1)) as u32,
                kind: DateMentionKind::Duration,
                source_span: Some(SourceSpan::new(doc_id, m.start(), m.end(), text)),
            });
            taken.push((m.start(), m.end()));
        }

        out.sort_by_key(|d| d.char_start);
        resolve_contextual_relatives(&mut out, document_created_at);
        out
    }
}

fn resolve_contextual_relatives(
    mentions: &mut [DateMention],
    document_created_at: Option<DateTime<Utc>>,
) {
    let mut previous_resolved = document_created_at;
    for mention in mentions {
        if mention.resolved.is_none() && mention.kind == DateMentionKind::Relative {
            mention.resolved =
                previous_resolved.and_then(|anchor| resolve_relative(&mention.text, anchor));
        }
        if mention.resolved.is_some() && !matches!(mention.kind, DateMentionKind::Duration) {
            previous_resolved = mention.resolved;
        }
    }
}

fn resolve_relative(text: &str, dct: DateTime<Utc>) -> Option<DateTime<Utc>> {
    let day = match text.to_lowercase().as_str() {
        "yesterday" => dct - Duration::days(1),
        "tomorrow" => dct + Duration::days(1),
        "last week" => dct - Duration::days(7),
        "next week" => dct + Duration::days(7),
        "two days later" => dct + Duration::days(2),
        "two days before" => dct - Duration::days(2),
        "the following morning" => dct + Duration::days(1),
        "next quarter" => return next_quarter_start(dct),
        _ => return None,
    };
    Some(Utc.from_utc_datetime(&day.date_naive().and_hms_opt(0, 0, 0)?))
}

fn next_quarter_start(dct: DateTime<Utc>) -> Option<DateTime<Utc>> {
    let month = dct.month();
    let next = match month {
        1..=3 => (dct.year(), 4),
        4..=6 => (dct.year(), 7),
        7..=9 => (dct.year(), 10),
        _ => (dct.year() + 1, 1),
    };
    Some(Utc.from_utc_datetime(&NaiveDate::from_ymd_opt(next.0, next.1, 1)?.and_hms_opt(0, 0, 0)?))
}

fn overlaps(s: usize, e: usize, taken: &[(usize, usize)]) -> bool {
    taken.iter().any(|(ts, te)| !(e <= *ts || s >= *te))
}

fn ymd(y: &str, m: &str, d: &str) -> Option<DateTime<Utc>> {
    let y = y.parse::<i32>().ok()?;
    let m = m.parse::<u32>().ok()?;
    let d = d.parse::<u32>().ok()?;
    Some(Utc.from_utc_datetime(&NaiveDate::from_ymd_opt(y, m, d)?.and_hms_opt(0, 0, 0)?))
}

fn month_day_year(month: &str, day: &str, year: &str) -> Option<DateTime<Utc>> {
    ymd(year, &month_number(month)?.to_string(), day)
}

fn month_year(month: &str, year: &str) -> Option<DateTime<Utc>> {
    ymd(year, &month_number(month)?.to_string(), "01")
}

fn month_number(month: &str) -> Option<u32> {
    match month.to_lowercase().as_str() {
        "january" | "jan" => Some(1),
        "february" | "feb" => Some(2),
        "march" | "mar" => Some(3),
        "april" | "apr" => Some(4),
        "may" => Some(5),
        "june" | "jun" => Some(6),
        "july" | "jul" => Some(7),
        "august" | "aug" => Some(8),
        "september" | "sep" | "sept" => Some(9),
        "october" | "oct" => Some(10),
        "november" | "nov" => Some(11),
        "december" | "dec" => Some(12),
        _ => None,
    }
}
