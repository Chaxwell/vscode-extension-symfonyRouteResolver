use std::collections::HashMap;

use aho_corasick::{AhoCorasick, MatchKind};

use crate::protocol::{LineCol, MatchPosition, RouteInfo, TextMatch};

/// Pre-compiled search engine that can be reused across multiple documents.
pub struct RouteSearcher {
    ac: AhoCorasick,
    /// Route names in the same order as the patterns passed to AhoCorasick.
    patterns: Vec<String>,
    route_info: HashMap<String, RouteInfo>,
}

impl RouteSearcher {
    /// Builds the AhoCorasick automaton from the current route index.
    /// Returns `None` when the index is empty.
    pub fn new(route_index: &HashMap<String, RouteInfo>) -> Option<Self> {
        if route_index.is_empty() {
            return None;
        }

        let patterns: Vec<String> = route_index.keys().cloned().collect();
        let ac = AhoCorasick::builder()
            .match_kind(MatchKind::LeftmostLongest)
            .build(&patterns)
            .ok()?;

        Some(Self {
            ac,
            patterns,
            route_info: route_index.clone(),
        })
    }

    /// Finds every occurrence of any known route name inside `content` and
    /// returns one [`TextMatch`] per occurrence (same route may appear many
    /// times).
    pub fn search(&self, content: &str) -> Vec<TextMatch> {
        let line_starts = compute_line_starts(content);
        let mut matches = Vec::new();

        let bytes = content.as_bytes();

        for mat in self.ac.find_iter(content) {
            let route_name = &self.patterns[mat.pattern().as_usize()];

            let info = match self.route_info.get(route_name) {
                Some(i) => i,
                None => continue,
            };

            let start_offset = mat.start();
            let end_offset = mat.end();

            // The route name must be surrounded by matching single or double quotes.
            if !is_quote_bounded(bytes, start_offset, end_offset) {
                continue;
            }

            let start_line = line_for_offset(&line_starts, start_offset);
            let start_col = start_offset - line_starts[start_line];

            let end_line = line_for_offset(&line_starts, end_offset);
            let end_col = end_offset - line_starts[end_line];

            matches.push(TextMatch {
                route: route_name.clone(),
                position: MatchPosition {
                    start: LineCol {
                        line: start_line as u32,
                        col: start_col as u32,
                    },
                    end: LineCol {
                        line: end_line as u32,
                        col: end_col as u32,
                    },
                },
                file: info.clone(),
            });
        }

        matches
    }
}

// ---------------------------------------------------------------------------
// Private helpers
// ---------------------------------------------------------------------------

/// Returns `true` when the byte slice around `[start..end]` is bounded by
/// matching single (`'`) or double (`"`) quotes:
///   `'route_name'`  or  `"route_name"`
///
/// Mixed quotes (`'route_name"`) are rejected.
fn is_quote_bounded(bytes: &[u8], start: usize, end: usize) -> bool {
    if start == 0 || end >= bytes.len() {
        return false;
    }
    let before = bytes[start - 1];
    let after = bytes[end];
    matches!(
        (before, after),
        (b'\'', b'\'') | (b'"', b'"')
    )
}

/// Returns the byte offset of the start of each line (0-indexed).
fn compute_line_starts(text: &str) -> Vec<usize> {
    let mut starts = vec![0usize];
    for (i, b) in text.bytes().enumerate() {
        if b == b'\n' {
            starts.push(i + 1);
        }
    }
    starts
}

