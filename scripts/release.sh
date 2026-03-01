#!/usr/bin/env bash
set -euo pipefail

# release.sh — bump version, commit, tag, and prompt to push.
#
# Usage:
#   ./scripts/release.sh <version>
#
# Example:
#   ./scripts/release.sh 1.2.0

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
CARGO_TOML="$REPO_ROOT/Cargo.toml"

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
RESET='\033[0m'

info()    { echo -e "${CYAN}[release]${RESET} $*"; }
success() { echo -e "${GREEN}[release]${RESET} $*"; }
warn()    { echo -e "${YELLOW}[release]${RESET} $*"; }
error()   { echo -e "${RED}[release] error:${RESET} $*" >&2; }
die()     { error "$*"; exit 1; }

if [ $# -ne 1 ]; then
    die "expected exactly one argument: <version> (e.g. 1.2.0)"
fi

NEW_VERSION="$1"

if ! echo "$NEW_VERSION" | grep -qE '^[0-9]+\.[0-9]+\.[0-9]+$'; then
    die "version '$NEW_VERSION' is not valid semver (expected X.Y.Z, e.g. 1.2.0)"
fi

NEW_TAG="v${NEW_VERSION}"

cd "$REPO_ROOT"

if ! git diff --quiet || ! git diff --cached --quiet; then
    die "working tree has uncommitted changes — commit or stash them first"
fi

CURRENT_VERSION=$(grep '^version' "$CARGO_TOML" | head -1 | sed 's/version = "\(.*\)"/\1/')
TAG_EXISTS=false
if git rev-parse "$NEW_TAG" >/dev/null 2>&1; then
    TAG_EXISTS=true
fi

info "current version : $CURRENT_VERSION"
info "new version     : $NEW_VERSION"
info "tag             : $NEW_TAG"
echo

if [ "$CURRENT_VERSION" = "$NEW_VERSION" ] && [ "$TAG_EXISTS" = true ]; then
    warn "version $NEW_VERSION is already set in Cargo.toml and tag $NEW_TAG already exists."
    warn "nothing to do."
    exit 0
fi

if [ "$CURRENT_VERSION" != "$NEW_VERSION" ] && [ "$TAG_EXISTS" = true ]; then
    die "tag '$NEW_TAG' already exists but Cargo.toml has version '$CURRENT_VERSION' — resolve the conflict manually"
fi

if [ "$CURRENT_VERSION" = "$NEW_VERSION" ]; then
    info "version is already $NEW_VERSION — skipping Cargo.toml update and commit"
else
    info "updating Cargo.toml..."

    # awk replaces only the first bare `version = "..."` line so that
    # dependency version strings are not affected. BSD sed on macOS lacks
    # the GNU sed 0,/pattern/ address-range extension, so awk is used instead.
    awk -v old="version = \"${CURRENT_VERSION}\"" \
        -v new="version = \"${NEW_VERSION}\"" \
        'done { print; next } $0 == old { print new; done=1; next } { print }' \
        "$CARGO_TOML" > "${CARGO_TOML}.tmp" && mv "${CARGO_TOML}.tmp" "$CARGO_TOML"

    UPDATED_VERSION=$(grep '^version' "$CARGO_TOML" | head -1 | sed 's/version = "\(.*\)"/\1/')
    if [ "$UPDATED_VERSION" != "$NEW_VERSION" ]; then
        die "failed to update Cargo.toml — please update it manually"
    fi

    success "Cargo.toml updated"
fi

info "running cargo check..."
cargo check --quiet
success "cargo check passed"

info "running cargo test..."
cargo test --quiet
success "all tests passed"

info "running cargo clippy..."
cargo clippy --quiet -- -D warnings
success "clippy clean"

if [ "$CURRENT_VERSION" != "$NEW_VERSION" ]; then
    info "committing..."
    git add Cargo.toml Cargo.lock
    git commit -m "chore: release ${NEW_TAG}"
    success "committed: chore: release ${NEW_TAG}"
fi

info "creating tag ${NEW_TAG}..."
git tag -a "$NEW_TAG" -m "Release ${NEW_TAG}"
success "tag ${NEW_TAG} created"

echo
success "release ${NEW_TAG} is ready locally."
warn "nothing has been pushed yet. When you're ready, run:"
echo
echo -e "    ${CYAN}git push origin main && git push origin ${NEW_TAG}${RESET}"
echo