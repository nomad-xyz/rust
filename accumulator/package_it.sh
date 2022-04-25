#!/usr/bin/env bash
set -e

# change these :)
SCOPE="nomad-xyz"
PKG_NAME="accumulator"

# Check if jq is installed
if ! [ -x "$(command -v jq)" ]; then
    echo "jq is not installed" >& 2
    echo "how do you live with yourself"
    exit 1
fi


# Check if jq is installed
if ! [ -x "$(command -v wasm-pack)" ]; then
    echo "wasm-pack is not installed" >& 2
    exit 1
fi
# Cleanup any old generated code
rm -rf ./ts
mkdir -p ./ts

# build
wasm-pack build  --target browser --scope $SCOPE --out-dir ts/web --out-name $PKG_NAME
wasm-pack build  --target nodejs --scope $SCOPE --out-dir ts/node --out-name $PKG_NAME

# use the browser package.json as a base
mv ts/web/package.json ts/
# copy in licenses from repo root
cp ../LICENSE* ts/
# Get the README in the root, delete the spare
mv ts/web/README* ts/

# clean redundant files
rm ts/node/README*
rm ts/node/package.json

# set the package.json main key (affects how nodejs loads this)
cat ts/package.json | jq --arg main "node/$PKG_NAME.js" '.main = $main'> TMP_FILE && mv TMP_FILE ts/package.json

# set the package.json browser key (affects how bundlers load this)
cat ts/package.json | jq --arg browser "web/$PKG_NAME.js" '.browser = $browser'> TMP_FILE && mv TMP_FILE ts/package.json

# set the package.json module key (affects how bundlers load this)
cat ts/package.json | jq --arg m "web/$PKG_NAME.js" '.module = $m' > TMP_FILE && mv TMP_FILE ts/package.json

# set the package.json name key (correct module name)
cat ts/package.json | jq --arg n "@$SCOPE/$PKG_NAME" '.name = $n' > TMP_FILE && mv TMP_FILE ts/package.json

# set the package.json types key
cat ts/package.json | jq --arg types "web/$PKG_NAME.d.ts" '.types = $types' > TMP_FILE && mv TMP_FILE ts/package.json

# empty the package.json files list
cat ts/package.json | jq '.files = []' > TMP_FILE && mv TMP_FILE ts/package.json

# add each web file to the package.json files list
for F in "web/$PKG_NAME""_bg.wasm" "web/$PKG_NAME""_bg.d.ts" "web/$PKG_NAME.js" "web/$PKG_NAME.d.ts" "web/$PKG_NAME""_bg.js"
do
    cat ts/package.json | jq --arg f "$F" '.files += [$f]' > TMP_FILE && mv TMP_FILE ts/package.json
done

# add each node file to the package.json files list
for F in "node/$PKG_NAME""_bg.wasm" "node/$PKG_NAME""_bg.d.ts" "node/$PKG_NAME.js" "node/$PKG_NAME.d.ts"
do
    cat ts/package.json | jq --arg f "$F" '.files += [$f]' > TMP_FILE && mv TMP_FILE ts/package.json
done