#!/usr/bin/env -S fabrik run bash
#FABRIK depends "./build.sh" use-outputs=true
#FABRIK input "build/"
#FABRIK output "deploy.log"
#FABRIK env "DEPLOY_ENV"

echo "Deploying to ${DEPLOY_ENV:-production}..."
echo "Deploy started at: $(date)" > deploy.log

# Check if build exists
if [ -d "build" ]; then
    echo "Build directory found" >> deploy.log
    ls -la build/ >> deploy.log
    echo "Deployment successful!" >> deploy.log
else
    echo "ERROR: Build directory not found!" >> deploy.log
    exit 1
fi
