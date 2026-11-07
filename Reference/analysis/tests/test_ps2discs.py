#!/usr/bin/env python3
"""ps2discs' volumes of files: a DVD's path tables at 257 and 259 and its root at 261, where the player's BIOS reads a
DVD's, and a CD's at 18, 19 and 20, each table naming the root and the root naming every file at an extent holding its
bytes - see Sapphire_Native.md §40.14 and Sapphire_Disputes.md D-483."""
import struct
import unittest

import ps2discs

U = 2048


def sector(image, n):
    return image[n * U:(n + 1) * U]


def root_entries(image, root):
    d, at, out = sector(image, root), 0, {}
    while at < U and d[at]:
        r = d[at:at + d[at]]
        out[r[33:33 + r[32]]] = (struct.unpack("<I", r[2:6])[0], struct.unpack("<I", r[10:14])[0])
        at += d[at]
    return out


class FilesVolume(unittest.TestCase):
    FILES = [("LOADER.ELF", b"\x7fELF" + bytes(3000)), ("TEST.IRX", bytes(range(256)) * 9)]

    def check(self, layout):
        lpt, mpt, root = layout
        image = ps2discs.files_volume(self.FILES, layout)
        pvd = sector(image, 16)
        self.assertEqual(pvd[:6], b"\x01CD001")
        self.assertEqual(struct.unpack("<I", pvd[80:84])[0], len(image) // U)
        self.assertEqual((struct.unpack("<I", pvd[140:144])[0], struct.unpack(">I", pvd[148:152])[0]), (lpt, mpt))
        self.assertEqual(struct.unpack("<I", pvd[158:162])[0], root)
        self.assertEqual(sector(image, lpt)[:8], bytes([1, 0]) + struct.pack("<I", root) + struct.pack("<H", 1))
        self.assertEqual(sector(image, mpt)[:8], bytes([1, 0]) + struct.pack(">I", root) + struct.pack(">H", 1))
        entries = root_entries(image, root)
        self.assertEqual(entries[b"\x00"], (root, U))
        cnf_at, cnf_size = entries[b"SYSTEM.CNF;1"]
        self.assertTrue(image[cnf_at * U:cnf_at * U + cnf_size].startswith(b"BOOT2 = cdrom0:\\LOADER.ELF;1\r\n"))
        for name, data in self.FILES:
            at, size = entries[name.encode() + b";1"]
            self.assertGreater(at, root)
            self.assertEqual((image[at * U:at * U + size], size), (data, len(data)))
        return image

    def test_a_dvds_path_tables_and_root_lie_where_the_bios_reads_a_dvds(self):
        self.assertEqual(ps2discs.DVD_LAYOUT, (257, 259, 261))
        self.check(ps2discs.DVD_LAYOUT)

    def test_a_cds_follow_its_descriptors(self):
        self.assertEqual(ps2discs.CD_LAYOUT, (18, 19, 20))
        image = self.check(ps2discs.CD_LAYOUT)
        raw = ps2discs.raw_track(image)
        self.assertEqual(len(raw), len(image) // U * 2352)
        self.assertEqual(raw[16 * 2352 + 24:16 * 2352 + 24 + U], sector(image, 16))


if __name__ == "__main__":
    unittest.main()
