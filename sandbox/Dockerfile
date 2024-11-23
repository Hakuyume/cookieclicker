ARG USER=user
ARG UID=1000

FROM docker.io/debian:bookworm AS firefox

RUN apt-get update && apt-get install --yes firefox-esr

COPY policies.json /etc/firefox/policies/policies.json

ARG USER
ARG UID
RUN useradd --create-home --uid=${UID} ${USER}
USER ${USER}
ENV USER=${USER}

FROM docker.io/debian:bookworm AS vncserver

RUN apt-get update && apt-get install --yes openbox tigervnc-standalone-server

ARG USER
ARG UID
RUN useradd --create-home --uid=${UID} ${USER}
USER ${USER}
ENV USER=${USER}

FROM docker.io/debian:bookworm AS novnc

RUN apt-get update && apt-get install --yes novnc websockify
