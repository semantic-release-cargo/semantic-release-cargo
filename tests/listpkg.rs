#![allow(clippy::unwrap_used)]

use std::env;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process;
use std::process::Output;

use log::Level;
use semantic_release_cargo::LoggerBuilder;
use semantic_release_cargo::{list_packages, list_packages_with_arguments};

enum TestVariants {
    Basic,
    Workspace,
    AlternateRegistryRestrictionInWorkspace,
    AlternateRegistryRestrictionInWorkspaceUnsetAlt,
}

impl TestVariants {
    const fn test_name_repr_str(&self) -> &'static str {
        match self {
            Self::Basic => "basic",
            Self::Workspace => "workspace",
            Self::AlternateRegistryRestrictionInWorkspace => {
                "alternate_registry_restriction_in_workspace"
            }
            Self::AlternateRegistryRestrictionInWorkspaceUnsetAlt => {
                "alternate_registry_restriction_in_workspace_with_unset_alt"
            }
        }
    }
}

fn initialize_logger_with_level(max_level: Level) {
    let log_builder = LoggerBuilder::default().max_level(max_level);

    // initialize the log_builder into a logger
    let boxed_logger = log_builder.finalize().unwrap();
    let max_level_filter = boxed_logger.max_level_filter();

    log::set_boxed_logger(boxed_logger)
        .map(|()| log::set_max_level(max_level_filter))
        .unwrap();
}

fn list_dependencies_basic_child_main() {
    initialize_logger_with_level(Level::Error);
    let path = get_test_data_manifest_path("basic");

    list_packages(Some(path)).expect("unable to list packages");
}

fn list_dependencies_workspace_child_main() {
    initialize_logger_with_level(Level::Error);
    let path = get_test_data_manifest_path("dependencies");

    list_packages(Some(path)).expect("unable to list packages");
}

fn list_dependencies_with_alternate_registry_restriction_in_workspace_child_main() {
    initialize_logger_with_level(Level::Error);
    with_env_var(
        "CARGO_REGISTRIES_TEST_INDEX",
        "https://github.com/rust-lang/crates.io-index",
        || {
            let path = get_test_data_manifest_path("dependencies_alternate_registry");

            // Test with a target registry set.
            let alternate_registry = Some("test");
            list_packages_with_arguments(alternate_registry, Some(path))
                .expect("unable to list packages");
        },
    )
}

fn list_dependencies_with_alternate_registry_restriction_in_workspace_unset_alt_child_main() {
    initialize_logger_with_level(Level::Error);
    with_env_var(
        "CARGO_REGISTRIES_TEST_INDEX",
        "https://github.com/rust-lang/crates.io-index",
        || {
            let path = get_test_data_manifest_path("dependencies_alternate_registry");

            list_packages(Some(path.clone())).expect("unable to list packages");
        },
    )
}

fn run_child(child_test_variant: TestVariants) -> Result<Output, std::io::Error> {
    let exe = env::current_exe().unwrap();
    process::Command::new(exe)
        .env("LISTPKGS_TEST", child_test_variant.test_name_repr_str())
        .output()
}

fn rust_child_test_with_assertion_fn(
    test_name: &str,
    child_test_variant: TestVariants,
    assert_fn: impl Fn(Output),
) {
    print!("test {} ... ", test_name);
    let output = run_child(child_test_variant)
        .unwrap_or_else(|e| panic!("Unable to start child process: {}", e));

    assert_fn(output);
    println!("ok");
}

fn get_test_data_manifest_path(dir: impl AsRef<Path>) -> PathBuf {
    let mut path = PathBuf::from(file!());

    path.pop();
    path.pop();
    path.push("test_data");
    path.push(dir);
    path.push("Cargo.toml");

    path
}

fn with_env_var<K, V, F>(key: K, value: V, f: F)
where
    K: AsRef<OsStr>,
    V: AsRef<OsStr>,
    F: FnOnce(),
{
    use std::env;

    // Store the previous value of the var, if defined.
    let previous_val = env::var(key.as_ref()).ok();

    env::set_var(key.as_ref(), value.as_ref());
    (f)();

    // Reset or clear the var after the test.
    if let Some(previous_val) = previous_val {
        env::set_var(key.as_ref(), previous_val);
    } else {
        env::remove_var(key.as_ref());
    }
}

fn main() {
    let maybe_test_name = env::var("LISTPKGS_TEST").ok();

    match maybe_test_name.as_deref() {
        Some("basic") => {
            list_dependencies_basic_child_main();
        }
        Some("workspace") => {
            list_dependencies_workspace_child_main();
        }
        Some("alternate_registry_restriction_in_workspace") => {
            list_dependencies_with_alternate_registry_restriction_in_workspace_child_main()
        }
        Some("alternate_registry_restriction_in_workspace_with_unset_alt") => {
            list_dependencies_with_alternate_registry_restriction_in_workspace_unset_alt_child_main()
        }
        _ => {
            parent_main();
        }
    }
}

fn parent_main() {
    let test_cnt = 4_usize;
    println!("running {} tests", test_cnt);

    rust_child_test_with_assertion_fn(
        "list_dependencies_basic_workspace",
        TestVariants::Basic,
        |output| {
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            let lines: Vec<_> = stderr.lines().collect();

            assert!(lines[0].starts_with("basic"));
        },
    );

    rust_child_test_with_assertion_fn(
        "list_dependencies_workspace",
        TestVariants::Workspace,
        |output| {
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            let lines: Vec<_> = stderr.lines().collect();

            if lines[0].starts_with("build1") {
                assert!(lines[1].starts_with("dep1"));
            } else {
                assert!(lines[0].starts_with("dep1"));
                assert!(lines[1].starts_with("build1"));
            }
            assert!(lines[2].starts_with("dependencies"));
        },
    );

    rust_child_test_with_assertion_fn(
        "list_dependencies_with_alternate_registry_restriction_in_workspace_with_unset_alternate_registry",
        TestVariants::AlternateRegistryRestrictionInWorkspaceUnsetAlt,
        |output| {
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            let lines: Vec<_> = stderr.lines().collect();

           assert!(lines.is_empty()) 
        },
    );

    rust_child_test_with_assertion_fn(
        "list_dependencies_with_alternate_registry_restriction_in_workspace",
        TestVariants::AlternateRegistryRestrictionInWorkspace,
        |output| {
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            let lines: Vec<_> = stderr.lines().collect();

            if lines[0].starts_with("build1") {
                assert!(lines[1].starts_with("dep1"), "{}", &lines.join("\n"));
            } else {
                assert!(lines[0].starts_with("dep1"));
                assert!(lines[1].starts_with("build1"));
            }
            assert!(lines[2].starts_with("dependencies_alt_registry"));
        },
    );

    println!(
        "\ntest result: ok. {} passed; 0 failed; 0 ignored; 0 measured; 0 filtered out",
        test_cnt
    )
}
