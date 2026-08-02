#!/bin/sh
# Copied to /bin/<name> at publish time; the real app dir is /lib/EmuSen - see `man hier`.
# Deliberately name-agnostic: it dispatches on its own filename, so one file
# serves every frontend and nothing has to be templated at publish time.
here=`dirname "$0"`
exec "$here/../lib/EmuSen/`basename "$0"`" "$@"
