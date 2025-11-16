//! Integration tests for NSFW (Nix Subsystem for Windows)
//!
//! These tests verify end-to-end workflows using the mock WSL2 bridge.

use nsfw::wsl2::{MockWSL2Bridge, CommandOutput};
use nsfw::nix_ops::{BridgedNixExecutor, NixError};

/// Helper to create a mock executor with common responses
fn create_test_executor() -> BridgedNixExecutor<MockWSL2Bridge> {
    let mut bridge = MockWSL2Bridge::new();
    bridge.add_common_responses();

    // Add mock responses for test packages
    bridge.set_response(
        "nix search nixpkgs firefox --json".to_string(),
        CommandOutput::new(
            r#"{"legacyPackages.x86_64-linux.firefox":{"pname":"firefox","version":"130.0","description":"A web browser from Mozilla"}}"#.to_string(),
            "".to_string(),
            0
        )
    );

    bridge.set_response(
        "nix profile install nixpkgs#firefox".to_string(),
        CommandOutput::new("".to_string(), "".to_string(), 0)
    );

    bridge.set_response(
        "nix profile remove firefox".to_string(),
        CommandOutput::new("".to_string(), "".to_string(), 0)
    );

    BridgedNixExecutor::new(bridge)
}

#[test]
fn test_workflow_search_install_list_remove() {
    // Complete workflow: search → install → list → remove
    let executor = create_test_executor();

    // Step 1: Verify Nix is available
    let version = executor.check_nix_available();
    assert!(version.is_ok(), "Nix should be available");

    // Step 2: Search for a package
    let search_results = executor.search("firefox", 10);
    assert!(search_results.is_ok(), "Search should succeed");
    let results = search_results.unwrap();
    assert_eq!(results.len(), 1, "Should find 1 result");
    assert_eq!(results[0].pname, "firefox");

    // Step 3: Install the package
    let install_result = executor.install("firefox");
    assert!(install_result.is_ok(), "Install should succeed");

    // Step 4: List installed packages
    let list_result = executor.list();
    assert!(list_result.is_ok(), "List should succeed");

    // Step 5: Remove the package
    let remove_result = executor.remove("firefox");
    assert!(remove_result.is_ok(), "Remove should succeed");
}

#[test]
fn test_wsl2_not_available_error() {
    // Test that all operations fail gracefully when WSL2 is not available
    let mut bridge = MockWSL2Bridge::new();
    bridge.set_available(false);

    let executor = BridgedNixExecutor::new(bridge);

    // Test check_nix_available
    let result = executor.check_nix_available();
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), NixError::WSL2NotAvailable));

    // Test search
    let result = executor.search("test", 10);
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), NixError::WSL2NotAvailable));

    // Test install
    let result = executor.install("test");
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), NixError::WSL2NotAvailable));

    // Test remove
    let result = executor.remove("test");
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), NixError::WSL2NotAvailable));

    // Test list
    let result = executor.list();
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), NixError::WSL2NotAvailable));
}

#[test]
fn test_nix_command_failure() {
    // Test handling of Nix command failures
    let mut bridge = MockWSL2Bridge::new();
    bridge.add_common_responses();

    // Add a failing command response
    bridge.set_response(
        "nix search nixpkgs nonexistent --json".to_string(),
        CommandOutput::new(
            "".to_string(),
            "error: package not found".to_string(),
            1
        )
    );

    let executor = BridgedNixExecutor::new(bridge);

    let result = executor.search("nonexistent", 10);
    assert!(result.is_err());

    match result.unwrap_err() {
        NixError::CommandFailed(msg) => {
            assert!(msg.contains("package not found"));
        }
        _ => panic!("Expected CommandFailed error"),
    }
}

