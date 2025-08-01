DOCKER_TAG ?= rcore-tutorial-v3:latest
.PHONY: docker build_docker
	
docker:
	docker run -it --rm --name myos --privileged --network=host -e http_proxy=http://127.0.0.1:10808 -e https_proxy=http://127.0.0.1:10808 -v .:/os -w /os/os os-image:latest bash

# build_docker: 
# 	docker build -t ${DOCKER_TAG} --target build .

fmt:
	cd os ; cargo fmt;  cd ..

build_docker:
	docker run -it --name myos -v .:/myOS --network=host --privileged -p 1234:1234 -e http_proxy=http://127.0.0.1:7890 -e https_proxy=http://127.0.0.1:7890 os-image:latest /bin/bash

run:
	docker exec -it myos /bin/bash

all: extract
	cp ./liblwext4-loongarch64.a ./vendor/lwext4_rust/c/lwext4/
	cp ./liblwext4-riscv64.a ./vendor/lwext4_rust/c/lwext4/
	cd ./os/ && make clean && make eval ARCH=riscv64 && make eval ARCH=loongarch64

clean:
	rm kernel-la
	rm kernel-rv

doc: 
	make -C ./doc
	cp ./doc/main.pdf ./Del0n1x初赛文档.pdf

extract:
	tar -xvzf vendor.tar.gz 


.PHONY: all clean doc docker build_docker run extract 