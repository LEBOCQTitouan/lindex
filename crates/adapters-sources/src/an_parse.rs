//! Parse the Assemblée nationale `dyn` scrutin page into the source-agnostic
//! [`RawScrutinData`]. All AN-specific HTML knowledge lives here; nothing about
//! the AN markup leaks past this module (hexagon boundary).
//!
//! There is no public per-scrutin JSON endpoint (verified 2026-08-10), so we
//! extract from the analysis page. The markup is stable: top-line counts sit in
//! `<b>` tags, and each political group is a `data-organe-id="PO…"` block whose
//! `FocusableList` carries the per-position counts.

use lindex_application::ports::{RawGroup, RawScrutinData, RawTotals};
use lindex_domain::ScrutinId;
use regex::Regex;

/// The AN has 577 seats — the denominator for the participation gap.
const AN_SEATS: u32 = 577;

/// AN political groups: `organe_id → (short id, hémicycle rank left→right)`.
/// Reference data for the 17th legislature; drives P5 symmetry (one ordering).
const GROUPS: &[(&str, &str, u8)] = &[
    ("PO845413", "LFI", 0),
    ("PO845514", "GDR", 1),
    ("PO845439", "ECOS", 2),
    ("PO845419", "SOC", 3),
    ("PO845485", "LIOT", 4),
    ("PO845454", "DEM", 5),
    ("PO845407", "EPR", 6),
    ("PO845470", "HOR", 7),
    ("PO845425", "DR", 8),
    ("PO872880", "UDR", 9),
    ("PO845401", "RN", 10),
    ("PO840056", "NI", 11),
];

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum ParseError {
    #[error("missing field in AN page: {0}")]
    Missing(&'static str),
    #[error("unknown group organe id: {0}")]
    UnknownGroup(String),
    #[error("unparseable sitting date: {0}")]
    BadDate(String),
    #[error("inconsistent top-line figures: {0}")]
    Inconsistent(String),
    #[error("internal parser error: {0}")]
    Internal(String),
}

fn re(pattern: &str) -> Result<Regex, ParseError> {
    Regex::new(pattern).map_err(|e| ParseError::Internal(e.to_string()))
}

/// Grab a top-line count rendered as `Label … : <b>NN</b>`.
fn bold_count(html: &str, label_pattern: &str, field: &'static str) -> Result<u32, ParseError> {
    let rx = re(&format!(r"{label_pattern}\s*:\s*<b>(\d+)</b>"))?;
    rx.captures(html)
        .and_then(|c| c.get(1))
        .and_then(|m| m.as_str().parse().ok())
        .ok_or(ParseError::Missing(field))
}

/// Decode the handful of HTML entities the AN page uses in text fields.
fn unescape(s: &str) -> String {
    s.replace("&#039;", "'")
        .replace("&#39;", "'")
        .replace("&amp;", "&")
        .replace("&quot;", "\"")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&nbsp;", " ")
}

fn parse_date(html: &str) -> Result<String, ParseError> {
    let rx = re(r"séance du\s+(?:\p{L}+\s+)?(\d{1,2})\s+(\p{L}+)\s+(\d{4})")?;
    let caps = rx
        .captures(html)
        .ok_or(ParseError::Missing("séance date"))?;
    let day: u32 = caps[1]
        .parse()
        .map_err(|_| ParseError::BadDate(caps[1].to_string()))?;
    let month = match caps[2].to_lowercase().as_str() {
        "janvier" => 1,
        "février" | "fevrier" => 2,
        "mars" => 3,
        "avril" => 4,
        "mai" => 5,
        "juin" => 6,
        "juillet" => 7,
        "août" | "aout" => 8,
        "septembre" => 9,
        "octobre" => 10,
        "novembre" => 11,
        "décembre" | "decembre" => 12,
        other => return Err(ParseError::BadDate(other.to_string())),
    };
    let year: i32 = caps[3]
        .parse()
        .map_err(|_| ParseError::BadDate(caps[3].to_string()))?;
    // Zero-padded ISO; the domain re-parses and would reject an impossible date.
    Ok(format!("{year:04}-{month:02}-{day:02}"))
}

fn parse_outcome(html: &str) -> Result<String, ParseError> {
    // The page states the result authoritatively; we never compute it (P3).
    // Anchor to the single bold result span (`…_bold">L'Assemblée nationale …`)
    // rather than scanning the whole document — a rejected amendment or a
    // "scrutins liés" mention elsewhere must not flip the result. The apostrophe
    // is HTML-encoded on the page (`&#039;`) but tolerate plain forms too.
    let apos = r"(?:&#0?39;|['’])";
    let rx = re(&format!(
        r#"_bold">L{apos}Assemblée nationale (n{apos}a pas adopté|a adopté|a rejeté)"#
    ))?;
    match rx.captures(html).and_then(|c| c.get(1)).map(|m| m.as_str()) {
        Some("a adopté") => Ok("adopte".to_string()),
        Some(_) => Ok("rejete".to_string()), // "n'a pas adopté" | "a rejeté"
        None => Err(ParseError::Missing("outcome sentence")),
    }
}

/// Parse the per-group breakdown, returned in hémicycle order.
fn parse_groups(html: &str) -> Result<Vec<RawGroup>, ParseError> {
    let list_rx = re(r#"(?s)FocusableList">(.*?)</ul>"#)?;
    let span_rx = re(r#"_small">\s*([^:<]+?)\s*:\s*(\d+)"#)?;
    let name_rx = re(r#"(?s)href="/dyn/org/PO\d+"[^>]*>\s*([^<]+?)\s*</a>"#)?;

    let mut found: Vec<(u8, RawGroup)> = Vec::new();
    // Each `data-organe-id="PO…"` starts one group block; the block runs to the
    // next marker. The page lists each group twice (a legend with no counts and
    // the detailed decompte) — we keep only blocks that carry position counts.
    for part in html.split("data-organe-id=\"").skip(1) {
        let Some(end) = part.find('"') else { continue };
        let organe = &part[..end];
        if !organe.starts_with("PO") {
            continue;
        }
        let Some(block) = list_rx.captures(part).and_then(|c| c.get(1)) else {
            continue;
        };
        let block = block.as_str();
        let mut pour = 0;
        let mut contre = 0;
        let mut abstention = 0;
        let mut non_votant = 0;
        let mut any = false;
        for cap in span_rx.captures_iter(block) {
            let n: u32 = cap[2].parse().unwrap_or(0);
            match cap[1].trim() {
                "Pour" | "Pour l'adoption" => pour = n,
                "Contre" => contre = n,
                "Abstention" => abstention = n,
                "Non votant" | "Non votants" => non_votant = n,
                _ => continue,
            }
            any = true;
        }
        if !any {
            continue; // legend block, no counts
        }
        let (short, rank) = GROUPS
            .iter()
            .find(|(oid, _, _)| *oid == organe)
            .map(|(_, s, r)| (*s, *r))
            .ok_or_else(|| {
                // include the group name for a debuggable message
                let name = name_rx
                    .captures(part)
                    .and_then(|c| c.get(1))
                    .map(|m| unescape(m.as_str()))
                    .unwrap_or_default();
                ParseError::UnknownGroup(format!("{organe} ({name})"))
            })?;
        found.push((
            rank,
            RawGroup {
                group: short.to_string(),
                pour,
                contre,
                abstention,
                non_votant,
            },
        ));
    }
    found.sort_by_key(|(rank, _)| *rank);
    Ok(found.into_iter().map(|(_, g)| g).collect())
}

/// Parse a fetched AN scrutin analysis page into the normalized record.
pub fn parse_scrutin_html(html: &str, id: &ScrutinId) -> Result<RawScrutinData, ParseError> {
    let pour = bold_count(html, r"Pour l['’]adoption", "pour")?;
    let contre = bold_count(html, r"Contre", "contre")?;
    let abstention = bold_count(html, r"Abstention", "abstention")?;
    let votants = bold_count(html, r"Nombre de votants", "votants")?;
    let exprimes = bold_count(html, r"Nombre de suffrages exprimés", "exprimes")?;

    // The declared votants must equal the decisive positions we summarize; guard
    // it so a page quirk can't ship a figure whose baseline is computed from a
    // different number than the one displayed (display rule #1 integrity).
    if pour + contre + abstention != votants {
        return Err(ParseError::Inconsistent(format!(
            "votants {votants} != pour+contre+abstention {}",
            pour + contre + abstention
        )));
    }

    let held_on = parse_date(html)?;
    let outcome = parse_outcome(html)?;
    let groups = parse_groups(html)?;
    let non_votants = groups.iter().map(|g| g.non_votant).sum();

    let title_rx = re(r#"<meta name="description" content="([^"]+)""#)?;
    let title = title_rx
        .captures(html)
        .and_then(|c| c.get(1))
        .map(|m| unescape(m.as_str()))
        .ok_or(ParseError::Missing("title"))?;

    Ok(RawScrutinData {
        scrutin_id: id.0.clone(),
        chamber: "AN".to_string(),
        title,
        held_on,
        outcome,
        totals: RawTotals {
            pour,
            contre,
            abstention,
            non_votants,
            members_total: AN_SEATS,
            votants,
            exprimes,
        },
        groups,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const HTML_8433: &str = include_str!("../tests/fixtures/scrutin_8433.html");
    const HTML_8430: &str = include_str!("../tests/fixtures/scrutin_8430.html");

    fn g<'a>(data: &'a RawScrutinData, short: &str) -> &'a RawGroup {
        data.groups
            .iter()
            .find(|g| g.group == short)
            .expect("group present")
    }

    #[test]
    fn parses_8433_totals_and_metadata() {
        let d = parse_scrutin_html(HTML_8433, &ScrutinId("8433".into())).expect("parse");
        assert_eq!(d.chamber, "AN");
        assert_eq!(d.held_on, "2026-07-21");
        assert_eq!(d.outcome, "adopte");
        assert!(
            d.title.contains("Scrutin public n°8433"),
            "title: {}",
            d.title
        );
        assert_eq!(d.totals.pour, 351);
        assert_eq!(d.totals.contre, 179);
        assert_eq!(d.totals.abstention, 7);
        assert_eq!(d.totals.votants, 537);
        assert_eq!(d.totals.exprimes, 530);
        assert_eq!(d.totals.members_total, 577);
        assert_eq!(d.totals.non_votants, 2); // EPR 1 + HOR 1
    }

    #[test]
    fn parses_8433_breakdown_in_hemicycle_order() {
        let d = parse_scrutin_html(HTML_8433, &ScrutinId("8433".into())).expect("parse");
        assert_eq!(d.groups.len(), 12);
        // Left→right hémicycle order (P5).
        let order: Vec<&str> = d.groups.iter().map(|g| g.group.as_str()).collect();
        assert_eq!(
            order,
            ["LFI", "GDR", "ECOS", "SOC", "LIOT", "DEM", "EPR", "HOR", "DR", "UDR", "RN", "NI"]
        );
        let lfi = g(&d, "LFI");
        assert_eq!(
            (lfi.pour, lfi.contre, lfi.abstention, lfi.non_votant),
            (0, 71, 0, 0)
        );
        let epr = g(&d, "EPR");
        assert_eq!(
            (epr.pour, epr.contre, epr.abstention, epr.non_votant),
            (84, 0, 0, 1)
        );
        let ni = g(&d, "NI");
        assert_eq!(
            (ni.pour, ni.contre, ni.abstention, ni.non_votant),
            (7, 0, 1, 0)
        );
        // The breakdown reconciles with the declared totals.
        assert_eq!(d.groups.iter().map(|g| g.pour).sum::<u32>(), d.totals.pour);
        assert_eq!(
            d.groups.iter().map(|g| g.contre).sum::<u32>(),
            d.totals.contre
        );
        assert_eq!(
            d.groups.iter().map(|g| g.abstention).sum::<u32>(),
            d.totals.abstention
        );
    }

    #[test]
    fn parses_8430_high_abstention_scrutin() {
        let d = parse_scrutin_html(HTML_8430, &ScrutinId("8430".into())).expect("parse");
        assert_eq!(d.held_on, "2026-07-21");
        assert_eq!(d.outcome, "adopte");
        assert_eq!(d.totals.pour, 378);
        assert_eq!(d.totals.contre, 7);
        assert_eq!(d.totals.abstention, 173);
        assert_eq!(d.totals.votants, 558);
        assert_eq!(d.totals.non_votants, 1); // EPR 1
        let lfi = g(&d, "LFI");
        assert_eq!(
            (lfi.pour, lfi.contre, lfi.abstention, lfi.non_votant),
            (0, 0, 69, 0)
        );
        let gdr = g(&d, "GDR");
        assert_eq!(
            (gdr.pour, gdr.contre, gdr.abstention, gdr.non_votant),
            (5, 7, 5, 0)
        );
        let epr = g(&d, "EPR");
        assert_eq!(
            (epr.pour, epr.contre, epr.abstention, epr.non_votant),
            (86, 0, 2, 1)
        );
    }
}
