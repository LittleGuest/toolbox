#!/bin/bash

set -e

script_dir="$(cd "$(dirname "$0")" && pwd)"
repo_root="$(cd "${script_dir}/../.." && pwd)"

cd "${repo_root}"
cargo b -r

cd "${script_dir}"
makepkg -fc
makepkg -f --nodeps --nocheck
