# Catalog Redesign PR Chain

Closes #188.

This draft tracker coordinates the dependent review slices for the Catalog redesign. It is not merged until every child pull request is reviewed and integrated in order.

```text
master
└── feat/catalog-redesign-tracker (this tracker)
    └── catalog-01-app
        └── catalog-02-db
            └── catalog-03-commands
                └── catalog-04-db-tests
                    └── catalog-05-backup
                        └── catalog-06-browser
                            └── catalog-07-editor
                                └── catalog-08-screen (size exception: ~537 lines)
                                    └── catalog-09-shell
```

## Review order

1. Catalog application model and repository contracts.
2. SQLite schema and persistence.
3. Rust and TypeScript command contracts.
4. SQLite and migration tests.
5. Backup/restore and desktop wiring.
6. Catalog product browser presentation.
7. Editor flow and confirmation dialog.
8. Catalog screen integration and mounted tests.
9. App-shell integration.

Each child PR links #188, carries `type:feature`, targets its immediate predecessor, and documents its dependency in its PR body.
