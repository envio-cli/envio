#!/bin/bash

set -e

if [ $# -ne 1 ]; then
    echo "Usage: $0 <version>"
    echo "Example: $0 0.0.0"
    exit 1
fi

VERSION=$1

if git rev-parse "v$VERSION" >/dev/null 2>&1; then
    echo "Error: Tag v$VERSION already exists"
    exit 1
fi

echo "Creating tag v$VERSION..."
git tag -a "v$VERSION" -m "chore(release): v$VERSION"

echo "Pushing tag to origin..."
git push origin "v$VERSION"

echo "Successfully created and pushed release v$VERSION"