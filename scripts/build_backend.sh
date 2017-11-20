#!/bin/bash

env $(cat .env | xargs) | egrep "LSYS_*|ROCKET_*" && \
source ./scripts/vars.sh && \

sudo docker run --rm -it \
	-v $(pwd)/back:/src \
	-v lsys-registry:/root/.cargo/registry \
	-v lsys-git:/root/.cargo/git \
	$back_builder \
	cargo build --release
