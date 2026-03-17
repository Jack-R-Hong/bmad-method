# README Consistency Checklist

Run after any agent add/remove to verify README stays in sync with the registered agents.

## Agents in registry vs README table

- [ ] bmad/analyst ↔ row in README table
- [ ] bmad/architect ↔ row in README table
- [ ] bmad/bmad-master ↔ row in README table
- [ ] bmad/dev ↔ row in README table
- [ ] bmad/devops ↔ row in README table
- [ ] bmad/pm ↔ row in README table
- [ ] bmad/qa ↔ row in README table
- [ ] bmad/quick-flow ↔ row in README table
- [ ] bmad/security ↔ row in README table
- [ ] bmad/sm ↔ row in README table
- [ ] bmad/tech-writer ↔ row in README table
- [ ] bmad/ux-designer ↔ row in README table

## How to verify

1. List registered agents from the plugin test output:
   ```
   cargo test -p bmad-plugin registry::tests::list_agents_returns_sorted_alphabetical -- --nocapture
   ```
2. Compare each executor name against rows in `README.md` Agent Reference table
3. Verify each README table row maps to an existing `agents/*.md` file
4. Confirm no phantom entries (rows with no corresponding agent file)
5. Confirm no gaps (registered agents with no README row)

## When to run this checklist

- After adding a new `agents/*.md` file
- After removing or renaming an agent file
- After updating executor names in agent frontmatter
- Before any release
