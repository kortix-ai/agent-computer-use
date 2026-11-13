/// Resolves an application name against enumerated Windows processes.
///
/// Exact executable-name matches take priority. Display-name fallback
/// is deliberately conservative, and ambiguous process candidates are rejected.
pub fn find_process_id<F>(
    requested_name: &str,
    processes: &[(u32, String)],
    mut has_visible_window: F,
) -> Option<u32>
where
    F: FnMut(u32) -> bool,
{
    let requested = normalize(requested_name);

    if requested.is_empty() {
        return None;
    }

    // Prefer exact executable-name matches.
    let exact_matches: Vec<u32> = processes
        .iter()
        .filter(|(_, name)| normalize(name) == requested)
        .map(|(pid, _)| *pid)
        .collect();

    if !exact_matches.is_empty() {
        return select_candidate(&exact_matches, &mut has_visible_window);
    }

    // Conservative fallback: match the final token of a display name.
    let final_token = requested.split_whitespace().last()?;

    let fallback_matches: Vec<u32> = processes
        .iter()
        .filter(|(_, name)| normalize(name) == final_token)
        .map(|(pid, _)| *pid)
        .collect();

    select_candidate(&fallback_matches, &mut has_visible_window)
}

fn select_candidate<F>(candidates: &[u32], has_visible_window: &mut F) -> Option<u32>
where
    F: FnMut(u32) -> bool,
{
    if candidates.is_empty() {
        return None;
    }

    let visible: Vec<u32> = candidates
        .iter()
        .copied()
        .filter(|pid| has_visible_window(*pid))
        .collect();

    match visible.as_slice() {
        [pid] => Some(*pid),
        [] if candidates.len() == 1 => Some(candidates[0]),
        _ => None,
    }
}

fn normalize(name: &str) -> String {
    let normalized = name
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase();

    normalized
        .strip_suffix(".exe")
        .unwrap_or(&normalized)
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn no_visible_window(_: u32) -> bool {
        false
    }

    #[test]
    fn matches_exact_process_name() {
        let processes = vec![(100, "chrome".into())];
        assert_eq!(
            find_process_id("chrome", &processes, no_visible_window),
            Some(100)
        );
    }

    #[test]
    fn matches_case_insensitively() {
        let processes = vec![(100, "CHROME".into())];
        assert_eq!(
            find_process_id("chrome", &processes, no_visible_window),
            Some(100)
        );
    }
    #[test]
    fn rejects_ambiguous_display_name_fallback() {
        let processes = vec![
            (100, "assistant".into()),
            (200, "assistant".into()),
        ];

        assert_eq!(
            find_process_id("Google Assistant", &processes, no_visible_window),
            None
        );
    }

    #[test]
    fn matches_display_name_using_final_token() {
        let processes = vec![(100, "chrome".into())];
        assert_eq!(
            find_process_id("Google Chrome", &processes, no_visible_window),
            Some(100)
        );
    }

    #[test]
    fn prefers_exact_match_over_display_name_fallback() {
        let processes = vec![(100, "chrome".into()), (200, "googlechrome".into())];

        assert_eq!(
            find_process_id("chrome", &processes, no_visible_window),
            Some(100)
        );
    }

    #[test]
    fn rejects_partial_process_names() {
        let processes = vec![(100, "chromium".into())];
        assert_eq!(
            find_process_id("chrome", &processes, no_visible_window),
            None
        );
    }

    #[test]
    fn handles_whitespace() {
        let processes = vec![(100, "chrome".into())];
        assert_eq!(
            find_process_id("  Google   Chrome  ", &processes, no_visible_window),
            Some(100)
        );
    }

    #[test]
    fn handles_exe_extension_case_insensitively() {
        let processes = vec![(100, "CHROME.EXE".into())];
        assert_eq!(
            find_process_id("chrome.exe", &processes, no_visible_window),
            Some(100)
        );
    }

    #[test]
    fn rejects_empty_names() {
        assert_eq!(find_process_id("  ", &[], no_visible_window), None);
    }

    #[test]
    fn returns_none_for_unmatched_names() {
        let processes = vec![(100, "chrome".into())];
        assert_eq!(
            find_process_id("Firefox", &processes, no_visible_window),
            None
        );
    }

    #[test]
    fn selects_the_only_process_with_a_visible_window() {
        let processes = vec![(100, "chrome".into()), (200, "chrome".into())];

        assert_eq!(
            find_process_id("chrome", &processes, |pid| pid == 200),
            Some(200)
        );
    }

    #[test]
    fn rejects_multiple_visible_processes() {
        let processes = vec![(100, "chrome".into()), (200, "chrome".into())];

        assert_eq!(find_process_id("chrome", &processes, |_| true), None);
    }

    #[test]
    fn rejects_multiple_processes_without_visible_windows() {
        let processes = vec![(100, "chrome".into()), (200, "chrome".into())];

        assert_eq!(
            find_process_id("chrome", &processes, no_visible_window),
            None
        );
    }

    #[test]
    fn accepts_a_single_process_without_a_visible_window() {
        let processes = vec![(100, "chrome".into())];

        assert_eq!(
            find_process_id("chrome", &processes, no_visible_window),
            Some(100)
        );
    }
}
