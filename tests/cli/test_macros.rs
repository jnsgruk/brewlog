/// Generates a test that asserts a CLI command fails without authentication.
macro_rules! define_cli_auth_test {
    ($name:ident, $args:expr) => {
        #[test]
        fn $name() {
            let _ = crate::helpers::server_info();
            let output = crate::helpers::run_brewlog($args, &[]);
            assert!(
                !output.status.success(),
                "{} should require authentication",
                stringify!($name)
            );
        }
    };
}

/// Generates a test that asserts a CLI list command succeeds without
/// authentication and returns a valid JSON array.
macro_rules! define_cli_list_test {
    ($name:ident, $args:expr) => {
        #[test]
        fn $name() {
            let _ = crate::helpers::server_info();
            let output = crate::helpers::run_brewlog($args, &[]);
            assert!(
                output.status.success(),
                "{} should succeed: {}",
                stringify!($name),
                String::from_utf8_lossy(&output.stderr)
            );
            let stdout = String::from_utf8_lossy(&output.stdout);
            let items: serde_json::Value =
                serde_json::from_str(&stdout).expect("Should output valid JSON");
            assert!(items.is_array(), "Should return a JSON array");
        }
    };
}

pub(crate) use define_cli_auth_test;
pub(crate) use define_cli_list_test;
