# Fish shell wrapper for rfind
# Add to your fish config: source /path/to/rfind.fish

function rfind
    set -l output (command rfind $argv)
    set -l code $status

    if string match -q '__RFIND_CD__:*' "$output"
        set -l dir (string replace '__RFIND_CD__:' '' "$output")
        cd "$dir"; or echo "rfind: failed to cd to $dir"
    else if test -n "$output"
        echo "$output"
    end

    return $code
end
