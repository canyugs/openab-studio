#!/usr/bin/env sh
# POSIX shell validation for repository authoring templates; requires only sh, awk, grep, dirname, and test.

set -eu

root=$(CDPATH= cd "$(dirname "$0")/.." && pwd)
cd "$root"

fail() {
  printf '%s\n' "template validation failed: $*" >&2
  exit 1
}

require_file() {
  test -f "$1" || fail "missing $1"
}

require_literal() {
  if ! grep -F -q -e "$2" "$1"; then
    fail "$1 is missing: $2"
  fi
}

require_form_basics() {
  form=$1
  for key in name description title body; do
    if ! grep -E -q "^${key}:" "$form"; then
      fail "$form is missing top-level ${key}"
    fi
  done
}

require_unique_ids() {
  form=$1
  if ! awk '
    /^    id: / {
      id = $2
      count += 1
      if (id !~ /^[a-z][a-z0-9_-]*$/ || seen[id]++) {
        invalid = 1
      }
    }
    END { exit (count > 0 && !invalid) ? 0 : 1 }
  ' "$form"; then
    fail "$form has missing, duplicate, or invalid field IDs"
  fi
}

require_required_field() {
  form=$1
  label=$2
  expected_type=$3
  if ! awk -v label="$label" -v expected_type="$expected_type" '
    function complete() {
      return in_field && field_type == expected_type && label_found && required
    }
    /^  - type: / {
      if (complete()) {
        complete_found = 1
      }
      in_field = 1
      field_type = $0
      sub(/^  - type: /, "", field_type)
      label_found = 0
      required = 0
      next
    }
    in_field && $0 == "      label: " label {
      label_found = 1
      next
    }
    in_field && label_found && $0 == "      required: true" {
      required = 1
    }
    END {
      if (complete()) {
        complete_found = 1
      }
      exit complete_found ? 0 : 1
    }
  ' "$form"; then
    fail "$form needs required ${expected_type} field: ${label}"
  fi
}

require_nonempty_section() {
  file=$1
  heading=$2
  stop_prefix=$3
  if ! awk -v heading="$heading" -v stop_prefix="$stop_prefix" '
    $0 == heading {
      found = 1
      next
    }
    found && index($0, stop_prefix) == 1 {
      exit
    }
    found && $0 !~ /^[[:space:]]*$/ {
      content = 1
    }
    END { exit (found && content) ? 0 : 1 }
  ' "$file"; then
    fail "$file needs non-empty section: $heading"
  fi
}

check_markdown_target() {
  file=$1
  href=$2
  target=${href%%#*}
  anchor=
  case "$href" in
    *\#*) anchor=${href#*#} ;;
  esac
  require_literal "$file" "]($href)"
  resolved="$(dirname "$file")/$target"
  test -e "$resolved" || fail "$file has a broken local Markdown link: $href"
  if test -n "$anchor" && ! awk -v anchor="$anchor" '
    /^#+[[:space:]]/ {
      heading = $0
      sub(/^#+[[:space:]]*/, "", heading)
      heading = tolower(heading)
      gsub(/[^[:alnum:] -]/, "", heading)
      gsub(/[[:space:]]+/, "-", heading)
      if (heading == anchor) {
        found = 1
      }
    }
    END { exit found ? 0 : 1 }
  ' "$resolved"; then
    fail "$file has a broken local Markdown anchor: $href"
  fi
}

config=.github/ISSUE_TEMPLATE/config.yml
task_form=.github/ISSUE_TEMPLATE/task.yml
decision_form=.github/ISSUE_TEMPLATE/decision-or-spike.yml
bug_form=.github/ISSUE_TEMPLATE/bug.yml
pr_template=.github/pull_request_template.md

for file in "$config" "$task_form" "$decision_form" "$bug_form" "$pr_template"; do
  require_file "$file"
done

require_literal "$config" "blank_issues_enabled: false"
require_literal "$config" "name: Project and tracker ownership"
require_literal "$config" "url: https://github.com/canyugs/openab-studio/blob/main/docs/project-management.md"

for form in "$task_form" "$decision_form" "$bug_form"; do
  require_form_basics "$form"
  require_unique_ids "$form"
  for label in \
    "Goal" \
    "Non-goals" \
    "Dependencies" \
    "Affected contracts" \
    "Acceptance criteria" \
    "Proof" \
    "Security and platform notes"; do
    require_required_field "$form" "$label" textarea
  done
done

require_literal "$task_form" "labels: [\"type:task\"]"
require_literal "$bug_form" "labels: [\"type:bug\"]"
require_required_field "$decision_form" "Issue type" dropdown
require_literal "$decision_form" "- Decision (\`type:decision\`)"
require_literal "$decision_form" "- Spike (\`type:spike\`)"

require_literal "$pr_template" "## Review Contract"
for heading in \
  "### Goal" \
  "### Non-goals" \
  "### Accepted residual risks" \
  "### Acceptance criteria and proof" \
  "### Follow-ups"; do
  require_literal "$pr_template" "$heading"
done

for fixture in \
  docs/template-examples/task-issue.md \
  docs/template-examples/decision-or-spike-issue.md \
  docs/template-examples/bug-issue.md; do
  require_file "$fixture"
  for heading in \
    "## Goal" \
    "## Non-goals" \
    "## Dependencies" \
    "## Affected contracts" \
    "## Acceptance criteria" \
    "## Proof" \
    "## Security and platform notes"; do
    require_nonempty_section "$fixture" "$heading" "## "
  done
done

pr_fixture=docs/template-examples/pull-request.md
require_file "$pr_fixture"
require_literal "$pr_fixture" "Closes #8"
for heading in \
  "### Goal" \
  "### Non-goals" \
  "### Accepted residual risks" \
  "### Acceptance criteria and proof" \
  "### Follow-ups"; do
  require_nonempty_section "$pr_fixture" "$heading" "### "
done

for guide in CONTRIBUTING.md AGENTS.md; do
  require_file "$guide"
  check_markdown_target "$guide" "docs/adr/remote-first-client.md"
  check_markdown_target "$guide" "docs/adr/plugin-platform.md"
  check_markdown_target "$guide" "ROADMAP.md"
  check_markdown_target "$guide" "docs/workstreams.md#workstreams"
  check_markdown_target "$guide" "docs/project-management.md#canonical-records"
  check_markdown_target "$guide" "docs/project-management.md#project-fields-and-views"
done
check_markdown_target README.md "./CONTRIBUTING.md"
check_markdown_target README.md "./AGENTS.md"

printf '%s\n' "Template validation passed: issue forms, PR Review Contract, dry-run fixtures, and guidance links."
