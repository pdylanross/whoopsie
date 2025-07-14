default:
    @just --list

fmt:
    cargo fmt

watch:
    env LOCAL_DEV="true" cargo leptos watch

watch-release:
    cargo leptos watch --release

generate-entities:
    sea-orm-cli generate entity -o entities/src --lib

dev-migration-refresh:
    cargo run --bin migration -- refresh
