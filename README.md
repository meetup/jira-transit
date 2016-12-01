# jira-transit [![Build Status](https://travis-ci.com/meetup/jira-transit.svg?token=jtveWukBghqdyHppHDFu&branch=master)](https://travis-ci.com/meetup/jira-transit)

A github webhook handler for transitioning jira issues. Listens on port 4567.

## usage

Intended to be run as a docker app.

```bash
$ make package
$ docker run --rm -it \
   -e RUST_LOG=info \
   -e GITHUB_SECRET=YOUR_HOOK_SECRET \
   -e SSL_CERT_FILE=/etc/ssl/certs/ca-certificates.crt \
   -e SSL_CERT_DIR=/etc/ssl/certs \
   meetup/jira-transit:0.1.{tag}
```

Meetup 2016
