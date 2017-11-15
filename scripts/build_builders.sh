#!/bin/bash

docker_args=""

while [[ $# -gt 0 ]]; do
	key="$1"
	case $key in
		--no-cache)
			no_cache=YES
			docker_args="$docker_args --no-cache"
			shift # past argument
		;;
		*)    # unknown option
			echo "Unknown argument $1 provided"
			shift # past argument
			exit 1
		;;
	esac
done

source ./vars.sh && \

echo "Building builders..." && \
sudo docker build --pull $docker_args -t $front_builder $(pwd)/front/docker-builder && \
sudo docker build --pull $docker_args -t $back_builder $(pwd)/back/docker-builder && \

echo "Done."
