#!/bin/zsh
 
if [[ -e /tmp/aurora-daemon.pid ]]; then
  echo "killing daemon"
  kill $(cat /tmp/aurora-daemon.pid)
fi

cargo rr -p aurora-daemon
cargo rr -p aurora-player