#[test]
fn test_package_already_installed() {
    // Test installing an already-installed package
    let mut bridge = MockWSL2Bridge::new();
    bridge.add_common_responses();

    bridge.set_response(
        "nix profile install nixpkgs#firefox".to_string(),
        CommandOutput::new(
            "".to_string(),
            "error: package 'firefox' is already installed".to_string(),
            1
        )
    );

    let executor = BridgedNixExecutor::new(bridge);

    let result = executor.install("firefox");
    assert!(result.is_err());

    match result.unwrap_err() {
        NixError::AlreadyInstalled(pkg) => {
            assert_eq!(pkg, "firefox");
        }
        _ => panic!("Expected AlreadyInstalled error"),
    }
}

#[test]
fn test_package_not_installed() {
    // Test removing a package that isn't installed
    let mut bridge = MockWSL2Bridge::new();
    bridge.add_common_responses();

    bridge.set_response(
        "nix profile remove firefox".to_string(),
        CommandOutput::new(
            "".to_string(),
            "error: package 'firefox' not found in profile".to_string(),
            1
        )
    );

    let executor = BridgedNixExecutor::new(bridge);

    let result = executor.remove("firefox");
    assert!(result.is_err());

    match result.unwrap_err() {
        NixError::NotInstalled(pkg) => {
            assert_eq!(pkg, "firefox");
        }
        _ => panic!("Expected NotInstalled error"),
    }
}

#[test]
fn test_search_with_limit() {
    // Test search respects limit parameter
    let mut bridge = MockWSL2Bridge::new();
    bridge.add_common_responses();

    // Mock response with multiple packages
    bridge.set_response(
        "nix search nixpkgs browser --json".to_string(),
        CommandOutput::new(
            r#"{
                "legacyPackages.x86_64-linux.firefox":{"pname":"firefox","version":"130.0","description":"Mozilla Firefox"},
                "legacyPackages.x86_64-linux.chromium":{"pname":"chromium","version":"120.0","description":"Chromium browser"},
                "legacyPackages.x86_64-linux.brave":{"pname":"brave","version":"1.60","description":"Brave browser"},
                "legacyPackages.x86_64-linux.vivaldi":{"pname":"vivaldi","version":"6.4","description":"Vivaldi browser"}
            }"#.to_string(),
            "".to_string(),
            0
        )
    );

    let executor = BridgedNixExecutor::new(bridge);

    // Test with limit of 2
    let results = executor.search("browser", 2).unwrap();
    assert_eq!(results.len(), 2, "Should respect limit of 2");
}

#[test]
fn test_list_installed_packages() {
    // Test listing installed packages with proper parsing
    let mut bridge = MockWSL2Bridge::new();
    bridge.add_common_responses();

    bridge.set_response(
        "nix profile list --json".to_string(),
        CommandOutput::new(
            r#"{
                "elements": {
                    "firefox": {
                        "storePaths": ["/nix/store/abc123-firefox-130.0/bin/firefox"]
                    },
                    "vim": {
                        "storePaths": ["/nix/store/def456-vim-9.0/bin/vim"]
                    }
                }
            }"#.to_string(),
            "".to_string(),
            0
        )
    );

    let executor = BridgedNixExecutor::new(bridge);

    let packages = executor.list().unwrap();
    assert_eq!(packages.len(), 2);

    // Check Firefox package
    let firefox = packages.iter().find(|p| p.name == "firefox").unwrap();
    assert_eq!(firefox.version, "130.0");
    assert!(firefox.store_path.contains("firefox-130.0"));

    // Check Vim package
    let vim = packages.iter().find(|p| p.name == "vim").unwrap();
    assert_eq!(vim.version, "9.0");
    assert!(vim.store_path.contains("vim-9.0"));
}

#[test]
fn test_empty_search_results() {
    // Test handling of empty search results
    let mut bridge = MockWSL2Bridge::new();
    bridge.add_common_responses();

    // Add response for non-existent package (empty JSON object)
    bridge.set_response(
        "nix search nixpkgs nonexistent-package-xyz --json".to_string(),
        CommandOutput::new("{}".to_string(), "".to_string(), 0)
    );

    let executor = BridgedNixExecutor::new(bridge);

    let results = executor.search("nonexistent-package-xyz", 10).unwrap();
    assert_eq!(results.len(), 0, "Should return empty results for non-existent package");
}

