#! /bin/env bash

# To find the correct one (if you have set the binary name == folder name)
# CMTDT_="$(cargo metadata --format-version 1)"; ETR_=$(basename $PWD); for ((i=0;; i++)); do if ! JQP_=$(jq -r ".packages[${i}].targets[0].name" <<<"${CMTDT_}"); then echo "JQ error"; break; fi; if [ "${JQP_}" == "${ETR_}" ]; then printf '\x1b[38;5;99mFound project "%s" in target %3d\n' "${ETR_}" "${i}"; break; else printf 'Item %3d %24s\n' "${i}" "${JQP_}" ; fi; [ "${JQP_}" == "null" ] && break; done; unset CMTDT_ ETR_ JQP_

BIN_NAME=$(cargo metadata --format-version 1 | jq -r '.packages[63].targets[0].name')
# cargo run --bin ${BIN_NAME} -- "${@}"

# Get this script location
MELOC=$(
  dirname "$(readlink -f "${BASH_SOURCE[0]}")"
)

# Check for both (debug or release) and select the most recent one
if [ -f "${MELOC}/target/debug/${BIN_NAME}" ]; then
  LOC=debug
elif [ -f "${MELOC}/target/release/${BIN_NAME}" ]; then
  LOC=release
else
  cargo build --release
  LOC=release
fi

if [ "${1}" == "--rebuild" ]; then
  shift 1
  cargo build --release
  LOC=release
fi

# Get the correct path to binary file
BIN="${MELOC}/target/${LOC}/${BIN_NAME}"

# Start bot or gui mode based on arguments (needed coreutils >= 9.5)
if [[ "${1}" =~ ^--(bot|gui)$ ]]; then
  env --argv0="lyr${1#--}" ${BIN} "${@:2}"
  exit
fi

# Run the binary normally
${BIN} "${@}"
