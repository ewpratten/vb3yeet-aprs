#! /bin/bash
set -ex

# APRS inbound packet routing
echo "Starting APRS inbound packet routing"
socat UDP4-RECVFROM:10152,fork UDP4-SENDTO:$(APRS_UDP_SENDTO_ADDR):10152

# Web proxy connection routing
echo "Starting Web proxy connection routing"
socat UDP4-RECVFROM:80,fork UDP4-SENDTO:$(WEB_PROXY_UDP_SENDTO_ADDR):80
socat UDP4-RECVFROM:443,fork UDP4-SENDTO:$(WEB_PROXY_UDP_SENDTO_ADDR):443
