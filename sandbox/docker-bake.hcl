group "default" {
  targets = [
    "firefox",
    "geckodriver",
    "vncserver",
    "ffmpeg",
  ]
}

variable "REPOSITORY" {
  default = "ghcr.io/hakuyume/cookieclicker"
}

variable "TAG_SUFFIX" {
  default = ""
}

variable "BASE_DEBIAN" {
  default = "docker.io/debian:bookworm"
}

target "firefox" {
  dockerfile-inline = <<-EOD
  FROM ${BASE_DEBIAN}
  RUN apt-get update && apt-get install --yes firefox-esr
  COPY firefox-policies.json /etc/firefox/policies/policies.json
  EOD

  tags = ["${REPOSITORY}:firefox${TAG_SUFFIX}"]
}

target "geckodriver" {
  dockerfile-inline = <<-EOD
  FROM ${BASE_DEBIAN}
  RUN apt-get update && apt-get install --yes ca-certificates curl
  RUN curl -L https://github.com/mozilla/geckodriver/releases/download/v0.35.0/geckodriver-v0.35.0-linux64.tar.gz | tar -xzf - -C /usr/local/bin/ --no-same-owner
  EOD

  tags = ["${REPOSITORY}:geckodriver${TAG_SUFFIX}"]
}

target "vncserver" {
  dockerfile-inline = <<-EOD
  FROM ${BASE_DEBIAN}
  RUN apt-get update && apt-get install --yes openbox tigervnc-standalone-server
  EOD

  tags = ["${REPOSITORY}:vncserver${TAG_SUFFIX}"]
}

target "ffmpeg" {
  dockerfile-inline = <<-EOD
  FROM ${BASE_DEBIAN}
  RUN apt-get update && apt-get install --yes ffmpeg
  EOD

  tags = ["${REPOSITORY}:ffmpeg${TAG_SUFFIX}"]
}
