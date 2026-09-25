# Lessons Learnt

| Timestamp (UTC) | Error | Root cause | Fix |
| --- | --- | --- | --- |
| 2026-09-25T08:55Z | GitHub org create 404/scope | Token lacks `admin:org` | Created user repo `lfernando2703/piercast`; document org migration |
| 2026-09-25T08:56Z | Domain purchase blocked | No registrar credentials in agent env | Tokens mark `purchase_status: pending_manual_checkout` |
| 2026-09-25T09:00Z | Absolute Write path failed | Workspace Write expects relative paths | Use relative paths / shell writers |
| 2026-09-25T09:08Z | `Engine::open` name clash | Constructor + app open both named open | Constructor `Engine::new` → `Arc<Engine>`; method `open(id)` |
| 2026-09-25T09:09Z | schemars 0.8 vs rmcp 1.x | Duplicate JsonSchema traits | Workspace `schemars = "1"` + `use rmcp::schemars::JsonSchema` |
| 2026-09-25T09:11Z | E2E upsert path process cwd | Missing app root dirs | `create_dir_all` in spawn_process / upsert |
