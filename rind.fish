# Fish shell wrapper for rind
# Add to your fish config: source /path/to/rind.fish

function rind
    set -l output (command rind $argv)
    set -l code $status

    if string match -q '__RFIND_CD__:*' "$output"
        set -l dir (string replace '__RFIND_CD__:' '' "$output")
        cd "$dir"; or echo "rind: failed to cd to $dir"
    else if test -n "$output"
        echo "$output"
    end

    return $code
end