#[test]
fn test_empty_package_list() {
    // Test handling of empty installed package list
    let executor = create_test_executor();

    let packages = executor.list().unwrap();
    assert_eq!(packages.len(), 0, "Should return empty list when no packages installed");
}

#[test]
fn test_version_extraction_from_store_path() {
    // Test version extraction logic with various store path formats
    let mut bridge = MockWSL2Bridge::new();
    bridge.add_common_responses();

    bridge.set_response(
        "nix profile list --json".to_string(),
        CommandOutput::new(
            r#"{
                "elements": {
                    "firefox": {"storePaths": ["/nix/store/abc-firefox-130.0-x86_64/bin"]},
                    "python3": {"storePaths": ["/nix/store/def-python3-3.11.6/bin"]},
                    "nodejs": {"storePaths": ["/nix/store/ghi-nodejs-20.10.0-linux/bin"]},
                    "simple": {"storePaths": ["/nix/store/jkl-simple/bin"]}
                }
            }"#.to_string(),
            "".to_string(),
            0
        )
    );

    let executor = BridgedNixExecutor::new(bridge);
    let packages = executor.list().unwrap();

    // Firefox: standard version
    let firefox = packages.iter().find(|p| p.name == "firefox").unwrap();
    assert_eq!(firefox.version, "130.0");

    // Python: version with multiple dots
    let python = packages.iter().find(|p| p.name == "python3").unwrap();
    assert_eq!(python.version, "3.11.6");

    // Node.js: version before platform suffix
    let nodejs = packages.iter().find(|p| p.name == "nodejs").unwrap();
    assert_eq!(nodejs.version, "20.10.0");

    // Simple: no version in path
    let simple = packages.iter().find(|p| p.name == "simple").unwrap();
    assert_eq!(simple.version, "unknown");
}

#[test]
fn test_json_parse_error_handling() {
    // Test handling of invalid JSON responses
    let mut bridge = MockWSL2Bridge::new();
    bridge.add_common_responses();

    bridge.set_response(
        "nix search nixpkgs malformed --json".to_string(),
        CommandOutput::new(
            "{ invalid json".to_string(),
            "".to_string(),
            0
        )
    );

    let executor = BridgedNixExecutor::new(bridge);

    let result = executor.search("malformed", 10);
    assert!(result.is_err());

    match result.unwrap_err() {
        NixError::ParseError(_) => {},
        _ => panic!("Expected ParseError"),
    }
}

#[test]
fn test_concurrent_operations_safety() {
    // Test that executor can be used for multiple operations sequentially
    let executor = create_test_executor();

    // Multiple searches
    let _ = executor.search("test1", 10);
    let _ = executor.search("test2", 10);
    let _ = executor.search("test3", 10);

    // Verify state is maintained
    let version = executor.check_nix_available();
    assert!(version.is_ok());
}

#[test]
fn test_special_characters_in_package_names() {
    // Test handling of package names with special characters
    let mut bridge = MockWSL2Bridge::new();
    bridge.add_common_responses();

    bridge.set_response(
        "nix search nixpkgs python3.11 --json".to_string(),
        CommandOutput::new(
            r#"{"legacyPackages.x86_64-linux.python311":{"pname":"python3.11","version":"3.11.6","description":"Python"}}"#.to_string(),
            "".to_string(),
            0
        )
    );

    let executor = BridgedNixExecutor::new(bridge);
    let results = executor.search("python3.11", 10).unwrap();

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].pname, "python3.11");
}

// ============================================================================
// Batch Operation Integration Tests
// ============================================================================

