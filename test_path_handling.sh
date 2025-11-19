#!/bin/bash
set -e

# Create a temporary directory for the mock monorepo
TEST_DIR=$(mktemp -d)
echo "Created test directory: $TEST_DIR"

# Create the directory structure
mkdir -p "$TEST_DIR/trees/my-tree/src/.meta"
mkdir -p "$TEST_DIR/trees/my-tree/src/areas/platform"

# Create a manifest file with clients entries
cat <<EOF > "$TEST_DIR/trees/my-tree/src/.meta/manifest.json"
{
  "//areas/clients/admin-mobile": { "id": "W-1" },
  "//areas/clients/admin-web": { "id": "W-2" },
  "//areas/clients/checkout-web": { "id": "W-3" },
  "//areas/clients/customer-account-web": { "id": "W-4" },
  "//areas/clients/customer-authentication-web": { "id": "W-5" },
  "//areas/clients/customerview-mobile": { "id": "W-6" },
  "//areas/clients/finance-mobile": { "id": "W-7" },
  "//areas/clients/inbox-mobile": { "id": "W-8" },
  "//areas/clients/portable-wallets": { "id": "W-9" },
  "//areas/clients/pos-mobile": { "id": "W-10" },
  "//areas/clients/simgym": { "id": "W-11" },
  "//areas/platform/billing": { "id": "W-12" }
}
EOF

# Create only one real directory in clients
mkdir -p "$TEST_DIR/trees/my-tree/src/areas/clients"
mkdir -p "$TEST_DIR/trees/my-tree/src/areas/clients/admin-web"

CARGO_MANIFEST="$PWD/Cargo.toml"

echo "=== Test 1: Relative path (ls or wls) ==="
cd "$TEST_DIR/trees/my-tree/src/areas/clients"
echo "PWD: $(pwd)"
cargo run --quiet --manifest-path "$CARGO_MANIFEST" -- -l

echo ""
echo "=== Test 2: Absolute path (ls \$(pwd) or wls \$(pwd)) ==="
cargo run --quiet --manifest-path "$CARGO_MANIFEST" -- -l "$(pwd)"

echo ""
echo "=== Test 3: Intermediate ghost (areas should show 'clients') ==="
cd "$TEST_DIR/trees/my-tree/src/areas"
cargo run --quiet --manifest-path "$CARGO_MANIFEST" -- -l

# Cleanup
rm -rf "$TEST_DIR"
