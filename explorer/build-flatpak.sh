#!/bin/sh

BUILD_DIR=./flatpak-build
PAK_DIR="${BUILD_DIR}/explorer"
REPO_DIR="${BUILD_DIR}/repo"
ARTIFACT="${BUILD_DIR}/explorer.flatpak"
IDENT="org.repositive.EventStoreExplorer"
MANIFEST="${IDENT}.json"

set -e

flatpak install flathub org.gnome.Platform//3.26 org.gnome.Sdk//3.26

cargo build --release

flatpak-builder "$PAK_DIR" "$MANIFEST" --force-clean --repo "$REPO_DIR"

flatpak build-bundle "$REPO_DIR" "$ARTIFACT" "$IDENT" --runtime-repo=https://flathub.org/repo/flathub.flatpakrepo