#[test]
fn test_batch_install_all_successful() {
    // Test batch installing multiple packages - all succeed
    let mut bridge = MockWSL2Bridge::new();
    bridge.add_common_responses();

    // Add responses for three successful installs
    bridge.set_response(
        "nix profile install nixpkgs#firefox".to_string(),
        CommandOutput::new("".to_string(), "".to_string(), 0)
    );
    bridge.set_response(
        "nix profile install nixpkgs#vim".to_string(),
        CommandOutput::new("".to_string(), "".to_string(), 0)
    );
    bridge.set_response(
        "nix profile install nixpkgs#git".to_string(),
        CommandOutput::new("".to_string(), "".to_string(), 0)
    );

    let executor = BridgedNixExecutor::new(bridge);

    // Install all three packages
    let result1 = executor.install("firefox");
    let result2 = executor.install("vim");
    let result3 = executor.install("git");

    assert!(result1.is_ok(), "Firefox install should succeed");
    assert!(result2.is_ok(), "Vim install should succeed");
    assert!(result3.is_ok(), "Git install should succeed");
}

#[test]
fn test_batch_install_with_partial_failure() {
    // Test batch installing with some packages failing
    let mut bridge = MockWSL2Bridge::new();
    bridge.add_common_responses();

    // Firefox succeeds
    bridge.set_response(
        "nix profile install nixpkgs#firefox".to_string(),
        CommandOutput::new("".to_string(), "".to_string(), 0)
    );

    // Vim already installed
    bridge.set_response(
        "nix profile install nixpkgs#vim".to_string(),
        CommandOutput::new(
            "".to_string(),
            "error: package 'vim' is already installed".to_string(),
            1
        )
    );

    // Git succeeds
    bridge.set_response(
        "nix profile install nixpkgs#git".to_string(),
        CommandOutput::new("".to_string(), "".to_string(), 0)
    );

    let executor = BridgedNixExecutor::new(bridge);

    let result1 = executor.install("firefox");
    let result2 = executor.install("vim");
    let result3 = executor.install("git");

    assert!(result1.is_ok(), "Firefox should succeed");
    assert!(result2.is_err(), "Vim should fail (already installed)");
    assert!(result3.is_ok(), "Git should succeed");

    // Verify error type for already installed
    match result2.unwrap_err() {
        NixError::AlreadyInstalled(pkg) => assert_eq!(pkg, "vim"),
        _ => panic!("Expected AlreadyInstalled error"),
    }
}

#[test]
fn test_batch_remove_all_successful() {
    // Test batch removing multiple packages - all succeed
    let mut bridge = MockWSL2Bridge::new();
    bridge.add_common_responses();

    // Add responses for three successful removals
    bridge.set_response(
        "nix profile remove firefox".to_string(),
        CommandOutput::new("".to_string(), "".to_string(), 0)
    );
    bridge.set_response(
        "nix profile remove vim".to_string(),
        CommandOutput::new("".to_string(), "".to_string(), 0)
    );
    bridge.set_response(
        "nix profile remove git".to_string(),
        CommandOutput::new("".to_string(), "".to_string(), 0)
    );

    let executor = BridgedNixExecutor::new(bridge);

    // Remove all three packages
    let result1 = executor.remove("firefox");
    let result2 = executor.remove("vim");
    let result3 = executor.remove("git");

    assert!(result1.is_ok(), "Firefox removal should succeed");
    assert!(result2.is_ok(), "Vim removal should succeed");
    assert!(result3.is_ok(), "Git removal should succeed");
}

