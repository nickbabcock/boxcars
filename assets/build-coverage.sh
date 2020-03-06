#!/bin/bash

# This script is used to find lines in src/data.rs that either don't have a
# corresponding test case or are unused. In a perfect world, code coverage
# would inform us that a certain key was never accessed, so this is a poor
# man's code coverage script.

DIR=$(dirname "$0")
cd "$DIR/.." || exit

function finish {
    git checkout src/data.rs
}
trap finish EXIT

LINES=()
ENTRIES=$(grep -n -e '=>' -e '), ' src/data.rs | cut -f1 -d:)
NUM_ENTRIES=$(echo "$ENTRIES" | wc -l)
echo "detected $NUM_ENTRIES lines to test"
while read -r ln; do
    sed -i "${ln}d" src/data.rs
    if cargo test >/dev/null 2>/dev/null; then
        echo "$ln is not necessary: $(sed -n "${ln}p" src/data.rs)"
        LINES+=("$ln")
    else
        echo "$ln is necessary"
    fi

    git checkout src/data.rs
done <<< "$ENTRIES";

if [[ "${#LINES[@]}" -ne 0 ]]; then
    echo "Lines that are not necessary: ${#LINES[@]} / $NUM_ENTRIES"
    for ln in "${LINES[@]}"; do
        echo "$ln is not necessary: $(sed -n ${ln}p src/data.rs)"
    done
    exit 1
else
    echo "All lines are necessary!"
fi
