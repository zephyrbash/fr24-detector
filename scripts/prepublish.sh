#!/bin/bash

###########################################################################
# Creates a clean commit that forces github to do the publishing workflow #
###########################################################################

# Check if the current branch is main
current_branch=$(git rev-parse --abbrev-ref HEAD)
if [ "$current_branch" != "main" ]; then
    echo "Error: You must be on the 'main' branch to run this script."
    exit 1
fi
echo "✓ Branch: main"

# Check that there are no uncommitted changes
if [ -n "$(git status --porcelain)" ]; then
    echo "Error: You have uncommitted changes. Please commit or stash them before running this script."
    exit 1
fi
echo "✓ No uncommitted changes"

# Check that the working directory is in root
if [ ! -f "Cargo.toml" ]; then
    echo "Error: This script must be run from the root of the fr24-detector repository."
    exit 1
fi
echo "✓ In correct directory"

# Get the latest hash of the main branch
latest_hash=$(git rev-parse HEAD)
echo "✓ Latest hash: $latest_hash"

# Write the hash to assets/hash.txt
echo "$latest_hash" > assets/hash.txt
echo "✓ Wrote hash to assets/hash.txt"

# Make a clean commit
git add assets/hash.txt
git commit -m "chore(publishing): trigger publishing of $latest_hash"
echo "✓ Created clean commit to trigger publishing"

# Push commit
git push origin main
echo "✓ Pushed commit to main branch"
echo ""
echo "Check GitHub, something should be happening"
