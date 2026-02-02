build-client:
    cargo build -p flux -p flux-core -p desktop-app

build-server:
    cargo build -p flux_server

build-proto:
    cargo build -p proto

build: build-proto build-server build-client

server: build-server
    cargo run -p flux_server

server-cleanup:
    rm -rf crates/server/uploads

clean-build:
    rm -rf target
    just build

format:
    cargo fmt --all

test:
    cargo test --all

lint:
    cargo clippy --all-targets --all-features --all -- -D warnings

lint-strict:
    cargo clippy --all-targets --all-features --all -- -D warnings -D missing-docs -D unsafe_code -D clippy::unwrap_used -D clippy::expect_used

fix:
    cargo clippy --all-targets --all-features --all --fix --allow-dirty
    just format

deps:
    cargo update
    cargo outdated || true

desktop-dev:
    cd crates/client/desktop-app && pnpm install
    cd crates/client/desktop-app && pnpm tauri dev

push COMMIT_MSG:
    just fix
    just test
    git diff --exit-code || git add -A
    git diff --cached --exit-code || git commit -m "{{COMMIT_MSG}}"
    git push