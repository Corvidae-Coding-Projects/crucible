#!/usr/bin/env bash
set -euo pipefail

required_files=(
  README.md
  crucible.md
  LICENSE
  CODE_OF_CONDUCT.md
  CONTRIBUTING.md
  SECURITY.md
  SUPPORT.md
  GOVERNANCE.md
  ROADMAP.md
  CHANGELOG.md
  CITATION.cff
  .github/CODEOWNERS
  .github/PULL_REQUEST_TEMPLATE.md
)

for path in "${required_files[@]}"; do
  if [[ ! -s "$path" ]]; then
    echo "required repository file is missing or empty: $path" >&2
    exit 1
  fi
done

while IFS= read -r -d '' path; do
  fence_count=$(awk '/^```/{count += 1} END {print count + 0}' "$path")
  if (( fence_count % 2 != 0 )); then
    echo "unbalanced fenced code blocks: $path ($fence_count fences)" >&2
    exit 1
  fi
done < <(find . -type f -name '*.md' -not -path './.git/*' -not -path './node_modules/*' -print0)

awk '
  BEGIN { expected = 1; found = 0 }
  /^## [0-9]+\./ {
    number = $2
    sub(/\.$/, "", number)
    if ((number + 0) != expected) {
      printf "top-level specification heading sequence: expected %d, found %s at line %d\n", expected, number, NR > "/dev/stderr"
      exit 1
    }
    expected += 1
    found += 1
  }
  END {
    if (found != 97) {
      printf "expected 97 numbered top-level specification sections, found %d\n", found > "/dev/stderr"
      exit 1
    }
  }
' crucible.md

if grep -RIn $'\r' --include='*.md' --include='*.yml' --include='*.yaml' --include='*.toml' . \
  --exclude-dir=.git --exclude-dir=node_modules; then
  echo "carriage returns found in text files" >&2
  exit 1
fi

if grep -RInE '^(<<<<<<<|=======|>>>>>>>)' . --exclude-dir=.git --exclude-dir=node_modules; then
  echo "unresolved merge-conflict marker found" >&2
  exit 1
fi

mapfile -d '' yaml_files < <(
  find . -type f \( -name '*.yml' -o -name '*.yaml' \) \
    -not -path './.git/*' -not -path './node_modules/*' -print0
)

if ((${#yaml_files[@]} > 0)); then
  ruby -e '
    require "yaml"
    ARGV.each do |path|
      YAML.parse_file(path)
    rescue StandardError => error
      warn "invalid YAML syntax: #{path}: #{error.message}"
      exit 1
    end
  ' "${yaml_files[@]}"
fi

echo "repository document structure is valid"
