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

spec_files=(
  docs/spec/01-mission-and-principles.md
  docs/spec/02-architecture-and-domain-model.md
  docs/spec/03-targets-execution-and-isolation.md
  docs/spec/04-configuration-and-crucible-yaml.md
  docs/spec/05-campaigns-oracles-and-bug-model.md
  docs/spec/06-engines-generation-and-corpus.md
  docs/spec/07-findings-replay-and-minimization.md
  docs/spec/08-repair-verification-and-agents.md
  docs/spec/09-scheduling-storage-cli-and-reporting.md
  docs/spec/10-phases-mvp-and-acceptance.md
  docs/spec/11-runtime-operational-contracts.md
  docs/spec/12-expansion-and-completion-standard.md
)

required_files+=("${spec_files[@]}")

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

if grep -Eq '^## [0-9]+\.' crucible.md; then
  echo "specification index must not own a numbered section" >&2
  exit 1
fi

spec_first=(1 6 10 12 13 30 43 54 71 81 84 94)
spec_last=(5 9 11 12 29 42 53 70 80 83 93 97)

for index in "${!spec_files[@]}"; do
  path=${spec_files[$index]}
  first=${spec_first[$index]}
  last=${spec_last[$index]}
  mapfile -t headings < <(
    awk '/^## [0-9]+\./ { number = $2; sub(/\.$/, "", number); print number }' "$path"
  )
  expected_count=$((last - first + 1))
  if ((${#headings[@]} != expected_count)); then
    echo "specification slice $path must own sections $first through $last" >&2
    exit 1
  fi
  for offset in "${!headings[@]}"; do
    expected=$((first + offset))
    if ((${headings[$offset]} != expected)); then
      echo "specification slice $path: expected section $expected, found ${headings[$offset]}" >&2
      exit 1
    fi
  done
done

awk '
  BEGIN { expected = 1; found = 0 }
  /^## [0-9]+\./ {
    number = $2
    sub(/\.$/, "", number)
    if ((number + 0) != expected) {
      printf "top-level specification heading sequence: expected %d, found %s at %s:%d\n", expected, number, FILENAME, FNR > "/dev/stderr"
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
' "${spec_files[@]}"

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
