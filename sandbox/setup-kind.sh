#! /usr/bin/env sh
set -eux
cd $(dirname $0)

NAME=cookieclicker

if ! kind get clusters | grep ${NAME}; then
    kind create cluster --config=kind.yaml --name=${NAME}
fi

DOCKER_BAKE_METADATA=$(mktemp)
docker buildx bake --load --metadata-file=${DOCKER_BAKE_METADATA}
jq -r '.[]["image.name"]' ${DOCKER_BAKE_METADATA} |
    while read -r IMAGE_NAME
    do
        kind load docker-image --name=${NAME} ${IMAGE_NAME}
    done
