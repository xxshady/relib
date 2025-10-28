#!/bin/bash

# A script to extract commit messages, changed files, and diff content
# from a GitHub/GitLab Pull Request, with options for controlling output.

# --- Configuration ---
DEFAULT_REMOTE="origin"
DEFAULT_TARGET_BRANCH="main"
GIT_HOST_TYPE="github" # Set to "gitlab" for GitLab Merge Requests

# --- Script State ---
SHOW_DIFF=false
NO_COLOR=false

# --- Colors for output ---
# These will be overridden if --no-color is used
COLOR_GREEN='\033[0;32m'
COLOR_YELLOW='\033[0;33m'
COLOR_BLUE='\033[0;34m'
COLOR_RESET='\033[0m'

# --- Functions ---
print_usage() {
  echo "Usage: $0 [OPTIONS] <PR_NUMBER> [TARGET_BRANCH]"
  echo
  echo "Extracts commit messages, a list of changed files, and optionally the full"
  echo "diff content for a specific Pull Request."
  echo
  echo "Arguments:"
  echo "  PR_NUMBER      The number of the Pull/Merge Request (required)."
  echo "  TARGET_BRANCH  The branch the PR is targeting (optional, defaults to '$DEFAULT_TARGET_BRANCH')."
  echo
  echo "Options:"
  echo "  -d, --show-diff  Display the full diff content of all changed files."
  echo "  -n, --no-color   Disable all colorized output."
  echo "  -h, --help       Display this help message."
  echo
  echo "Examples:"
  echo "  $0 42"
  echo "  $0 -d 123 develop"
  echo "  $0 --no-color 42"
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
      shift # past argument
      ;;
    -n|--no-color)
      NO_COLOR=true
      shift # past argument
      ;;
    *)
      # This is not an option, so it must be a positional argument.
      break
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

# 2. Input Validation for Positional Arguments
if ! [[ $1 =~ ^[0-9]+$ ]]; then
  echo "Error: PR_NUMBER must be a positive integer." >&2
  echo
  print_usage
  exit 1
fi

if ! git rev-parse --is-inside-work-tree > /dev/null 2>&1; then
    echo "Error: Not inside a git repository." >&2
    exit 1
fi

PR_NUMBER=$1
TARGET_BRANCH=${2:-$DEFAULT_TARGET_BRANCH}
REMOTE=$DEFAULT_REMOTE

echo -e "${COLOR_BLUE}--- Analyzing PR #${PR_NUMBER} against branch '${TARGET_BRANCH}' ---${COLOR_RESET}"

# 3. Fetch PR and Target Branch data from remote
PR_LOCAL_REF="pr/${PR_NUMBER}"

if [[ "$GIT_HOST_TYPE" == "gitlab" ]]; then
    REF_SPEC="merge-requests/${PR_NUMBER}/head"
else # Default to GitHub
    REF_SPEC="pull/${PR_NUMBER}/head"
fi

echo "Fetching latest data from remote '$REMOTE'..."
git fetch "$REMOTE" "$TARGET_BRANCH" > /dev/null 2>&1
if ! git fetch "$REMOTE" "${REF_SPEC}:${PR_LOCAL_REF}" > /dev/null 2>&1; then
    echo "Error: Failed to fetch PR #${PR_NUMBER}. Does it exist on remote '$REMOTE'?" >&2
    exit 1
fi

# 4. Find the common ancestor
MERGE_BASE=$(git merge-base "refs/remotes/${REMOTE}/${TARGET_BRANCH}" "refs/heads/${PR_LOCAL_REF}")

if [ -z "$MERGE_BASE" ]; then
    echo "Error: Could not find a common ancestor between '${PR_LOCAL_REF}' and '${TARGET_BRANCH}'." >&2
    git branch -D "$PR_LOCAL_REF" > /dev/null 2>&1 # Cleanup
    exit 1
fi

echo -e "Found merge-base: ${MERGE_BASE:0:12}"
echo

# 5. List Commit Messages
echo -e "${COLOR_YELLOW}=== Commits in PR #${PR_NUMBER} ===${COLOR_RESET}"
# The color variables in the format string will be empty if --no-color was used.
git log --pretty="format:${COLOR_GREEN}%h${COLOR_RESET} - %s ${COLOR_BLUE}(%an)${COLOR_RESET}" "$MERGE_BASE..$PR_LOCAL_REF"

echo
echo

# 6. List Changed Files
echo -e "${COLOR_YELLOW}=== Changed Files in PR #${PR_NUMBER} ===${COLOR_RESET}"
git diff --name-status "$MERGE_BASE" "$PR_LOCAL_REF"

echo

# 7. Show Diff Content if requested
if [ "$SHOW_DIFF" = true ]; then
  echo -e "${COLOR_YELLOW}=== Diff Content in PR #${PR_NUMBER} ===${COLOR_RESET}"
  
  GIT_COLOR_FLAG="--color=always"
  if [ "$NO_COLOR" = true ]; then
    GIT_COLOR_FLAG="--color=never"
  fi
  
  # Use the flag to control git's own color output
  git diff "$GIT_COLOR_FLAG" "$MERGE_BASE" "$PR_LOCAL_REF"
  echo
fi

# 8. Cleanup the temporary local reference
git branch -D "$PR_LOCAL_REF" > /dev/null 2>&1
echo -e "${COLOR_BLUE}--- Analysis complete. ---${COLOR_RESET}"
