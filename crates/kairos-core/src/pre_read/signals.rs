use crate::pre_read::TensionMarker;
use regex::Regex;

pub fn detect_tension_markers(text: &str) -> Vec<TensionMarker> {
    let patterns = [
        (
            "delay",
            r"(?i)\b(delay|delayed|late|missed|postponed|cure period|slippage)\b",
            0.72,
        ),
        (
            "breach",
            r"(?i)\b(breach|violat(?:e|ed|ion)|failed to|noncompliance|default)\b",
            0.86,
        ),
        (
            "refusal",
            r"(?i)\b(refus(?:e|ed|al)|reject(?:ed|s)?|declined|will not)\b",
            0.78,
        ),
        (
            "ambiguity",
            r"(?i)\b(uncertain|ambiguous|unresolved|unclear|disputed explanation)\b",
            0.62,
        ),
        (
            "reversal",
            r"(?i)\b(reversal|reversed|rescind(?:ed)?|freeze|froze|suspend(?:ed)?)\b",
            0.7,
        ),
        (
            "threat",
            r"(?i)\b(threaten(?:ed|s)?|ultimatum|warn(?:ed|s)?|escalat(?:e|ed|ion))\b",
            0.8,
        ),
        (
            "procedural_block",
            r"(?i)\b(court blocks?|injunction|procedural|quorum|veto|blocked|obstruction)\b",
            0.82,
        ),
        (
            "trust_loss",
            r"(?i)\b(trust|bad faith|premature|collateral damage|confidence collapsed)\b",
            0.76,
        ),
    ];

    let mut out = Vec::new();
    for (marker_type, pattern, weight) in patterns {
        let rx = Regex::new(pattern).unwrap();
        for m in rx.find_iter(text) {
            out.push(TensionMarker {
                id: format!("tm_{:04}", out.len() + 1),
                marker_type: marker_type.to_string(),
                phrase: m.as_str().to_string(),
                char_start: m.start(),
                char_end: m.end(),
                weight,
            });
        }
    }
    out.sort_by_key(|marker| marker.char_start);
    out
}
