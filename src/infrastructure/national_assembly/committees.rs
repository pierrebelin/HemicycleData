pub fn resolve_committee(organe_ref: &str) -> Option<&'static str> {
    match organe_ref {
        // 17e legislature (= 16e)
        "PO59051" => Some("Commission des lois"),
        "PO420120" => Some("Commission des affaires sociales"),
        "PO419610" => Some("Commission des affaires \u{00e9}conomiques"),
        "PO419604" => Some("Commission des affaires culturelles et de l'\u{00e9}ducation"),
        "PO419865" => Some("Commission du d\u{00e9}veloppement durable"),
        "PO59048" => Some("Commission des finances"),
        "PO59047" => Some("Commission des affaires \u{00e9}trang\u{00e8}res"),
        "PO59046" => Some("Commission de la d\u{00e9}fense"),
        // 15e legislature
        "PO211493" => Some("Commission des lois"),
        "PO211495" => Some("Commission des affaires sociales"),
        "PO211494" => Some("Commission des affaires \u{00e9}conomiques"),
        "PO211490" => Some("Commission des affaires culturelles et de l'\u{00e9}ducation"),
        "PO211491" => Some("Commission des affaires \u{00e9}trang\u{00e8}res"),
        // 14e legislature
        "PO516753" => Some("Commission des affaires sociales"),
        "PO516754" => Some("Commission du d\u{00e9}veloppement durable"),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_known_commission() {
        assert_eq!(resolve_committee("PO59051"), Some("Commission des lois"));
    }

    #[test]
    fn returns_none_for_unknown() {
        assert_eq!(resolve_committee("PO999999"), None);
    }

    #[test]
    fn resolves_older_legislature() {
        assert_eq!(resolve_committee("PO211493"), Some("Commission des lois"));
    }
}
