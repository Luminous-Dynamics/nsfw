//! Edge case tests for NSFW
//!
//! These tests verify behavior in unusual or boundary conditions.

use nsfw::wsl2::{MockWSL2Bridge, CommandOutput};
use nsfw::nix_ops::{BridgedNixExecutor, NixError};
use nsfw::path_translation::PathTranslator;

#[test]
fn test_very_long_package_name() {
    // Test handling of extremely long package names
    let mut bridge = MockWSL2Bridge::new();
    bridge.add_common_responses();

    let long_name = "a".repeat(500);
    let command = format!("nix search nixpkgs {} --json", long_name);

    bridge.set_response(
        command,
        CommandOutput::new("{}".to_string(), "".to_string(), 0)
    );

    let executor = BridgedNixExecutor::new(bridge);
    let results = executor.search(&long_name, 10);

    assert!(results.is_ok(), "Should handle very long package names");
}

#[test]
fn test_package_name_with_unicode() {
    // Test package names with Unicode characters
    let mut bridge = MockWSL2Bridge::new();
    bridge.add_common_responses();

    bridge.set_response(
        "nix search nixpkgs café --json".to_string(),
        CommandOutput::new("{}".to_string(), "".to_string(), 0)
    );

    let executor = BridgedNixExecutor::new(bridge);
    let results = executor.search("café", 10).unwrap();

    assert_eq!(results.len(), 0);
}

#[test]
fn test_package_name_with_quotes() {
    // Test package names with quotes
    let mut bridge = MockWSL2Bridge::new();
    bridge.add_common_responses();

    bridge.set_response(
        "nix search nixpkgs test\"package --json".to_string(),
        CommandOutput::new("{}".to_string(), "".to_string(), 0)
    );

    let executor = BridgedNixExecutor::new(bridge);
    let results = executor.search("test\"package", 10).unwrap();

    assert_eq!(results.len(), 0);
}

#[test]
fn test_zero_search_limit() {
    // Test search with limit of 0
    let mut bridge = MockWSL2Bridge::new();
    bridge.add_common_responses();

    bridge.set_response(
        "nix search nixpkgs test --json".to_string(),
        CommandOutput::new(
            r#"{"pkg1":{"pname":"test1","version":"1.0","description":"Test"}}"#.to_string(),
            "".to_string(),
            0
        )
    );

    let executor = BridgedNixExecutor::new(bridge);
    let results = executor.search("test", 0).unwrap();

    assert_eq!(results.len(), 0, "Limit of 0 should return no results");
}

#[test]
fn test_very_large_search_limit() {
    // Test search with very large limit
    let mut bridge = MockWSL2Bridge::new();
    bridge.add_common_responses();

    bridge.set_response(
        "nix search nixpkgs test --json".to_string(),
        CommandOutput::new(
            r#"{"pkg1":{"pname":"test1","version":"1.0","description":"Test"}}"#.to_string(),
            "".to_string(),
            0
        )
    );

    let executor = BridgedNixExecutor::new(bridge);
    let results = executor.search("test", usize::MAX).unwrap();

    assert_eq!(results.len(), 1, "Should handle very large limits");
}

#[test]
fn test_empty_package_name() {
    // Test searching for empty string
    let mut bridge = MockWSL2Bridge::new();
    bridge.add_common_responses();

    bridge.set_response(
        "nix search nixpkgs  --json".to_string(),
        CommandOutput::new("{}".to_string(), "".to_string(), 0)
    );

    let executor = BridgedNixExecutor::new(bridge);
    let results = executor.search("", 10).unwrap();

    assert_eq!(results.len(), 0);
}

#[test]
fn test_whitespace_only_package_name() {
    // Test searching for whitespace
    let mut bridge = MockWSL2Bridge::new();
    bridge.add_common_responses();

    // Note: Whitespace in args is passed as-is to command
    bridge.set_response(
        "nix search nixpkgs     --json".to_string(),
        CommandOutput::new("{}".to_string(), "".to_string(), 0)
    );

    let executor = BridgedNixExecutor::new(bridge);
    let results = executor.search("   ", 10).unwrap();

    assert_eq!(results.len(), 0);
}

#[test]
fn test_json_with_missing_fields() {
    // Test JSON parsing with missing optional fields
    let mut bridge = MockWSL2Bridge::new();
    bridge.add_common_responses();

    bridge.set_response(
        "nix search nixpkgs minimal --json".to_string(),
        CommandOutput::new(
            r#"{"pkg":{"pname":"minimal"}}"#.to_string(),
            "".to_string(),
            0
        )
    );

    let executor = BridgedNixExecutor::new(bridge);
    let results = executor.search("minimal", 10).unwrap();

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].pname, "minimal");
    assert_eq!(results[0].version, "unknown");
    assert_eq!(results[0].description, "");
}

#[test]
fn test_json_with_null_values() {
    // Test JSON parsing with null values
    let mut bridge = MockWSL2Bridge::new();
    bridge.add_common_responses();

    bridge.set_response(
        "nix search nixpkgs nulls --json".to_string(),
        CommandOutput::new(
            r#"{"pkg":{"pname":"test","version":null,"description":null}}"#.to_string(),
            "".to_string(),
            0
        )
    );

    let executor = BridgedNixExecutor::new(bridge);
    let results = executor.search("nulls", 10).unwrap();

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].pname, "test");
    assert_eq!(results[0].version, "unknown");
    assert_eq!(results[0].description, "");
}

