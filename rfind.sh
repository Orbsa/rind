#!/usr/bin/env bash
# Shell wrapper for rfind - source this or add as a function to your shell rc
# Usage: rfind [args...]

rfind() {
    local output
    output=$("$(dirname "${BASH_SOURCE[0]}")/target/release/rfind" "$@")
    local exit_code=$?

    if [[ "$output" == __RFIND_CD__:* ]]; then
        local dir="${output#__RFIND_CD__:}"
        cd "$dir" || echo "rfind: failed to cd to $dir"
    elif [[ -n "$output" ]]; then
        echo "$output"
    fi

    return $exit_code
}
