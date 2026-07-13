# Test Writing Style Guide

## 1. Hierarchical Structure
Tests are organized into nested modules that mirror the structure of the implementation. Each public method has its own sub-module.
```rust
mod struct_name {
  mod method_name {
    #[test]
    fn case_name() { ... }
  }
}
```

## 2. Descriptive Naming Convention
Test functions use snake_case names that describe the specific scenario and expected outcome:
- `path_ok_[scenario]` for paths that lead to successes.
- `path_err_[scenario]` for paths that lead to failures.
- `anxiety_[scenario]` for non-path tests that are just nice to have.

## 3. Given-When-Then Pattern
Every test body is divided into three distinct phases using comments to improve readability and intent:
- **Given**: Setup of state, mock data, or temporary files.
- **When**: Execution of the specific function or logic being tested.
- **Then**: Assertions to verify the outcome, side effects, and state changes.

## 4. Error Verification
Errors are checked not just for existence, but for specific enum variants using pattern matching to ensure the correct error logic is triggered.

## 6. Literal-First Comparisons
Expected values (especially raw strings or JSON) are defined as constants or literals within the test to provide a clear "source of truth" for the comparison. `#[rustfmt::skip]` is utilized on raw strings to preserve the visual formatting of serialized data.

## 7. Data-Driven Assertions
Assertions are comprehensive, checking:
- The `Result` status (`is_ok()`/`is_err()`).
- The returned value.
- The internal state of the object (e.g., collection length, specific field values).