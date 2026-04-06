#!/bin/bash

# A script to extract changed files and diff content for local unstaged changes.

#
# EXAMPLE:
# ./scripts/local_changes.sh -d -n &> out.txt
#

# --- Script State ---
SHOW_DIFF=false
NO_COLOR=false
INCLUDE_STAGED=false
INCLUDE_UNTRACKED=false

# --- Colors for output ---
COLOR_GREEN='\033[0;32m'
COLOR_YELLOW='\033[0;33m'
COLOR_BLUE='\033[0;34m'
COLOR_RESET='\033[0m'

# --- Functions ---
print_usage() {
  echo "Usage: $0 [OPTIONS]"
  echo
  echo "Extracts a list of changed files and optionally the full diff content"
  echo "for local changes in the current git repository."
  echo
  echo "Options:"
  echo "  -d, --show-diff       Display the full diff content."
  echo "  -s, --staged          Include staged changes (git diff --cached)."
  echo "  -u, --untracked       Include untracked files."
  echo "  -n, --no-color        Disable all colorized output."
  echo "  -h, --help            Display this help message."
  echo
  echo "Examples:"
  echo "  $0"
  echo "  $0 -d"
  echo "  $0 -s -u -d"
}

# --- Script Start ---

# 1. Parse Options
while [[ $# -gt 0 ]]; do
  key="$1"
  case $key in
    -h|--help)
      print_usage
      exit 0
      ;;
    -d|--show-diff)
      SHOW_DIFF=true
      shift
      ;;
    -s|--staged)
      INCLUDE_STAGED=true
      shift
      ;;
    -u|--untracked)
      INCLUDE_UNTRACKED=true
      shift
      ;;
    -n|--no-color)
      NO_COLOR=true
      shift
      ;;
    *)
      echo "Unknown option: $key"
      print_usage
      exit 1
      ;;
  esac
done

# If --no-color is set, overwrite color variables with empty strings.
if [ "$NO_COLOR" = true ]; then
  COLOR_GREEN=''
  COLOR_YELLOW=''
  COLOR_BLUE=''
  COLOR_RESET=''
fi

if ! git rev-parse --is-inside-work-tree > /dev/null 2>&1; then
    echo "Error: Not inside a git repository." >&2
    exit 1
fi

echo -e "${COLOR_BLUE}--- Analyzing Local Changes ---${COLOR_RESET}"

# 2. List Changed Files (Unstaged)
echo -e "${COLOR_YELLOW}=== Unstaged Changed Files ===${COLOR_RESET}"
git diff --name-status

if [ "$INCLUDE_STAGED" = true ]; then
  echo
  echo -e "${COLOR_YELLOW}=== Staged Changed Files ===${COLOR_RESET}"
  git diff --cached --name-status
fi

if [ "$INCLUDE_UNTRACKED" = true ]; then
  echo
  echo -e "${COLOR_YELLOW}=== Untracked Files ===${COLOR_RESET}"
  git ls-files --others --exclude-standard
fi

echo

# 3. Show Diff Content if requested
if [ "$SHOW_DIFF" = true ]; then
  GIT_COLOR_FLAG="--color=always"
  if [ "$NO_COLOR" = true ]; then
    GIT_COLOR_FLAG="--color=never"
  fi

  echo -e "${COLOR_YELLOW}=== Unstaged Diff Content ===${COLOR_RESET}"
  git diff "$GIT_COLOR_FLAG"
  echo

  if [ "$INCLUDE_STAGED" = true ]; then
    echo -e "${COLOR_YELLOW}=== Staged Diff Content ===${COLOR_RESET}"
    git diff --cached "$GIT_COLOR_FLAG"
    echo
  fi
  
  if [ "$INCLUDE_UNTRACKED" = true ]; then
    # Git diff doesn't show untracked files by default unless they are added to index
    # But we can show their content using a loop or git add -N trick if we wanted to be fancy.
    # For now, just listing them is usually enough for "unstaged local changes".
    UNTRACKED_FILES=$(git ls-files --others --exclude-standard)
    if [ -n "$UNTRACKED_FILES" ]; then
        echo -e "${COLOR_YELLOW}=== Untracked Files Content ===${COLOR_RESET}"
        for f in $UNTRACKED_FILES; do
            if [ -f "$f" ]; then
                echo -e "${COLOR_GREEN}File: $f${COLOR_RESET}"
                cat "$f"
                echo
            fi
        done
    fi
  fi
fi

echo -e "${COLOR_BLUE}--- Analysis complete. ---${COLOR_RESET}"
