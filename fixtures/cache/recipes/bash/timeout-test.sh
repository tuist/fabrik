#!/usr/bin/env -S fabrik run bash
#FABRIK output "timeout-output.txt"
#FABRIK exec timeout="2s"

echo "Starting long-running task..."
echo "This should timeout" > timeout-output.txt
sleep 10
echo "This should never execute" >> timeout-output.txt
