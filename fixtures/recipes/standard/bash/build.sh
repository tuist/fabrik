#!/usr/bin/env -S fabrik run bash
#FABRIK input "source/*.txt"
#FABRIK output "build/"
#FABRIK cache ttl="7d"

echo "Building project..."
mkdir -p build
echo "Build timestamp: $(date)" > build/manifest.txt

# Simulate build process
if [ -d "source" ]; then
    for file in source/*.txt; do
        if [ -f "$file" ]; then
            basename=$(basename "$file")
            echo "Processing $basename..." >> build/manifest.txt
            cp "$file" "build/$basename"
        fi
    done
fi

echo "Build complete!" >> build/manifest.txt
