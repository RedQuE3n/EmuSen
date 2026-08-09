#!/usr/bin/env python3
"""One emulator's output, read without knowing which emulator made it.

The identity and the CRC32 stream come from the database (dumpdb.py ingests the
directory on first use); the screens are read from the raw `.bin` blobs on
demand, because a dump set is tens of megabytes of framebuffer and a comparison
touches a couple of dozen frames of it.

Screen filenames are reconstructed by convention - `<backend>_screen_fNNNNN.bin`
- rather than read from the manifest's `file` field. That is what the C# did, and
the manifest agrees with the convention in every set on disk, so following the
manifest instead would be a behaviour change rather than a fix. The `screen` table
records what the manifest claimed, so the two can be checked against each other
when that becomes worth doing.

Screens are cached in memory here, unlike the C#, which re-read and re-decoded a
blob every time it was asked for. The phase search asks for the same window twice
- once to find the best match, once to decide whether the reference moves at all -
so the C# decoded every frame in the window twice per comparison. Caching changes
no result, only the time.

See EmuSen_Debugging_Tools_Reference_v5.md §3.48.
"""
import os

import dumpdb
import screen as screen_module


class DumpSet:

    def __init__(self, db, row, columns, signature, directory):
        self.db = db
        self.set_id = row["id"]
        self.backend = row["backend"]
        self.system = row["system"]
        self.rom = row["rom"]
        self.board = row["board"]
        self.region = row["region"]
        self.header_trust = row["header_trust"]
        self.prg_bytes = row["prg_bytes"]
        self.chr_bytes = row["chr_bytes"]
        self.save_loaded = bool(row["save_loaded"])
        self.screen_format = row["screen_format"]
        self.directory = directory
        self.columns = columns
        self.signature = signature
        self._screens = {}

    @staticmethod
    def load(db, directory):
        """The set for a directory, or None when there is no dump there to read."""
        row = dumpdb.set_for(db, directory)
        if row is None or not row["backend"]:
            return None
        return DumpSet(db, row, dumpdb.columns(db, row["id"]),
                       dumpdb.signature(db, row["id"]), os.path.abspath(directory))

    def screen(self, frame):
        """The decoded framebuffer for a frame, or None when it was not dumped."""
        if frame in self._screens:
            return self._screens[frame]

        path = os.path.join(self.directory, f"{self.backend}_screen_f{frame:05d}.bin")
        image = None
        if os.path.exists(path):
            with open(path, "rb") as blob:
                image = screen_module.read(blob.read(), self.screen_format)

        self._screens[frame] = image
        return image

    def varies(self, column):
        """Whether a column ever changes; one that never does cannot locate anything."""
        return dumpdb.varies(self.db, self.set_id, column)
