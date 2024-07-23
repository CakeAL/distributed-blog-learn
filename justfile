default:
    @just --list

run-category:
    cargo run --package category-srv

run-topic:
    cargo run --package topic-srv

run-frontend:
    cargo run --package blog-frontend