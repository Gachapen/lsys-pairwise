front_builder=registry.gitlab.com/gachapen/lsys-pairwise/frontend-builder:latest
back_builder=registry.gitlab.com/gachapen/lsys-pairwise/backend-builder:latest

env $(cat .env | xargs) | egrep "LSYS_*|ROCKET_*" && \

echo "Building builders..." && \
sudo docker build -t $front_builder $(pwd)/front/docker-builder && \
sudo docker build -t $back_builder $(pwd)/back/docker-builder && \

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
sudo docker-compose build && \

echo "Done."
