#!/bin/bash

unset GITHUB_TOKEN

git checkout main
git pull

version=$(grep "^version =" Cargo.toml | sed 's/version = "\(.*\)"/\1/')

type=$(gum choose 'patch' 'minor' 'major')

# Split the version into major, minor, and patch
IFS='.' read -r -a parts <<< "$version"
major=${parts[0]}
minor=${parts[1]}
patch=${parts[2]}

# Determine which part to bump
case "$type" in
    major)
        major=$((major + 1))
        minor=0
        patch=0
        ;;
    minor)
        minor=$((minor + 1))
        patch=0
        ;;
    patch|*)
        patch=$((patch + 1))
        ;;
esac

# New version
new_version="$major.$minor.$patch"

function release() {
    sed -i.bak "s/version = \"$version\"/version = \"$new_version\"/" Cargo.toml
    rm *.bak
    echo "Version bumped to $new_version"
    cargo install --path .

    git add Cargo.toml Cargo.lock
    git commit -m "Release: $new_version"
    git push
    git tag "v$new_version"
    git push origin "v$new_version"
}

gum confirm "Release $new_version?" && release