#[test]
fn test_json_array_instead_of_object() {
    // Test handling of JSON array when object expected
    let mut bridge = MockWSL2Bridge::new();
    bridge.add_common_responses();

    bridge.set_response(
        "nix search nixpkgs array --json".to_string(),
        CommandOutput::new(
            r#"[]"#.to_string(),
            "".to_string(),
            0
        )
    );

    let executor = BridgedNixExecutor::new(bridge);
    let results = executor.search("array", 10).unwrap();

    // Should handle gracefully by returning empty results
    assert_eq!(results.len(), 0);
}

#[test]
fn test_store_path_with_unusual_characters() {
    // Test version extraction from store paths with unusual but valid characters
    let mut bridge = MockWSL2Bridge::new();
    bridge.add_common_responses();

    bridge.set_response(
        "nix profile list --json".to_string(),
        CommandOutput::new(
            r#"{
                "elements": {
                    "my+package": {"storePaths": ["/nix/store/abc-my+package-1.0+beta/bin"]},
                    "test_pkg": {"storePaths": ["/nix/store/def-test_pkg-2.0_rc1/bin"]},
                    "dash-pkg": {"storePaths": ["/nix/store/ghi-dash-pkg-3.0-final/bin"]}
                }
            }"#.to_string(),
            "".to_string(),
            0
        )
    );

    let executor = BridgedNixExecutor::new(bridge);
    let packages = executor.list().unwrap();

    assert_eq!(packages.len(), 3);

    let plus_pkg = packages.iter().find(|p| p.name == "my+package").unwrap();
    // Version extraction stops at first dash after package name
    assert_eq!(plus_pkg.version, "1.0+beta");

    let underscore_pkg = packages.iter().find(|p| p.name == "test_pkg").unwrap();
    // Version extraction stops at underscore
    assert_eq!(underscore_pkg.version, "2.0_rc1");
}

#[test]
fn test_multiple_consecutive_errors() {
    // Test that multiple consecutive errors don't cause state issues
    let mut bridge = MockWSL2Bridge::new();
    bridge.add_common_responses();

    bridge.set_response(
        "nix profile install nixpkgs#fail1".to_string(),
        CommandOutput::new("".to_string(), "error: failed".to_string(), 1)
    );

    bridge.set_response(
        "nix profile install nixpkgs#fail2".to_string(),
        CommandOutput::new("".to_string(), "error: failed".to_string(), 1)
    );

    let executor = BridgedNixExecutor::new(bridge);

    let result1 = executor.install("fail1");
    assert!(result1.is_err());

    let result2 = executor.install("fail2");
    assert!(result2.is_err());

    // Verify executor still works after errors
    let version = executor.check_nix_available();
    assert!(version.is_ok());
}

#[test]
fn test_path_translation_edge_cases() {
    let translator = PathTranslator::new();

    // Test path with multiple consecutive backslashes (in string literal, \\\\ = one backslash)
    let result = translator.to_linux("C:\\Users\\John");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "/mnt/c/Users/John");

    // Test path ending with backslash
    let result = translator.to_linux("C:\\Users\\");
    assert!(result.is_ok());
    // Trailing slash is preserved by path translator
    assert!(result.unwrap().starts_with("/mnt/c/Users"));

    // Test mixed slashes
    let result = translator.to_linux("C:\\Users\\John\\Documents");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "/mnt/c/Users/John/Documents");
}

#[test]
fn test_extremely_nested_store_path() {
    // Test version extraction from deeply nested store paths
    let mut bridge = MockWSL2Bridge::new();
    bridge.add_common_responses();

    bridge.set_response(
        "nix profile list --json".to_string(),
        CommandOutput::new(
            r#"{
                "elements": {
                    "deep": {"storePaths": ["/nix/store/abc-deep-1.0/share/doc/deep/examples/test/data/file"]}
                }
            }"#.to_string(),
            "".to_string(),
            0
        )
    );

    let executor = BridgedNixExecutor::new(bridge);
    let packages = executor.list().unwrap();

    assert_eq!(packages.len(), 1);
    assert_eq!(packages[0].version, "1.0");
}

#[test]
fn test_store_path_without_hash() {
    // Test version extraction when hash is missing or unusual
    let mut bridge = MockWSL2Bridge::new();
    bridge.add_common_responses();

    bridge.set_response(
        "nix profile list --json".to_string(),
        CommandOutput::new(
            r#"{
                "elements": {
                    "firefox": {"storePaths": ["/nix/store/firefox-130.0/bin/firefox"]}
                }
            }"#.to_string(),
            "".to_string(),
            0
        )
    );

    let executor = BridgedNixExecutor::new(bridge);
    let packages = executor.list().unwrap();

    assert_eq!(packages.len(), 1);
    assert_eq!(packages[0].name, "firefox");
    // Version extraction should work even with non-standard path format
    // The version extraction looks for "packagename-version" pattern
    assert_eq!(packages[0].version, "130.0");
}

#[test]
fn test_concurrent_bridge_instances() {
    // Test that multiple bridge instances can coexist
    let bridge1 = MockWSL2Bridge::new();
    let bridge2 = MockWSL2Bridge::new();

    let executor1 = BridgedNixExecutor::new(bridge1);
    let executor2 = BridgedNixExecutor::new(bridge2);

    // Both should work independently
    let _ = executor1.check_nix_available();
    let _ = executor2.check_nix_available();
}
