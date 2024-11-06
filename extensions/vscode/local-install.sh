#!/bin/sh
EXTENSIONS_PATH=$HOME/.vscode-oss/extensions
SRC=$(dirname $0)
DST=$EXTENSIONS_PATH/spruceid-treeldr-0.1.0
echo "Installing \`$SRC\` to \`$DST\`"
ln -s $SRC $DST