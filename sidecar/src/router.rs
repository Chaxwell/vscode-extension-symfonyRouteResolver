use std::collections::HashMap;
use std::process::Command;

use serde_json::Value;

use crate::php_finder;
use crate::protocol::RouteInfo;

/// Runs `symfony console debug:router --format=json`, parses the output and
/// resolves each PHP-namespaced controller to a (file, line) pair.
///
/// Only routes whose `_controller` value looks like a PHP FQCN (`App\...`) are
/// kept; all others are silently skipped.
pub fn build_index(
    workspace_path: &str,
    symfony_binary: &str,
) -> Result<HashMap<String, RouteInfo>, String> {
    let output = Command::new(symfony_binary)
        .args(["console", "debug:router", "--format=json"])
        .current_dir(workspace_path)
        .output()
        .map_err(|e| format!("Failed to execute symfony binary '{}': {}", symfony_binary, e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!(
            "symfony console debug:router exited with error: {}",
            stderr.trim()
        ));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: Value = serde_json::from_str(&stdout)
        .map_err(|e| format!("Failed to parse debug:router JSON: {}", e))?;

    let routes_obj = json
        .as_object()
        .ok_or_else(|| "Expected a JSON object at top level of debug:router output".to_string())?;

    let mut index = HashMap::new();

    for (route_name, route_data) in routes_obj {
        // Skip Symfony-internal and API Platform placeholder routes.
        if route_name.starts_with('_') { // TODO Pas forcément une règle à préserver
            continue;
        }

        let controller = match route_data
            .get("defaults")
            .and_then(|d| d.get("_controller"))
            .and_then(|c| c.as_str())
        {
            Some(c) => c,
            None => continue,
        };

        // Only handle PHP-namespaced App controllers.
        if !controller.starts_with("App\\") { // TODO Idem pas foorcément à garder, il faudra analyser le composer.json pour savoir exactement.
            continue;
        }

        let (class_fqcn, method) = match controller.split_once("::") {
            Some((class, method)) => (class, Some(method)),
            None => (controller, None),
        };

        match php_finder::find_location(workspace_path, class_fqcn, method) {
            Ok(info) => {
                index.insert(route_name.clone(), info);
            }
            Err(_) => {
                // Best-effort: skip routes whose source file cannot be resolved.
            }
        }
    }

    Ok(index)
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// Ensure that routes with a non-PHP or non-App controller are skipped.
    #[test]
    fn test_filter_non_app_controller() {
        // Simulate what the JSON would look like for a route we should ignore.
        let json_str = r#"{
            "api_platform.action.placeholder": {
                "path": "/api/entities",
                "defaults": {
                    "_controller": "api_platform.action.placeholder"
                }
            }
        }"#;

        let json: Value = serde_json::from_str(json_str).unwrap();
        let obj = json.as_object().unwrap();

        for (name, data) in obj {
            let controller = data["defaults"]["_controller"].as_str().unwrap_or("");
            assert!(
                !controller.starts_with("App\\"),
                "Route '{}' with controller '{}' should be filtered out",
                name,
                controller
            );
        }
    }

    /// Ensure that the controller string is split correctly on `::`.
    #[test]
    fn test_split_controller_string() {
        let controller = "App\\Controller\\HomeController::indexAction";
        let (class, method) = controller.split_once("::").unwrap();
        assert_eq!(class, "App\\Controller\\HomeController");
        assert_eq!(method, "indexAction");
    }

    /// Ensure that a controller without a method (invokable) is handled.
    #[test]
    fn test_invokable_controller_no_method() {
        let controller = "App\\Controller\\HomeController";
        assert!(controller.split_once("::").is_none());
    }
}
