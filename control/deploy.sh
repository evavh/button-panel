#!/usr/bin/env bash
set -e

# Button panel compile-deploy script, needs:
#  - rust build system [cargo] (https://www.rust-lang.org/tools/install)
#  - rust cross compile tool [cross] (cargo install cargo-cross)
# please set the variables directly below

SERVER_ADDR="pi"
SERVER_USER="eva"
SERVER_DIR="/home/$SERVER_USER/button_panel"

dir=debug
if [ "$1" = "--release" ]; then
	dir=release
fi

cross build --target=armv7-unknown-linux-gnueabihf $1
rsync button_panel.service $SERVER_ADDR:/tmp/
rsync -vh --progress \
  target/armv7-unknown-linux-gnueabihf/$dir/control \
  $SERVER_ADDR:/tmp/

# sets up/updates the systemd service and places the binary
cmds="
sed -i \"s/<USER>/$SERVER_USER/g\" /tmp/button_panel.service
sed -i \"s+<DIR>+$SERVER_DIR+g\" /tmp/button_panel.service
sudo mv /tmp/button_panel.service /etc/systemd/system/

mkdir -p $SERVER_DIR
mv /tmp/control $SERVER_DIR/button_panel
chown $SERVER_USER:$SERVER_USER $SERVER_DIR/button_panel

sudo systemctl enable button_panel.service
sudo systemctl restart button_panel.service
"

ssh -t $SERVER_ADDR "$cmds"
