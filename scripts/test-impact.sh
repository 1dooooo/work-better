#!/bin/bash
# Determines which test gates to run based on git diff
CHANGED_FILES=$(git diff --name-only origin/main...HEAD)

if echo "$CHANGED_FILES" | grep -qE "^src-tauri/|^crates/"; then
  echo "RUST_CHANGED=true" >> $GITHUB_ENV
fi

if echo "$CHANGED_FILES" | grep -qE "^src/|^package\.json"; then
  echo "TS_CHANGED=true" >> $GITHUB_ENV
fi

if echo "$CHANGED_FILES" | grep -qE "^docs/"; then
  echo "DOCS_ONLY=true" >> $GITHUB_ENV
fi
