ci: add CI workflow, issue/PR templates, and fix Pages deployment

### PR Type

*Please check the type of this PR (you can select multiple)*

- [ ] 🐞 Bug fix
- [ ] ✨ New feature
- [ ] 📚 Documentation change
- [ ] 🎨 Code style change (such as formatting, renaming)
- [ ] ♻️ Refactoring
- [ ] ⚡️ Performance improvement
- [ ] 🧪 Adding or updating tests
- [x] 🤖 CI/CD (Changes to CI/CD configuration)
- [ ] 📦 Other changes

### Related Issue

<!--
Please fill in the issue number related to this PR below.
For example: Closes #123
If there is no related issue, please fill in "None" or "N/A".
-->

- N/A

### What does this PR do?

<!--
Please briefly describe the main purpose and specific content of this PR.
-->

- Added comprehensive CI workflow (`ci.yml`) that builds both native desktop and WASM targets
- Added issue templates: bug report, feature request, and refactor request
- Added pull request template
- Added dependabot configuration for automated dependency updates
- Added workflow for automatic dependency updates (`update-deps.yml`)
- Fixed Pages deployment to only deploy on main branch push (not on PRs)

### How to test?

<!--
If necessary, please describe in detail how the reviewer can verify your changes.
-->

- CI workflow will automatically run on pull requests and push to main
- Verify all CI jobs pass: formatting check, clippy linting, native build, WASM build, tests
- Check that Pages deployment only triggers on main branch push, not on PRs

### Screenshots or Videos

<!--
If your changes include visual changes or new features, please attach relevant screenshots or GIFs here.
-->

N/A

### Additional Information

<!--
Any additional information you think needs to be explained.
-->

This project supports both desktop and WASM targets. The CI workflow is designed to test both build targets to ensure cross-platform compatibility.
