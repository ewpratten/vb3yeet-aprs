#! /bin/bash
set -ex

ip a
ping -c 3 10.31.62.1

# APRS inbound packet routing
echo "Starting APRS inbound packet routing"
socat UDP4-RECVFROM:10152,fork UDP4-SENDTO:${APRS_UDP_SENDTO_ADDR}:10152 &

# Web proxy connection routing
echo "Starting Web proxy connection routing"
socat TCP4-LISTEN:80,fork TCP4:${WEB_PROXY_TCP_SENDTO_ADDR}:80 &
socat TCP4-LISTEN:443,fork TCP4:${WEB_PROXY_TCP_SENDTO_ADDR}:443

