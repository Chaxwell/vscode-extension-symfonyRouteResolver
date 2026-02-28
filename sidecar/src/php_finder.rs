use std::borrow::Cow;
use std::path::PathBuf;

use bumpalo::Bump;
use mago_database::file::{File, FileType};
use mago_syntax::ast::ast::class_like::member::ClassLikeMember;
use mago_syntax::ast::ast::namespace::Namespace;
use mago_syntax::ast::ast::statement::Statement;
use mago_syntax::ast::Program;
use mago_syntax::parser::parse_file;

use crate::protocol::RouteInfo;

/// Resolves a PHP class FQCN (and optional method name) to an absolute file
/// path and a 1-indexed line number.
///
/// The file is located using PSR-4 convention: `App\Foo\Bar` → `{workspace}/src/Foo/Bar.php`.
pub fn find_location(
    workspace_path: &str,
    class_fqcn: &str,
    method: Option<&str>,
) -> Result<RouteInfo, String> {
    let php_path = fqcn_to_path(workspace_path, class_fqcn)?;

    let content = std::fs::read_to_string(&php_path)
        .map_err(|e| format!("Cannot read '{}': {}", php_path.display(), e))?;

    let absolute_path = php_path.to_string_lossy().into_owned();

    // Extract the simple class name (last segment of the FQCN).
    let class_name = class_fqcn
        .split('\\')
        .last()
        .unwrap_or(class_fqcn);

    let arena = Bump::new(); // TODO Est-ce qu'on est sûr qu'on clean la mémoire ici ? Bump indique bien qu'il implémente pas Drop.
    let file = File::new(
        Cow::Owned(absolute_path.clone()),
        FileType::Host,
        Some(php_path),
        Cow::Owned(content),
    );
    let program = parse_file(&arena, &file);

    find_in_program(program, &file, &absolute_path, class_name, method)
        .ok_or_else(|| {
            format!(
                "Could not locate class '{}' (method: {:?}) in '{}'",
                class_fqcn, method, absolute_path
            )
        })
}

// ---------------------------------------------------------------------------
// Private helpers
// ---------------------------------------------------------------------------

/// Converts a PHP FQCN to its expected file path on disk (PSR-4, `App\` → `src/`).
fn fqcn_to_path(workspace_path: &str, fqcn: &str) -> Result<PathBuf, String> {
    // TODO Il faudra lire le composer.json pour savoir comment trouver les fichiers.
    // Only the `App\` namespace prefix is supported via PSR-4 convention.
    let relative = fqcn
        .strip_prefix("App\\")
        .ok_or_else(|| format!("Unsupported namespace prefix in FQCN '{}'", fqcn))?
        .replace('\\', "/");

    let path = PathBuf::from(workspace_path)
        .join("src")
        .join(relative)
        .with_extension("php");

    Ok(path)
}

/// Walks the top-level statements of the program to find the target class.
fn find_in_program<'a>(
    program: &Program<'a>,
    file: &File,
    absolute_path: &str,
    class_name: &str,
    method: Option<&str>,
) -> Option<RouteInfo> {
    for stmt in program.statements.iter() {
        if let Some(info) =
            find_in_statement(stmt, file, absolute_path, class_name, method)
        {
            return Some(info);
        }
    }
    None
}

/// Recursively searches a statement for the target class declaration.
fn find_in_statement<'a>(
    stmt: &Statement<'a>,
    file: &File,
    absolute_path: &str,
    class_name: &str,
    method: Option<&str>,
) -> Option<RouteInfo> {
    match stmt {
        Statement::Namespace(ns) => find_in_namespace(ns, file, absolute_path, class_name, method),
        Statement::Class(class) => {
            if class.name.value != class_name {
                return None;
            }

            if let Some(method_name) = method {
                // Search for the method inside the class.
                for member in class.members.iter() {
                    if let ClassLikeMember::Method(m) = member {
                        if m.name.value == method_name {
                            // `function` keyword span marks the declaration line.
                            let offset = m.function.span.start.offset;
                            let line = file.line_number(offset) + 1;
                            return Some(RouteInfo { file: absolute_path.to_string(), line });
                        }
                    }
                }
                // Method not found — fall back to the class keyword line.
            }

            let offset = class.class.span.start.offset;
            let line = file.line_number(offset) + 1;
            Some(RouteInfo { file: absolute_path.to_string(), line })
        }
        _ => None,
    }
}

/// Descends into a namespace body to keep searching.
fn find_in_namespace<'a>(
    ns: &Namespace<'a>,
    file: &File,
    absolute_path: &str,
    class_name: &str,
    method: Option<&str>,
) -> Option<RouteInfo> {
    for stmt in ns.statements().iter() {
        if let Some(info) =
            find_in_statement(stmt, file, absolute_path, class_name, method)
        {
            return Some(info);
        }
    }
    None
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    const SIMPLE_CLASS_PHP: &str = r#"<?php

namespace App\Controller;

class HomeController
{
    public function indexAction(): void {}
    public function aboutAction(): void {}
}
"#;

    fn parse_and_find(
        source: &str,
        class_name: &str,
        method: Option<&str>,
    ) -> Option<u32> {
        let arena = Bump::new();
        let file = File::ephemeral(
            Cow::Borrowed("test.php"),
            Cow::Owned(source.to_string()),
        );
        let program = parse_file(&arena, &file);
        find_in_program(program, &file, "/tmp/test.php", class_name, method)
            .map(|info| info.line)
    }

    #[test]
    fn test_finds_class_declaration_line() {
        let line = parse_and_find(SIMPLE_CLASS_PHP, "HomeController", None).unwrap();
        // "class HomeController" is on line 5 (1-indexed).
        assert_eq!(line, 5);
    }

    #[test]
    fn test_finds_method_declaration_line() {
        let line = parse_and_find(SIMPLE_CLASS_PHP, "HomeController", Some("indexAction")).unwrap();
        // "public function indexAction" is on line 7.
        assert_eq!(line, 7);
    }

    #[test]
    fn test_finds_second_method_declaration_line() {
        let line = parse_and_find(SIMPLE_CLASS_PHP, "HomeController", Some("aboutAction")).unwrap();
        assert_eq!(line, 8);
    }

    #[test]
    fn test_returns_class_line_when_method_not_found() {
        // Method does not exist → should fall back to class line.
        let line = parse_and_find(SIMPLE_CLASS_PHP, "HomeController", Some("nonExistent")).unwrap();
        assert_eq!(line, 5);
    }

    #[test]
    fn test_returns_none_for_unknown_class() {
        let result = parse_and_find(SIMPLE_CLASS_PHP, "UnknownController", None);
        assert!(result.is_none());
    }

    #[test]
    fn test_fqcn_to_path() {
        let path = fqcn_to_path("/workspace", "App\\Controller\\HomeController").unwrap();
        assert_eq!(path, PathBuf::from("/workspace/src/Controller/HomeController.php"));
    }

    #[test]
    fn test_fqcn_to_path_rejects_non_app_namespace() {
        let result = fqcn_to_path("/workspace", "Vendor\\Lib\\SomeClass");
        assert!(result.is_err());
    }
}
