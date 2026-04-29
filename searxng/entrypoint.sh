#!/bin/sh

sed -i "s/ultrasecretkeythatischangedindockerfile/${SEARXNG_SECRET_KEY}/g" /etc/searxng/settings.yml

exec /usr/local/searxng/entrypoint.sh