/// Binary-search for the (0-indexed) line that contains `offset`.
fn line_for_offset(line_starts: &[usize], offset: usize) -> usize {
    match line_starts.binary_search(&offset) {
        Ok(line) => line,
        Err(next) => next.saturating_sub(1),
    }
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn make_index(routes: &[(&str, &str, u32)]) -> HashMap<String, RouteInfo> {
        routes
            .iter()
            .map(|(name, file, line)| {
                (
                    name.to_string(),
                    RouteInfo { file: file.to_string(), line: *line },
                )
            })
            .collect()
    }

    #[test]
    fn test_finds_route_in_single_line() {
        let index = make_index(&[("app_home_index", "/src/Controller/HomeController.php", 10)]);
        let searcher = RouteSearcher::new(&index).unwrap();

        let content = "{{ path('app_home_index') }}";
        let matches = searcher.search(content);

        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].route, "app_home_index");
        // Content: {{ path('app_home_index') }}
        //          0123456789^ offset 9 = 'a'
        assert_eq!(matches[0].position.start.line, 0);
        assert_eq!(matches[0].position.start.col, 9); // offset of 'a' in 'app_home_index'
        assert_eq!(matches[0].position.end.col, 23);  // 9 + len("app_home_index") = 23
    }

    #[test]
    fn test_finds_multiple_occurrences() {
        let index = make_index(&[("my_route", "/src/Controller/Foo.php", 5)]);
        let searcher = RouteSearcher::new(&index).unwrap();

        let content = "'my_route' and then 'my_route' again";
        let matches = searcher.search(content);

        assert_eq!(matches.len(), 2);
    }

    #[test]
    fn test_finds_route_on_second_line() {
        let index = make_index(&[("app_about", "/src/Controller/AboutController.php", 20)]);
        let searcher = RouteSearcher::new(&index).unwrap();

        let content = "first line\n'app_about' here";
        let matches = searcher.search(content);

        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].position.start.line, 1);
        assert_eq!(matches[0].position.start.col, 1); // col 0 is the quote, route starts at 1
    }

    #[test]
    fn test_no_match_returns_empty() {
        let index = make_index(&[("app_foo", "/src/Controller/FooController.php", 1)]);
        let searcher = RouteSearcher::new(&index).unwrap();

        let matches = searcher.search("nothing interesting here");
        assert!(matches.is_empty());
    }

    #[test]
    fn test_no_match_without_quotes() {
        // Route names not surrounded by quotes must not be matched.
        let index = make_index(&[("app_foo", "/src/Controller/FooController.php", 1)]);
        let searcher = RouteSearcher::new(&index).unwrap();

        // No quotes → no match.
        assert!(searcher.search("app_foo").is_empty());
        // Mixed quotes → no match.
        assert!(searcher.search("'app_foo\"").is_empty());
        // Part of a longer unquoted string → no match.
        assert!(searcher.search("app_foo_translation.key").is_empty());
    }

    #[test]
    fn test_no_match_for_prefix_inside_longer_unquoted_string() {
        // 'pdp_add' must NOT match inside "pdp_add_something.translation" (no quotes).
        let index = make_index(&[
            ("pdp_add", "/src/Controller/PdpController.php", 10),
        ]);
        let searcher = RouteSearcher::new(&index).unwrap();

        assert!(searcher.search("pdp_add_something.translation").is_empty());
    }

    #[test]
    fn test_empty_index_returns_none() {
        let index = HashMap::new();
        assert!(RouteSearcher::new(&index).is_none());
    }

    #[test]
    fn test_file_info_is_propagated() {
        let index = make_index(&[("route_x", "/path/to/Controller.php", 42)]);
        let searcher = RouteSearcher::new(&index).unwrap();

        let matches = searcher.search("'route_x'");
        assert_eq!(matches[0].file.file, "/path/to/Controller.php");
        assert_eq!(matches[0].file.line, 42);
    }

    #[test]
    fn test_double_quotes_also_match() {
        let index = make_index(&[("app_home", "/src/Controller/HomeController.php", 5)]);
        let searcher = RouteSearcher::new(&index).unwrap();

        let matches = searcher.search(r#"path("app_home")"#);
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].route, "app_home");
    }

    #[test]
    fn test_prefers_longer_match_over_prefix() {
        // "pdp_add" is a prefix of "pdp_add_tiers" — only the longer one should match.
        let index = make_index(&[
            ("pdp_add", "/src/Controller/PdpController.php", 10),
            ("pdp_add_tiers", "/src/Controller/PdpController.php", 20),
        ]);
        let searcher = RouteSearcher::new(&index).unwrap();

        let matches = searcher.search("route: 'pdp_add_tiers'");
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].route, "pdp_add_tiers");
    }

    #[test]
    fn test_matches_prefix_when_no_longer_route_present() {
        // "pdp_add" alone should still match when "pdp_add_tiers" is not in the text.
        let index = make_index(&[
            ("pdp_add", "/src/Controller/PdpController.php", 10),
            ("pdp_add_tiers", "/src/Controller/PdpController.php", 20),
        ]);
        let searcher = RouteSearcher::new(&index).unwrap();

        let matches = searcher.search("route: 'pdp_add' and something");
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].route, "pdp_add");
    }

    #[test]
    fn test_matches_both_when_both_present() {
        let index = make_index(&[
            ("pdp_add", "/src/Controller/PdpController.php", 10),
            ("pdp_add_tiers", "/src/Controller/PdpController.php", 20),
        ]);
        let searcher = RouteSearcher::new(&index).unwrap();

        let matches = searcher.search("'pdp_add_tiers' and 'pdp_add' here");
        assert_eq!(matches.len(), 2);
        assert!(matches.iter().any(|m| m.route == "pdp_add_tiers"));
        assert!(matches.iter().any(|m| m.route == "pdp_add"));
    }

    #[test]
    fn test_compute_line_starts() {
        let starts = compute_line_starts("ab\ncd\nef");
        assert_eq!(starts, vec![0, 3, 6]);
    }

    #[test]
    fn test_line_for_offset() {
        let starts = vec![0, 3, 6];
        assert_eq!(line_for_offset(&starts, 0), 0);
        assert_eq!(line_for_offset(&starts, 2), 0);
        assert_eq!(line_for_offset(&starts, 3), 1);
        assert_eq!(line_for_offset(&starts, 5), 1);
        assert_eq!(line_for_offset(&starts, 6), 2);
        assert_eq!(line_for_offset(&starts, 8), 2);
    }
}
