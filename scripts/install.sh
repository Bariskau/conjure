#!/usr/bin/env sh
set -eu

repo="${CONJURE_REPO:-Bariskau/conjure}"
version="${CONJURE_VERSION:-latest}"
bin_dir="${CONJURE_BIN_DIR:-$HOME/.local/bin}"

cleanup() {
  rm -rf "$tmp_dir"
}

detect_target() {
  os="$(uname -s)"
  arch="$(uname -m)"

  case "$os:$arch" in
    Darwin:arm64 | Darwin:aarch64)
      printf '%s\n' "aarch64-apple-darwin"
      ;;
    Darwin:x86_64)
      printf '%s\n' "x86_64-apple-darwin"
      ;;
    Linux:x86_64 | Linux:amd64)
      printf '%s\n' "x86_64-unknown-linux-gnu"
      ;;
    *)
      printf '%s\n' "Unsupported platform: $os $arch" >&2
      exit 1
      ;;
  esac
}

release_base_url() {
  release_repo="$1"
  release_version="$2"

  if [ "$release_version" = "latest" ]; then
    printf 'https://github.com/%s/releases/latest/download\n' "$release_repo"
  else
    printf 'https://github.com/%s/releases/download/%s\n' "$release_repo" "$release_version"
  fi
}

frontend_data_dir() {
  if [ -n "${CONJURE_DATA_DIR:-}" ]; then
    printf '%s\n' "$CONJURE_DATA_DIR"
    return
  fi

  case "$(uname -s)" in
    Darwin)
      printf '%s\n' "$HOME/Library/Application Support/Conjure/frontend"
      ;;
    *)
      printf '%s\n' "${XDG_DATA_HOME:-$HOME/.local/share}/conjure/frontend"
    ;;
  esac
}

target="$(detect_target)"
asset="conjure-$target.tar.gz"
base_url="$(release_base_url "$repo" "$version")"
tmp_dir="$(mktemp -d)"

trap cleanup EXIT INT TERM

archive="$tmp_dir/$asset"
package_dir="$tmp_dir/conjure-$target"
data_dir="$(frontend_data_dir)"
binary_tmp="$tmp_dir/conjure-bin"

curl -fL "$base_url/$asset" -o "$archive"
tar -xzf "$archive" -C "$tmp_dir"

mkdir -p "$bin_dir"
cp "$package_dir/bin/conjure" "$binary_tmp"
chmod +x "$binary_tmp"
if command -v xattr >/dev/null 2>&1; then
  xattr -cr "$binary_tmp" 2>/dev/null || true
fi
mv -f "$binary_tmp" "$bin_dir/conjure"

rm -rf "$data_dir"
mkdir -p "$data_dir"
cp -R "$package_dir/frontend/." "$data_dir/"

if command -v xattr >/dev/null 2>&1; then
  xattr -cr "$data_dir" 2>/dev/null || true
fi

printf 'Installed Conjure to %s\n' "$bin_dir/conjure"
printf 'Installed UI assets to %s\n' "$data_dir"
printf 'Run: conjure\n'

case ":$PATH:" in
  *":$bin_dir:"*) ;;
  *)
    printf '\nAdd this to your shell profile if needed:\n'
    printf '  export PATH="%s:$PATH"\n' "$bin_dir"
    ;;
esac
