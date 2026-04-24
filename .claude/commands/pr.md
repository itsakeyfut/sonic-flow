---
description: Commit changes and create PR
allowed-tools: ["bash", "read", "grep"]
argument-hint: "[file1] [file2] ..."
---

Complete the implementation workflow:

**Steps:**

1. **Check current status:**
   ```bash
   git status
   git diff --stat
   ```

2. **MANDATORY: Verify and move to working branch:**

   **CRITICAL**: NEVER commit directly to `main` branch!

   ```bash
   git branch --show-current
   ```

   **If currently on `main`:**
   - STOP and create/switch to a feature branch first

   **Branch naming convention:**
   - `feat/<description>` for features
   - `fix/<description>` for bug fixes
   - `refactor/<description>` for refactoring
   - `docs/<description>` for documentation
   - `chore/<description>` for tooling/build

3. **Run quality checks:**
   ```bash
   just fmt
   just clippy
   just test
   just build
   ```

4. **Stage and commit changes:**

   **File selection:**
   - If specific files were provided as arguments: `$ARGUMENTS`
     -> Use: `git add $ARGUMENTS`
   - If no arguments were provided:
     -> Use: `git add .`

   **Commit guidelines:**
   - Create logical, atomic commits
   - Follow conventional commits format: `<type>(<scope>): <description>`
   - Reference issue numbers with "Closes #XXX"
   - Write commit message in English

   **Commit Scopes:**
   - `audio`: Audio processing, decoding, FFT (sonic-core)
   - `shader`: WGSL shaders, wgpu pipeline (sonic-shader)
   - `visualizer`: Visualizer logic and plugins (sonic-visualizer)
   - `ui`: Slint UI components (app/ui)
   - `config`: Configuration system
   - `app`: Application entry point
   - `docs`: Documentation
   - `chore`: Build/tooling

5. **Push changes:**
   ```bash
   git push -u origin <branch-name>
   ```

6. **Create PR using gh command:**
   ```bash
   gh pr create --title "..." --body "..." --assignee itsakeyfut
   ```

**PR Guidelines:**

**MANDATORY: Write PR in Japanese**

**MANDATORY PR Body Limit: MAXIMUM 100 LINES**

**PR Title:**
- Follow conventional commits format in English
- Include scope if applicable
- Example: `feat(shader): implement spectrum bars WGSL visualizer`
- Example: `fix(audio): resolve FFT buffer underrun`

**PR Body Template:**
```markdown
## 概要

[このPRが何をするのか・なぜ変更するのかを1〜4文で記述]

## 変更内容

- [変更点1]
- [変更点2]
- [変更点3]

## 関連 Issue

Closes #XXX

## テスト計画

- [ ] `cargo test --workspace` 通過
- [ ] `cargo clippy --workspace -- -D warnings` 通過
- [ ] `cargo fmt --all -- --check` 通過
```

Please proceed with these steps.
