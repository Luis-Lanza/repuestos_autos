# Local Issue Triage States

Local delivery issues use this lifecycle:

```text
Draft → Approved → Done
```

## States

- `draft`: The issue is being prepared or needs clarification. It is not eligible for a delivery branch or pull request.
- `approved`: Scope, dependencies, acceptance evidence, and rollback boundary are accepted. The issue is eligible to be linked from a delivery pull request.
- `done`: The approved work is integrated and its recorded verification remains valid.

## Transitions

- `draft` → `approved`: A maintainer explicitly accepts the issue's scope and delivery evidence.
- `approved` → `done`: The linked delivery work is integrated after its recorded checks pass.

Any scope, dependency, evidence, or rollback change returns the issue to `draft` until it is approved again.
