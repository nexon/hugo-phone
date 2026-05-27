#!/bin/bash
exec /usr/bin/dosbox \
-c "mount c /opt/hugo-phone/games" \
-c "imgmount d /opt/hugo-phone/games/hugo/hugo4.iso -t iso" \
-c "d:" \
-c "INSTALL.EXE"
