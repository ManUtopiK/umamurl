#!/usr/bin/env bash
# Print the CHANGELOG.md section of a version, ready to be used as GitHub
# release notes. Fails if the section is missing or empty.
#
# usage: changelog-section.sh <version> [changelog]
set -euo pipefail

version=$1
changelog=${2:-CHANGELOG.md}

# Lines between "## [<version>]" and the next "## [" heading.
section=$(awk -v v="$version" '
  index($0, "## [" v "]") == 1 { found = 1; next }
  found && /^## \[/ { exit }
  found { print }
' "$changelog")

# Drop leading and trailing blank lines.
section=$(printf '%s\n' "$section" | sed -e '/./,$!d' | sed -e ':a' -e '/^\n*$/{$d;N;ba' -e '}')

if [ -z "$section" ]; then
  echo "error: no '## [$version]' section in $changelog" >&2
  exit 1
fi

# Release bodies render every newline as a line break: join the indented
# continuation lines of list items back into one line.
printf '%s\n' "$section" | perl -0pe 's/\n  (?=\S)/ /g'
