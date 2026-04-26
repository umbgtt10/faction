# faction

**faction** is a `no_std` + `alloc` Rust workspace for cluster readiness coordination — a startup barrier that tracks participation and readiness across a known set of peers.

It is designed for **embedded**, **distributed**, and **deterministic testing** environments.

---

## Workspace

| Crate | Description |
|---|---|
| `core/` | Core cluster readiness state machine with quorum tracking, freshness classification, and observer hooks |
| `validation/` | Deterministic multi-node scenario harness for testing readiness coordination sequences |

---

## Quality Gates

```powershell
powershell -File scripts\run_stage_1.ps1   # format, clippy, no_std checks, tests
powershell -File scripts\run_stage_2.ps1   # coverage and file risk analysis
```

---

## License

Licensed under the MIT License. See [LICENSE](./LICENSE).