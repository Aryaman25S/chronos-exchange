#!/bin/sh
set -e
mkdir -p "${DATA_DIR:-/data}"
/usr/local/bin/gateway &
# Brief delay so the first proxied request is less likely to race nginx startup
sleep 1
exec nginx -g "daemon off;"
