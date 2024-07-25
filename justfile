default:
    @just --list

run-category:
    cargo run --package category-srv

run-topic:
    cargo run --package topic-srv

run-admin:
    cargo run --package admin-srv

run-frontend:
    cargo run --package blog-frontend

run-backend:
    cargo run --package blog-backend
