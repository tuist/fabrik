#!/usr/bin/env -S fabrik run bash
#FABRIK input "input.txt"
#FABRIK output "output.txt"
#FABRIK cache ttl="1h"

# This script processes input.txt and generates output.txt
echo "Processing input file..."
cat input.txt | tr '[:lower:]' '[:upper:]' > output.txt
echo "Processing complete!"
