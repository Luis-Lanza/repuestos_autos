# Release and deployment skill

## Objective
Create a project skill that makes release versioning and Windows NSIS customer deployment repeatable and safe.

## Decisions
- Apply the skill to release, publish, deploy, installer-update, and customer-update requests.
- Version Tauri and Cargo together before building an installer; never move an existing tag.
- Keep the Tauri identifier stable across updates.
- Treat NSIS update smoke testing over the prior installed version as required before customer delivery.
- Do not assume Authenticode signing; follow explicit user preference.

## Tasks
- [x] S1 Create the project release/deployment skill with versioning, build, smoke, and delivery gates. Evidence: `.pi/skills/release-deployment/SKILL.md`.
- [x] S2 Refresh the local skill registry and record the new trigger path. Evidence: `.atl/skill-registry.md` indexes the project-scoped `release-deployment` skill.

## Acceptance criteria
- The skill names version files, NSIS build command, artifact path, backup/smoke steps, and release authorization boundaries.
- Future agents can discover it by common release and deployment triggers.
