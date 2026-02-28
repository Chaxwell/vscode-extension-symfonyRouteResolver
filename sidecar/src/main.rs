mod php_finder;
mod protocol;
mod router;
mod text_search;

use std::collections::HashMap;
use std::io::{self, BufRead, Write};

use protocol::{Request, Response, RouteInfo};
use text_search::RouteSearcher;

fn main() {
    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut out = stdout.lock();

    let mut route_index: HashMap<String, RouteInfo> = HashMap::new();
    // The searcher is rebuilt whenever the index changes.
    let mut searcher: Option<RouteSearcher> = None;

    for line in stdin.lock().lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => break,
        };

        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        let request: Request = match serde_json::from_str(line) {
            Ok(r) => r,
            Err(e) => {
                let response = Response::Error { id: 0, message: format!("Parse error: {e}") };
                emit(&mut out, &response);
                continue;
            }
        };

        let response = handle_request(request, &mut route_index, &mut searcher);
        emit(&mut out, &response);
    }
}

fn handle_request(
    request: Request,
    route_index: &mut HashMap<String, RouteInfo>,
    searcher: &mut Option<RouteSearcher>,
) -> Response {
    match request {
        Request::Index { id, workspace_path, symfony_binary } => {
            match router::build_index(&workspace_path, &symfony_binary) {
                Ok(index) => {
                    let count = index.len();
                    *route_index = index;
                    // Rebuild the AhoCorasick automaton with the new patterns.
                    *searcher = RouteSearcher::new(route_index);
                    Response::Indexed { id, count }
                }
                Err(message) => Response::Error { id, message },
            }
        }

        Request::Search { id, content } => {
            let Some(s) = searcher else {
                return Response::Error {
                    id,
                    message: "Route index is empty — run indexation first.".into(),
                };
            };
            let matches = s.search(&content);
            Response::Searched { id, matches }
        }
    }
}

fn emit(out: &mut impl Write, response: &Response) {
    if let Ok(json) = serde_json::to_string(response) {
        let _ = writeln!(out, "{json}");
        let _ = out.flush();
    }
}
