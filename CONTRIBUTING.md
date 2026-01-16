# Contributing to MeCP

First off, thank you for considering contributing to MeCP! It's people like you that make MeCP a great tool for the community.

## 🌟 Ways to Contribute

- **🐛 Bug Reports** - Report issues you encounter
- **✨ Feature Requests** - Suggest new features or improvements
- **📚 Documentation** - Improve or add documentation
- **💻 Code Contributions** - Fix bugs or implement features
- **🧪 Testing** - Add or improve test coverage
- **🎨 Design** - Enhance UI/UX of the dashboard

## 🚀 Getting Started

### Development Setup

1. **Fork the repository**
   ```bash
   # Click "Fork" on GitHub, then clone your fork
   git clone https://github.com/YOUR_USERNAME/mecp.git
   cd mecp
   ```

2. **Install dependencies**
   ```bash
   # Rust (if not already installed)
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   
   # MySQL
   sudo apt-get install mysql-server
   ```

3. **Build the project**
   ```bash
   cargo build
   ```

4. **Run tests**
   ```bash
   cargo test
   ```

5. **Start the server**
   ```bash
   cargo run --release
   ```

## 📝 Development Workflow

### 1. Create a Branch

```bash
git checkout -b feature/your-feature-name
# or
git checkout -b fix/bug-description
```

### 2. Make Your Changes

- Write clear, concise code
- Follow Rust conventions and idioms
- Add tests for new functionality
- Update documentation as needed

### 3. Test Your Changes

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_name

# Check formatting
cargo fmt --check

# Run linter
cargo clippy -- -D warnings
```

### 4. Commit Your Changes

Follow [Conventional Commits](https://www.conventionalcommits.org/):

```bash
git commit -m "feat: add new database adapter"
git commit -m "fix: resolve authentication timeout issue"
git commit -m "docs: update installation guide"
git commit -m "test: add integration tests for API"
```

**Commit Types:**
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `style`: Code style changes (formatting, etc.)
- `refactor`: Code refactoring
- `test`: Adding or updating tests
- `chore`: Maintenance tasks

### 5. Push and Create Pull Request

```bash
git push origin feature/your-feature-name
```

Then create a Pull Request on GitHub with:
- Clear title and description
- Reference to related issues (if any)
- Screenshots (for UI changes)
- Test results

## 🎯 Pull Request Guidelines

### PR Checklist

- [ ] Code follows Rust style guidelines (`cargo fmt`)
- [ ] All tests pass (`cargo test`)
- [ ] No clippy warnings (`cargo clippy`)
- [ ] New tests added for new functionality
- [ ] Documentation updated (if needed)
- [ ] Commit messages follow conventional format
- [ ] PR description clearly explains changes
- [ ] Related issues referenced

### PR Description Template

```markdown
## Description
Brief description of changes

## Type of Change
- [ ] Bug fix
- [ ] New feature
- [ ] Breaking change
- [ ] Documentation update

## Testing
How has this been tested?

## Checklist
- [ ] Tests pass
- [ ] Code formatted
- [ ] Documentation updated
```

## 🏗️ Code Style Guidelines

### Rust Style

- Follow the [Rust Style Guide](https://doc.rust-lang.org/nightly/style-guide/)
- Use `rustfmt` for automatic formatting
- Use `clippy` for linting

```bash
# Format code
cargo fmt

# Check linting
cargo clippy
```

### Project Conventions

1. **Error Handling**
   - Use `anyhow::Result` for error propagation
   - Provide context with `.context()` or `.with_context()`
   - Log errors appropriately

2. **Async Code**
   - Use `async/await` consistently
   - Prefer `tokio` runtime
   - Handle cancellation properly

3. **Documentation**
   - Document public APIs with `///` comments
   - Include examples in documentation
   - Update README for major changes

4. **Testing**
   - Write unit tests for functions
   - Write integration tests for features
   - Use descriptive test names

## 🧪 Testing Guidelines

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feature() {
        // Arrange
        let input = setup_test_data();
        
        // Act
        let result = function_to_test(input);
        
        // Assert
        assert_eq!(result, expected_output);
    }
}
```

### Integration Tests

Place integration tests in `tests/` directory:

```rust
// tests/integration_test.rs
#[tokio::test]
async fn test_api_endpoint() {
    // Test full API flow
}
```

### Running Tests

```bash
# All tests
cargo test

# Specific test
cargo test test_name

# With output
cargo test -- --nocapture

# Integration tests only
cargo test --test integration_test
```

## 📚 Documentation Standards

### Code Documentation

```rust
/// Brief description of the function
///
/// More detailed explanation if needed
///
/// # Arguments
///
/// * `param1` - Description of param1
/// * `param2` - Description of param2
///
/// # Returns
///
/// Description of return value
///
/// # Examples
///
/// ```
/// let result = my_function(arg1, arg2);
/// assert_eq!(result, expected);
/// ```
///
/// # Errors
///
/// Describe possible errors
pub fn my_function(param1: Type1, param2: Type2) -> Result<ReturnType> {
    // Implementation
}
```

### Markdown Documentation

- Use clear headings
- Include code examples
- Add table of contents for long documents
- Use proper markdown formatting

## 🐛 Bug Reports

### Before Submitting

1. Check if the bug has already been reported
2. Verify it's reproducible
3. Test with the latest version

### Bug Report Template

```markdown
## Description
Clear description of the bug

## Steps to Reproduce
1. Step 1
2. Step 2
3. Step 3

## Expected Behavior
What should happen

## Actual Behavior
What actually happens

## Environment
- OS: [e.g., Ubuntu 22.04]
- Rust version: [e.g., 1.70.0]
- MeCP version: [e.g., 0.1.0]

## Additional Context
Any other relevant information
```

## ✨ Feature Requests

### Feature Request Template

```markdown
## Problem Statement
What problem does this solve?

## Proposed Solution
Describe your solution

## Alternatives Considered
Other approaches you've thought about

## Additional Context
Any other relevant information
```

## 🔍 Code Review Process

1. **Automated Checks** - CI runs tests and linting
2. **Maintainer Review** - Code review by maintainers
3. **Discussion** - Address feedback and questions
4. **Approval** - Once approved, PR will be merged
5. **Merge** - Squash and merge to main branch

## 🎓 Resources

### Learning Rust
- [The Rust Book](https://doc.rust-lang.org/book/)
- [Rust by Example](https://doc.rust-lang.org/rust-by-example/)
- [Async Rust](https://rust-lang.github.io/async-book/)

### Project Specific
- [Architecture Guide](ARCHITECTURE.md)
- [API Documentation](API_DOCUMENTATION.md)
- [Installation Guide](INSTALLATION.md)

## 💬 Communication

- **GitHub Issues** - Bug reports and feature requests
- **GitHub Discussions** - Questions and ideas
- **Pull Requests** - Code contributions

## 📜 Code of Conduct

- Be respectful and inclusive
- Welcome newcomers
- Focus on constructive feedback
- Help others learn and grow

## 🙏 Recognition

Contributors will be:
- Listed in project credits
- Mentioned in release notes
- Appreciated by the community!

## ❓ Questions?

Feel free to:
- Open a GitHub Discussion
- Ask in Pull Request comments
- Check existing documentation

---

Thank you for contributing to MeCP! 🚀
