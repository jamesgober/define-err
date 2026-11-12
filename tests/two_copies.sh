#!/usr/bin/env bash
# A build that needs two copies of define-err from 0.10 on must fail to
# resolve, naming the `links` key (docs/PLATFORM-NOTES.md, "Two copies of
# define-err"). The second copy is this crate's own manifest under another
# minor version. Run from anywhere; needs only cargo and network-free
# resolution of path dependencies.
set -eu
# `pwd -W` gives a path Cargo can read under Git Bash on Windows.
native() { (cd "$1" && { pwd -W 2>/dev/null || pwd; }); }
root=$(native "$(dirname "$0")/..")
work=$(mktemp -d)
: "${work:?}"
trap 'rm -rf "$work"' EXIT
work=$(native "$work")

mkdir -p "$work/other/src" "$work/app/src"
sed -e 's/^version *= *".*"/version = "0.99.0"/' -e '/^\[dev-dependencies\]/,$d' \
    "$root/Cargo.toml" > "$work/other/Cargo.toml"
cp "$root/build.rs" "$root/README.md" "$work/other/"
: > "$work/other/src/lib.rs"
: > "$work/app/src/lib.rs"

manifest() {
    printf '[package]\nname = "app"\nversion = "0.0.0"\nedition = "2021"\n\n[dependencies]\n%s\n\n[workspace]\n' "$1" \
        > "$work/app/Cargo.toml"
}

# One copy resolves.
manifest "define-err = { path = '$root' }"
cargo generate-lockfile --offline --manifest-path "$work/app/Cargo.toml"

# Two copies do not.
manifest "define-err = { path = '$root' }
other = { package = 'define-err', path = '$work/other' }"
if out=$(cargo generate-lockfile --offline --manifest-path "$work/app/Cargo.toml" 2>&1); then
    echo "two copies of define-err resolved; the links key is missing" >&2
    exit 1
fi
case "$out" in
    *define_err_facade*) echo "two copies refused: ok"; echo "$out" | grep -m1 -i "links" ;;
    *) echo "resolve failed for another reason:" >&2; echo "$out" >&2; exit 1 ;;
esac
