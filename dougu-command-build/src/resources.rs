pub mod log_messages {
    pub const PACKAGING_APP: &str = "Packaging application for target: {target}, mode: {mode}, output: {output}";
    pub const README_MISSING: &str = "README.md not found in project root";
    pub const EXECUTABLE_SEARCH_FAILED: &str = "Failed to find executables in {dir}";
    pub const FOUND_EXECUTABLES: &str = "Found {count} executable(s)";
    pub const CREATING_PACKAGE_DIR: &str = "Creating package directory: {dir}";
    pub const COPIED_FILES: &str = "Copied {count} files to package directory";
    pub const PACKAGE_CREATED: &str = "Package created at: {path}";
    pub const BUILD_COMPLETE: &str = "Build completed successfully";
    pub const RUNNING_TESTS: &str = "Running {type} tests";
    pub const TEST_FILTER: &str = "Using test filter: {filter}";
    pub const CARGO_TEST_FAILED: &str = "Cargo test failed with exit code: {code}";
    pub const COMPILING_APP: &str = "Compiling application in {mode} mode, output directory: {output}";
    pub const CARGO_BUILD_FAILED: &str = "Cargo build failed with exit code: {code}";
    pub const PACKING_ARTIFACT: &str = "Packing artifact: {name}";
    pub const PACK_COMPLETE: &str = "Pack operation completed successfully";
    pub const EXECUTABLE_NOT_FOUND: &str = "No executable found in cargo output or input directory";
    pub const INVALID_EXECUTABLE_TYPE: &str = "Found a zip file instead of an executable. This would create a nested archive";
    pub const ADDING_EXECUTABLE_TO_ARCHIVE: &str = "Adding executable '{source}' to archive as '{target}'";
} 