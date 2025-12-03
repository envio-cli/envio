#!/usr/bin/env bash
set -e

if [ -z "$ENVIO_VERSION" ]; then
  echo "Error: ENVIO_VERSION environment variable not set"
  exit 1
fi

VERSION="${ENVIO_VERSION#v}"

CARGO_TOML="./Cargo.toml"

if [ ! -f "$CARGO_TOML" ]; then
  echo "Error: Cargo.toml not found at $CARGO_TOML"
  exit 1
fi

echo "Updating Cargo.toml version to $VERSION"

if [[ "$(uname)" == "Darwin" ]]; then
  sed -i '' -E "s/^version = \".*\"/version = \"${VERSION}\"/" "$CARGO_TOML"
else
  sed -i -E "s/^version = \".*\"/version = \"${VERSION}\"/" "$CARGO_TOML"
fi

echo "Version update complete."
