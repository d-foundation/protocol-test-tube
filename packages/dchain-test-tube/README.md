The Go library is built automatically by build.rs when you build the Rust crate. You don't build it manually.

To build & use it:

cd packages/dchain-test-tube
cargo build

build.rs invokes go mod tidy and go build -buildmode=c-shared on libdchaintesttube/main.go, producing
libdchaintesttube.dylib (macOS) into OUT_DIR, then generates Rust FFI bindings via bindgen.

Useful env vars (build.rs:35,44):

- DCHAIN_TEST_TUBE_DEV=1 — force rebuild of the Go shared lib on every cargo build (use while iterating on main.go).
- PREBUILD_LIB=1 — also write the shared lib to libdchaintesttube/artifacts/ (for shipping prebuilt binaries).

So the typical dev loop after editing Go code:

DCHAIN_TEST_TUBE_DEV=1 cargo build -p dchain-test-tube

Or run tests directly: DCHAIN_TEST_TUBE_DEV=1 cargo test -p dchain-test-tube.
