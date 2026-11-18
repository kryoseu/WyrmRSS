#!/bin/sh
# Restores the public demo's database to demo-snapshot.dump. Run nightly from
# cron on the demo host:
#
#   0 4 * * * /root/wyrmrss/reset-demo.sh >> /var/log/wyrm-reset.log 2>&1
#
# Take the snapshot once the demo is set up (feeds from demo-feeds.opml,
# display modes, filters), from the same directory:
#
#   docker exec wyrm-db pg_dump -U wyrm -Fc wyrm > demo-snapshot.dump
set -eu
cd /root/wyrmrss

echo "$(date -u '+%F %T') reset start"
docker compose stop wyrm-server
docker exec wyrm-db dropdb -U wyrm --if-exists --force wyrm
docker exec wyrm-db createdb -U wyrm wyrm
docker exec -i wyrm-db pg_restore -U wyrm -d wyrm --no-owner < demo-snapshot.dump
docker compose start wyrm-server
echo "$(date -u '+%F %T') reset done"
