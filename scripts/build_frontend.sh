#!/bin/bash

env $(cat .env | xargs) | egrep "LSYS_*|ROCKET_*" && \
source ./scripts/vars.sh && \

sudo docker run --rm -it \
	-v $(pwd)/front:/src \
	-w /src \
	$front_builder \
	sh -c "npm install && npm run build"
