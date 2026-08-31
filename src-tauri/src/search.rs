#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SearchFilter {
    pub q: String,
    pub title: Option<String>,
    pub created_from: Option<String>,
    pub created_to: Option<String>,
    pub tag: Option<String>,
}

pub fn normalize_tag(raw: &str) -> Option<String> {
    let t = raw.trim().to_ascii_lowercase();
    if t.is_empty() {
        None
    } else {
        Some(t)
    }
}

pub fn normalize_tags(tags: &[String]) -> Vec<String> {
    let mut out: Vec<String> = tags.iter().filter_map(|t| normalize_tag(t)).collect();
    out.sort();
    out.dedup();
    out
}
