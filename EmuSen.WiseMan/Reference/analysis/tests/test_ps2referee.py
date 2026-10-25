#!/usr/bin/env python3
"""ps2referee's parts that need no referee: PINE's framing, the scan of a process's RAM, the BIOS folder of links, the
settings file and the probe's command line - see Sapphire_Referees.md §2."""
import os
import socket
import struct
import tempfile
import threading
import unittest
from pathlib import Path

import ps2referee


class FakePine:
    """A PINE server on a Unix socket that answers Read32 from a dictionary, FAIL for any address it lacks, and a
    string for the version; it keeps every request it was sent."""

    def __init__(self, path, memory):
        self.memory, self.requests = memory, []
        self.server = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
        self.server.bind(path)
        self.server.listen(1)
        threading.Thread(target=self.serve, daemon=True).start()

    def serve(self):
        conn, _ = self.server.accept()
        with conn:
            while True:
                head = conn.recv(4)
                if len(head) < 4:
                    return
                size = struct.unpack("<I", head)[0]
                body = conn.recv(size - 4)
                self.requests.append(head + body)
                if body[0] == 2:
                    address = struct.unpack("<I", body[1:5])[0]
                    answer = b"\0" + struct.pack("<I", self.memory[address]) if address in self.memory else b"\xff"
                elif body[0] == 8:
                    text = b"PCSX2 test\0"
                    answer = b"\0" + struct.pack("<I", len(text)) + text
                else:
                    answer = b"\xff"
                conn.sendall(struct.pack("<I", 4 + len(answer)) + answer)


class PineTests(unittest.TestCase):
    def setUp(self):
        self.dir = tempfile.TemporaryDirectory()
        self.path = os.path.join(self.dir.name, "pcsx2.sock.28123")
        self.server = FakePine(self.path, {0x001FFFF0: 0x5A99D00E, 0x00200000: 0x80000000, 0x00200004: 0xF8000000})
        self.pine = ps2referee.Pine(self.path)

    def tearDown(self):
        self.pine.close()
        self.server.server.close()
        self.dir.cleanup()

    def test_a_read_is_nine_bytes_its_size_counting_itself(self):
        self.assertEqual(self.pine.read32(0x001FFFF0), 0x5A99D00E)
        self.assertEqual(self.server.requests[0], bytes.fromhex("09000000" "02" "f0ff1f00"))

    def test_words_are_read_one_after_another(self):
        self.assertEqual(self.pine.words(0x00200000, 2), [0x80000000, 0xF8000000])

    def test_a_fail_answer_is_its_own_exception(self):
        with self.assertRaises(ps2referee.PineFail):
            self.pine.read32(0x01000000)
        self.assertNotIsInstance(ps2referee.PineFail("x"), OSError)

    def test_a_string_answer_is_its_length_then_its_bytes(self):
        self.assertEqual(self.pine.version(), "PCSX2 test")


class ScanTests(unittest.TestCase):
    """A process faked as a maps file and a memory file: the done word in the second writable mapping."""

    def setUp(self):
        self.dir = tempfile.TemporaryDirectory()
        d = Path(self.dir.name)
        self.mem = d / "mem"
        memory = bytearray(0x30000)
        struct.pack_into("<I", memory, 0x0FF0, 0x5A99D00E)      # in the read-only mapping: ignored
        struct.pack_into("<I", memory, 0x21000, 0x5A99D00E)     # the done word at 001FFFF0h
        struct.pack_into("<3I", memory, 0x21010, 0x80000000, 0xF8000000, 0x08000000)
        self.mem.write_bytes(bytes(memory))
        self.maps = d / "maps"
        self.maps.write_text("00000000-00010000 r--p 00000000 00:00 0\n"
                             "00010000-00020000 rw-p 00000000 00:00 0\n"
                             "00020000-00030000 rw-p 00000000 00:00 0 [heap]\n")
        self.scan = ps2referee.Scan(0, (0x001FFFF0, 0x5A99D00E), maps=self.maps, mem=self.mem)

    def tearDown(self):
        self.dir.cleanup()

    def test_the_done_word_is_found_in_a_writable_mapping_only(self):
        self.assertTrue(self.scan.find())
        self.assertEqual((self.scan.at, self.scan.span, self.scan.hits), (0x21000, (0x20000, 0x30000), 1))

    def test_the_record_is_read_relative_to_the_done_word(self):
        self.scan.find()
        self.assertEqual(self.scan.words(0x00200000, 3), [0x80000000, 0xF8000000, 0x08000000])
        self.assertEqual(self.scan.words(0x001FFFF0, 1), [0x5A99D00E])

    def test_a_read_past_the_mapping_is_refused(self):
        self.scan.find()
        with self.assertRaises(ValueError):
            self.scan.words(0x00200000, 0x4000)
        with self.assertRaises(ValueError):
            self.scan.words(0x001E0000, 1)

    def test_no_done_word_finds_nothing(self):
        scan = ps2referee.Scan(0, (0x001FFFF0, 0x12345678), maps=self.maps, mem=self.mem)
        self.assertFalse(scan.find())


