#!/usr/bin/env -S fabrik run bash
#FABRIK env "USER" "HOME"
#FABRIK output "env-output.txt"
#FABRIK cache ttl="30m"

echo "User: $USER" > env-output.txt
echo "Home: $HOME" >> env-output.txt
echo "Generated at: $(date)" >> env-output.txt