#[test]
fn test_batch_remove_with_not_installed() {
    // Test batch removing with some packages not installed
    let mut bridge = MockWSL2Bridge::new();
    bridge.add_common_responses();

    // Firefox succeeds
    bridge.set_response(
        "nix profile remove firefox".to_string(),
        CommandOutput::new("".to_string(), "".to_string(), 0)
    );

    // Vim not installed
    bridge.set_response(
        "nix profile remove vim".to_string(),
        CommandOutput::new(
            "".to_string(),
            "error: package 'vim' not found in profile".to_string(),
            1
        )
    );

    // Git succeeds
    bridge.set_response(
        "nix profile remove git".to_string(),
        CommandOutput::new("".to_string(), "".to_string(), 0)
    );

    let executor = BridgedNixExecutor::new(bridge);

    let result1 = executor.remove("firefox");
    let result2 = executor.remove("vim");
    let result3 = executor.remove("git");

    assert!(result1.is_ok(), "Firefox should succeed");
    assert!(result2.is_err(), "Vim should fail (not installed)");
    assert!(result3.is_ok(), "Git should succeed");

    // Verify error type for not installed
    match result2.unwrap_err() {
        NixError::NotInstalled(pkg) => assert_eq!(pkg, "vim"),
        _ => panic!("Expected NotInstalled error"),
    }
}

#[test]
fn test_batch_operations_maintain_sequence() {
    // Test that batch operations maintain order and can handle many packages
    let mut bridge = MockWSL2Bridge::new();
    bridge.add_common_responses();

    let packages = vec!["pkg1", "pkg2", "pkg3", "pkg4", "pkg5"];

    // Add responses for all packages
    for pkg in &packages {
        bridge.set_response(
            format!("nix profile install nixpkgs#{}", pkg),
            CommandOutput::new("".to_string(), "".to_string(), 0)
        );
    }

    let executor = BridgedNixExecutor::new(bridge);

    // Install packages in sequence
    for pkg in &packages {
        let result = executor.install(pkg);
        assert!(result.is_ok(), "Package {} should install successfully", pkg);
    }
}

#[test]
fn test_batch_with_mixed_success_and_failure() {
    // Test realistic scenario: some succeed, some already installed, some fail
    let mut bridge = MockWSL2Bridge::new();
    bridge.add_common_responses();

    // Package 1: Success
    bridge.set_response(
        "nix profile install nixpkgs#firefox".to_string(),
        CommandOutput::new("".to_string(), "".to_string(), 0)
    );

    // Package 2: Already installed
    bridge.set_response(
        "nix profile install nixpkgs#vim".to_string(),
        CommandOutput::new(
            "".to_string(),
            "error: package 'vim' is already installed".to_string(),
            1
        )
    );

    // Package 3: Success
    bridge.set_response(
        "nix profile install nixpkgs#git".to_string(),
        CommandOutput::new("".to_string(), "".to_string(), 0)
    );

    // Package 4: Command fails
    bridge.set_response(
        "nix profile install nixpkgs#nonexistent".to_string(),
        CommandOutput::new(
            "".to_string(),
            "error: package not found".to_string(),
            1
        )
    );

    // Package 5: Success
    bridge.set_response(
        "nix profile install nixpkgs#python3".to_string(),
        CommandOutput::new("".to_string(), "".to_string(), 0)
    );

    let executor = BridgedNixExecutor::new(bridge);

    let results = vec![
        executor.install("firefox"),      // Success
        executor.install("vim"),          // AlreadyInstalled
        executor.install("git"),          // Success
        executor.install("nonexistent"),  // CommandFailed
        executor.install("python3"),      // Success
    ];

    // Count successes and failures
    let success_count = results.iter().filter(|r| r.is_ok()).count();
    let already_installed = results.iter()
        .filter(|r| matches!(r.as_ref().err(), Some(NixError::AlreadyInstalled(_))))
        .count();
    let failures = results.iter()
        .filter(|r| r.is_err() && !matches!(r.as_ref().err(), Some(NixError::AlreadyInstalled(_))))
        .count();

    assert_eq!(success_count, 3, "Should have 3 successful installs");
    assert_eq!(already_installed, 1, "Should have 1 already installed");
    assert_eq!(failures, 1, "Should have 1 failure");
}