class BiosFolderTests(unittest.TestCase):
    def setUp(self):
        self.dir = tempfile.TemporaryDirectory()
        self.cache, self.home = Path(self.dir.name) / "cache", Path(self.dir.name) / "home"

    def tearDown(self):
        self.dir.cleanup()

    def test_the_folder_holds_links_to_the_gates_images_and_nothing_is_opened(self):
        folder = ps2referee.bios_folder(self.cache, self.home)
        self.assertFalse(self.home.exists(), "the images' folder is never created or written")
        for name in ps2referee.MARKETS.values():
            self.assertTrue((folder / name).is_symlink())
            self.assertEqual(os.readlink(folder / name), str(self.home / name))
        self.assertEqual(sorted(p.name for p in folder.iterdir()), sorted(ps2referee.MARKETS.values()))

    def test_a_referees_own_files_beside_the_links_are_kept(self):
        folder = ps2referee.bios_folder(self.cache, self.home)
        (folder / "ps2-0230a-20080220.nvm").write_bytes(b"\0" * 1024)
        ps2referee.bios_folder(self.cache, self.home)
        self.assertTrue((folder / "ps2-0230a-20080220.nvm").exists())

    def test_an_image_copied_in_is_refused(self):
        folder = ps2referee.bios_folder(self.cache, self.home)
        (folder / "ps2-0230a-20080220.bin").unlink()
        (folder / "ps2-0230a-20080220.bin").write_bytes(b"\0")
        with self.assertRaises(SystemExit):
            ps2referee.bios_folder(self.cache, self.home)


class SettingsTests(unittest.TestCase):
    def test_wanted_keys_are_set_and_the_rest_kept_in_order(self):
        text = "[UI]\nTheme = dark\nSetupWizardIncomplete = true\n\n[EmuCore/GS]\nRenderer = -1\n"
        merged = ps2referee.ini_merge(text, {"EmuCore/GS": {"Renderer": "13"}, "UI": {"SetupWizardIncomplete": "false"}, "Filenames": {"BIOS": "x.bin"}})
        self.assertEqual(merged, "[UI]\nTheme = dark\nSetupWizardIncomplete = false\n\n[EmuCore/GS]\nRenderer = 13\n\n[Filenames]\nBIOS = x.bin\n\n")

    def test_a_run_is_refused_unless_the_log_names_the_markets_image(self):
        log = "[    0.0673] BIOS Found: USA     v02.30(20/02/2008)  Console 20080220-175343\n"
        self.assertTrue(ps2referee.bios_loaded(log, "americas"))
        self.assertFalse(ps2referee.bios_loaded(log, "europe"))
        self.assertFalse(ps2referee.bios_loaded("", "americas"))


class ProbeTests(unittest.TestCase):
    def test_the_bios_cores_get_the_folder_the_image_and_the_software_renderer(self):
        cmd = ps2referee.libretro_command("pcee2", Path("/p/record.elf"), "europe", 60, "/out")
        self.assertEqual(cmd[1:5], ["/p/record.elf", "/out", "60", "60"])
        self.assertIn("pcsx2_bios=ps2-0230e-20080220.bin", cmd)
        self.assertIn("pcsx2_renderer=software", cmd)
        self.assertEqual(Path(cmd[cmd.index("--sysdir") + 1]) / "pcsx2/bios", ps2referee.CACHE / "system/pcsx2/bios")

    def test_play_gets_no_firmware(self):
        cmd = ps2referee.libretro_command("play_libretro", Path("/p/record.elf"), "americas", 60, "/out")
        self.assertNotIn("--sysdir", cmd)
        self.assertTrue(cmd[cmd.index("--core") + 1].endswith("/play_libretro.so"))

    def test_record_words_read_ram_from_address_zero_whatever_the_segment(self):
        dump = bytearray(ps2referee.EE_RAM)
        struct.pack_into("<2I", dump, 0x200000, 0x80000000, 0xF8000000)
        self.assertEqual(ps2referee.record_words(bytes(dump), [(0x00200000, 2), (0x20200004, 1)]), {str(0x00200000): [0x80000000, 0xF8000000], str(0x20200004): [0xF8000000]})


if __name__ == "__main__":
    unittest.main()
