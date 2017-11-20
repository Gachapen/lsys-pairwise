#!/bin/bash

env $(cat .env | xargs) | egrep "LSYS_*|ROCKET_*" && \
source ./scripts/vars.sh && \

echo "Building frontend..." && \
./scripts/build_frontend.sh && \
echo "" && \

echo "Building backend..." && \
./scripts/build_backend.sh && \
echo "" && \

echo "Building images..." && \
sudo docker-compose build --pull && \
echo "Done."
