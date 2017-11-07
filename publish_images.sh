image_base="registry.gitlab.com/gachapen/lsys-pairwise"
version_tag=$(git tag -l --sort=v:refname "v*" | egrep "v[0-9]+\.[0-9]+\.[0-9]+" | tail -n 1)
version=$(echo $version_tag | sed 's/v//g')
services=$@

echo -n "Tag services $services with version $version? (y/N): "
read tag_input

if [ "$tag_input" = "y" ]; then
	tag=true
else
	tag=false
fi

if [ $tag = true ]; then
	for service in $services; do
		sudo docker tag $image_base/$service:latest $image_base/$service:$version
	done
fi

sudo docker push $image_base/frontend-builder:latest
sudo docker push $image_base/backend-builder:latest

sudo LSYS_VERSION=latest docker-compose push

if [ $tag = true ]; then
	sudo LSYS_VERSION=$version docker-compose push
fi
