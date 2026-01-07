# Contributing to MusterHub 🛡️

Thank you for your interest in MusterHub! To maintain high code quality and security, please follow
these guidelines.

## 🛡️ The "No Unsafe" Policy

This project strictly forbids the use of `unsafe` code.

- Pull Requests containing `unsafe` blocks will be automatically rejected unless there is an
  extraordinary architectural requirement discussed beforehand.

## 🛠️ Development Workflow

We use `cargo xtask` to manage the project. Avoid using raw `cargo` commands for infrastructure
tasks.

1. **Setup**: `cargo xtask setup`
2. **Develop**: Use `cargo xtask dev up` to start local infrastructure (Databases/Vault).
3. **Verify**: Before pushing, run `cargo format`, `cargo lint`
   and `cargo xtask test`/`cargo xtask doctest` (use `--project <crate>` to target a crate).
4. **Profile**: If you are optimizing, use `cargo xtask profiling --project <NAME>`.

## 📝 Commit Messages

We follow [Conventional Commits](https://www.conventionalcommits.org/):

- `feat:` for new features.
- `fix:` for bug fixes.
- `refactor:` for code changes that neither fix a bug nor add a feature.

## ⚖️ Licensing

By contributing, you agree that your contributions will be dual-licensed under the **MIT** and *
*Apache-2.0** licenses.
