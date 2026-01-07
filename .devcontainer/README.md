## Dev Container Notes

This dev container is tuned for RustRover Remote development and `perf` profiling.
It installs the Rust toolchain pinned to `1.93.0`, common build tools, Dioxus desktop
dependencies, and a SurrealDB sidecar service.

### Perf Requirements

`perf` needs extra kernel permissions and sysctl settings. The container is started with:

- `SYS_PTRACE`, `SYS_ADMIN`, and `PERFMON` capabilities
- `seccomp=unconfined`
- `kernel.perf_event_paranoid=1`
- `kernel.kptr_restrict=0`
- `kernel.yama.ptrace_scope=0`

If `perf` still fails on your host, verify the host kernel allows perf events and
consider relaxing `perf_event_paranoid` further.

### Services and Ports

- SurrealDB: `8000`
- tokio-console: `6669`
- mhub-server: `4583`
- flamegraph (default): `4040`

### Typical Commands

```sh
cargo test -p mhub-vault
cargo bench -p mhub-vault
perf stat -d cargo bench -p mhub-vault
perf record -F 99 --call-graph dwarf -- cargo bench -p mhub-vault
perf report
```
