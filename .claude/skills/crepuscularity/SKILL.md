```markdown
# crepuscularity Development Patterns

> Auto-generated skill from repository analysis

## Overview
This skill teaches the core development patterns and conventions used in the `crepuscularity` Rust repository. It covers file naming, import/export styles, commit message practices, and testing patterns, providing clear examples and step-by-step workflows to streamline contributions and maintain code consistency.

## Coding Conventions

### File Naming
- Use **snake_case** for Rust file names and kebab-case for crate/package directory names where Cargo package names use hyphens.
  - **Example:** `template_context.rs`, `crepuscularity-web`

### Import Style
- Use Rust module paths within the project.
  - **Example:**
    ```rust
    mod utils;
    use crate::utils::helper_function;
    ```

### Export Style
- Use **named exports** for functions, structs, and modules.
  - **Example:**
    ```rust
    pub fn calculate_twilight() { ... }
    pub struct SunPosition { ... }
    ```

### Commit Message Patterns
- Commit messages are **freeform** and typically concise (average 46 characters).
- No enforced prefixes, but keep messages clear and descriptive.
  - **Example:**  
    ```
    Add sunrise calculation to time module
    Fix bug in angle normalization
    ```

## Workflows

### Adding a New Module
**Trigger:** When you need to introduce new functionality.
**Command:** `/add-module`

1. Create a new Rust file using snake_case (e.g., `new_feature.rs`).
2. Implement your functions/structs with `pub` for named exports.
3. Use Rust module paths to include dependencies from other modules.
4. Update the main module (`lib.rs` or `main.rs`) to include your new module.
5. Write corresponding tests near the implementation or in the crate's `tests/` directory.

### Writing and Running Tests
**Trigger:** When you need to verify code correctness.
**Command:** `/run-tests`

1. Create tests near the implementation or in a crate-level integration test file (e.g., `tests/sunrise.rs`).
2. Write test functions using Rust's built-in test framework.
    ```rust
    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_sunrise_time() {
            assert_eq!(calculate_sunrise( ... ), expected_value);
        }
    }
    ```
3. Run tests using Cargo:
    ```
    cargo test
    ```

### Making a Commit
**Trigger:** After implementing or fixing a feature.
**Command:** `/commit`

1. Stage your changes:
    ```
    git add .
    ```
2. Write a clear, concise commit message (no enforced prefix).
    ```
    git commit -m "Describe your change here"
    ```
3. Push your changes:
    ```
    git push
    ```

## Testing Patterns

- Tests live near the implementation or in crate-level `tests/` directories.
- Tests are written using Rust's built-in test framework (`#[test]`).
- Place tests in a `mod tests` block within the test file or alongside the module.
- Example:
    ```rust
    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_functionality() {
            assert_eq!(my_function(), expected_result);
        }
    }
    ```

## Commands
| Command      | Purpose                                   |
|--------------|-------------------------------------------|
| /add-module  | Scaffold and integrate a new module       |
| /run-tests   | Run all tests in the repository           |
| /commit      | Stage, commit, and push your changes      |
```
