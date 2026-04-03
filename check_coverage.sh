#!/bin/bash
set -e

echo "Running tests..."
cargo test

if command -v cargo-llvm-cov &> /dev/null; then
    echo "Generating coverage report with cargo-llvm-cov..."
    cargo llvm-cov --html
    echo "Coverage report generated in target/llvm-cov/html/index.html"
elif command -v cargo-tarpaulin &> /dev/null; then
    echo "Generating coverage report with cargo-tarpaulin..."
    cargo tarpaulin --out Html
    echo "Coverage report generated in tarpaulin-report.html"
else
    echo "No coverage tool found. Please install cargo-llvm-cov or cargo-tarpaulin."
    echo "Example: cargo install cargo-llvm-cov"
fi
