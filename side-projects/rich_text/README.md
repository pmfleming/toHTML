# rich_text

`rich_text` is a small repository-owned egui editor surface used by the
`html-editor` side project.

## Code Ownership

Prefer repository-owned code. External dependencies should be the exception,
not the normal design habit.

When any part of this project is based on code from a crate or reference
project, bring the smallest useful subset of that source into local modules in
this repository, then adapt it to this project.

Copied or reference-derived code must be:

- reduced to the smallest useful subset
- attributed where required
- adapted behind local project interfaces
- reviewed for complexity and maintainability

