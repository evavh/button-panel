#!/usr/bin/env bash
set -e

# Button panel compile-deploy script, needs:
#  - rust build system [cargo] (https://www.rust-lang.org/tools/install)
#  - rust cross compile tool [cross] (cargo install cargo-cross)
# please set the variables directly below

BRANCH=$(git symbolic-ref --short HEAD)
if [ $BRANCH = "dev" ]; then
	echo "Deploying dev, with previously deployed main as fallback"
	SUFFIX="_dev"
	STOP_MAIN=";sudo systemctl stop button_panel.service;"
elif [ $BRANCH = "main" ]; then
	echo "Deploying main only"
	SUFFIX=""
	STOP_MAIN=""
else
	echo "Unknown branch $BRANCH, aborting"
	exit 1
fi

SERVER_ADDR="pi"
SERVER_USER="eva"
SERVER_DIR="/home/$SERVER_USER/button_panel$SUFFIX"

dir=debug
if [ "$1" = "--release" ]; then
	dir=release
fi

cross build --target=armv7-unknown-linux-gnueabihf $1
rsync button_panel$SUFFIX.service $SERVER_ADDR:/tmp/
rsync -vh --progress \
  target/armv7-unknown-linux-gnueabihf/$dir/control \
  $SERVER_ADDR:/tmp/

# sets up/updates the systemd service and places the binary
cmds="
sed -i \"s/<USER>/$SERVER_USER/g\" /tmp/button_panel$SUFFIX.service
sed -i \"s+<DIR>+$SERVER_DIR+g\" /tmp/button_panel$SUFFIX.service
sudo mv /tmp/button_panel$SUFFIX.service /etc/systemd/system/

mkdir -p $SERVER_DIR
mv /tmp/control $SERVER_DIR/button_panel$SUFFIX
chown $SERVER_USER:$SERVER_USER $SERVER_DIR/button_panel$SUFFIX

sudo systemctl enable button_panel$SUFFIX.service$STOP_MAIN
sudo systemctl restart button_panel$SUFFIX.service
"

ssh -t $SERVER_ADDR "$cmds"
