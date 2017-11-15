#!/bin/bash

env $(cat .env | xargs) | egrep "LSYS_*|ROCKET_*" && \
source ./vars.sh && \

echo "Building frontend..." && \
sudo docker run --rm -it \
	-v $(pwd)/front:/src \
	-w /src \
	$front_builder \
	sh -c "npm install && npm run build" && \

echo "" && \
echo "Building backend..." && \
sudo docker run --rm -it \
	-v $(pwd)/back:/src \
	-v lsys-registry:/root/.cargo/registry \
	-v lsys-git:/root/.cargo/git \
	$back_builder \
	cargo build --release && \

echo "" && \
echo "Building images..." && \
sudo docker-compose build --pull && \

echo "Done."
