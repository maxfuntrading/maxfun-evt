.PHONY: build run

IMAGE_NAME=maxfun-evt
IMAGE_VERSION=v1

build:
	docker build --build-arg ENV_FILE=.env -t ${IMAGE_NAME}:${IMAGE_VERSION} -f Dockerfile .

run:
	if [ $$(docker ps -aq --filter name=^/$(IMAGE_NAME)$$) ]; then docker stop ${IMAGE_NAME} && docker rm -f ${IMAGE_NAME};fi
	docker run -d --name ${IMAGE_NAME} -v /var/log/${IMAGE_NAME}:/app/log --restart=always --net=host ${IMAGE_NAME}:${IMAGE_VERSION}