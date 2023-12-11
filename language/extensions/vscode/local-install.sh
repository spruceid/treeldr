#!/bin/sh
EXTENSIONS_PATH=`codium --verbose --list-extensions | grep -Po "(?<='extensions-dir': ')([a-zA-Z0-9\\-:./]+)(?=')"`
HERE=`pwd`
sudo ln -s $HERE $EXTENSIONS_PATH/spruceid-treeldr