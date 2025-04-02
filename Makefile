DOCKER_TAG ?= rcore-tutorial-v3:latest
.PHONY: docker build_docker
	
docker:
	docker run --rm -it --privileged --network=host -e http_proxy=http://127.0.0.1:10808 -e https_proxy=http://127.0.0.1:10808 -v .:/os -w /os/os os-image:latest bash

build_docker: 
	docker build -t ${DOCKER_TAG} --target build .

fmt:
	cd os ; cargo fmt;  cd ..
