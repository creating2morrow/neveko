#!/bin/bash
# Build neveko debug
# Run from the neveko root
# usage: ./scripts/build_debug_release vX.X.X-ver

# Linux x86_64 output directory

LINUX_X86_64="x86_64-linux-gnu"
RELEASE_NAME="neveko-$LINUX_X86_64-$1"
LINUX_OUTPUT_DIR=".build/debug/$RELEASE_NAME"
DEBUG_DIR="target/debug"
AUTH_PATH="neveko-auth"
AUTH_BINARY="neveko_auth"
AUTH_DIR=$LINUX_OUTPUT_DIR/$AUTH_PATH/$DEBUG_DIR/
CONTACT_PATH="neveko-contact"
CONTACT_BINARY="neveko_contact"
CONTACT_DIR=$LINUX_OUTPUT_DIR/$CONTACT_PATH/$DEBUG_DIR/
GUI_PATH="neveko-gui"
GUI_BINARY="neveko_gui"
GUI_DIR=$LINUX_OUTPUT_DIR/$GUI_PATH/$DEBUG_DIR/
MARKET_PATH="neveko-market"
MARKET_BINARY="neveko_market"
MARKET_DIR=$LINUX_OUTPUT_DIR/$MARKET_PATH/$DEBUG_DIR/
MESSAGE_PATH="neveko-message"
MESSAGE_BINARY="neveko_message"
MESSAGE_DIR=$LINUX_OUTPUT_DIR/$MESSAGE_PATH/$DEBUG_DIR/

mkdir -p $LINUX_OUTPUT_DIR
mkdir -p $AUTH_DIR
mkdir -p $CONTACT_DIR
mkdir -p $GUI_DIR
mkdir -p $MARKET_DIR
mkdir -p $MESSAGE_DIR
cargo build 
cp $DEBUG_DIR/neveko $LINUX_OUTPUT_DIR
cd $GUI_PATH && cargo build  && cp $DEBUG_DIR/$GUI_BINARY ../$GUI_DIR
cp -r assets/ ../$LINUX_OUTPUT_DIR
cd ../
cd $AUTH_PATH && cargo build  && cp $DEBUG_DIR/$AUTH_BINARY ../$AUTH_DIR
cd ../
cd $CONTACT_PATH && cargo build  && cp $DEBUG_DIR/$CONTACT_BINARY ../$CONTACT_DIR
cd ../
cd $MARKET_PATH && cargo build  && cp $DEBUG_DIR/$MARKET_BINARY ../$MARKET_DIR
cd ../
cd $MESSAGE_PATH && cargo build  && cp $DEBUG_DIR/$MESSAGE_BINARY ../$MESSAGE_DIR
cd ../
# make the bzip for linux
cd .build/debug/ && tar -cjf $RELEASE_NAME.tar.bz2 $RELEASE_NAME/ && mv $RELEASE_NAME.tar.bz2 ../../
