#!/bin/sh
# Copied to /bin/<name> at publish time; the real app dir is /lib/EmuSen - see `man hier`.
# Deliberately name-agnostic: it dispatches on its own filename, so one file
# serves every frontend and nothing has to be templated at publish time.
here=`dirname "$0"`
# Mesa's GLX keeps the window's drawable across Avalonia's pbuffer binds, unless the player set it - see EmuSen_DevicePicture.md §18.
: "${keep_native_window_glx_drawable:=true}"
export keep_native_window_glx_drawable
exec "$here/../lib/EmuSen/`basename "$0"`" "$@"
