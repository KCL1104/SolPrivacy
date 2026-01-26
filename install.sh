#!/bin/sh
set -e

# SolPrivacy CLI Installer
# Usage: curl -fsSL https://.../install.sh | sh

GITHUB_REPO="user/solprivacy-cli"
BINARY_NAME="solprivacy"

get_latest_release() {
  curl --silent "https://api.github.com/repos/$GITHUB_REPO/releases/latest" | \
    grep '"tag_name":' | \
    sed -E 's/.*"([^"]+)".*/\1/'
}

download_binary() {
  local version=$1
  local os=$(uname -s | tr '[:upper:]' '[:lower:]')
  local arch=$(uname -m)
  
  case "$arch" in
    x86_64) arch="x86_64" ;;
    aarch64|arm64) arch="aarch64" ;;
    *) echo "Unsupported architecture: $arch"; exit 1 ;;
  esac

  case "$os" in
    linux) target="$arch-unknown-linux-gnu" ;;
    darwin) target="$arch-apple-darwin" ;;
    *) echo "Unsupported OS: $os"; exit 1 ;;
  esac

  local asset_name="$BINARY_NAME-$version-$target.tar.gz" # Adjust extension based on cargo-dist config
  local download_url="https://github.com/$GITHUB_REPO/releases/download/$version/$asset_name"

  echo "Downloading $BINARY_NAME $version for $os-$arch..."
  
  # create temp dir
  tmp_dir=$(mktemp -d)
  curl -fsSL "$download_url" -o "$tmp_dir/$asset_name"
  
  # unpack
  tar -xzf "$tmp_dir/$asset_name" -C "$tmp_dir"
  
  # install
  if [ -w "/usr/local/bin" ]; then
    mv "$tmp_dir/$BINARY_NAME" "/usr/local/bin/"
    echo "Installed to /usr/local/bin/$BINARY_NAME"
  else
    # Fallback to current dir or user bin
    local install_dir="$HOME/.local/bin"
    mkdir -p "$install_dir"
    mv "$tmp_dir/$BINARY_NAME" "$install_dir/"
    echo "Installed to $install_dir/$BINARY_NAME"
    echo "Please add $install_dir to your PATH"
  fi
  
  rm -rf "$tmp_dir"
}

VERSION=$(get_latest_release)
if [ -z "$VERSION" ]; then
  echo "Could not fetch latest release version."
  exit 1
fi

download_binary "$VERSION"

echo "Success! Run '$BINARY_NAME --help' to get started."
