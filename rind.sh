#!/usr/bin/env bash
# Shell wrapper for rind - source this or add as a function to your shell rc
# Usage: rind [args...]

rind() {
    local output
    output=$("$(dirname "${BASH_SOURCE[0]}")/target/release/rind" "$@")
    local exit_code=$?

    if [[ "$output" == __RFIND_CD__:* ]]; then
        local dir="${output#__RFIND_CD__:}"
        cd "$dir" || echo "rind: failed to cd to $dir"
    elif [[ -n "$output" ]]; then
        echo "$output"
    fi

    return $exit_code
}
