#!/usr/bin/env python3
"""ps2referee's parts that need no referee: PINE's framing, the scan of a process's RAM, the BIOS folder of links, the
settings file, the probe's command line and the tty mode's cut of a log - see Sapphire_Referees.md §2 and Sapphire_Native.md §14."""
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
    """A process faked as a maps file and a memory file. A recompiler-like cache holds the done word first; the RAM,
    split in two mappings by a read-only page as a referee's write-protected code page splits it, holds the program at
    00100000h and the done word at 001FFFF0h, so RAM's address 0 is at host 00100000h."""
    CODE = bytes(range(16))

    def setUp(self):
        self.dir = tempfile.TemporaryDirectory()
        d = Path(self.dir.name)
        self.mem = d / "mem"
        memory = bytearray(0x400000)
        struct.pack_into("<I", memory, 0x00010FF0, 0x5A99D00E)   # in a read-only mapping: ignored
        struct.pack_into("<I", memory, 0x00020100, 0x5A99D00E)   # a cache's constant, with no program beside it
        memory[0x00200000:0x00200010] = self.CODE                # the program, in the read-only page
        struct.pack_into("<I", memory, 0x002FFFF0, 0x5A99D00E)   # the done word at 001FFFF0h
        struct.pack_into("<3I", memory, 0x00300000, 0x80000000, 0xF8000000, 0x08000000)
        self.mem.write_bytes(bytes(memory))
        self.maps = d / "maps"
        self.maps.write_text("00010000-00011000 r--p 00000000 00:00 0\n"
                             "00020000-00030000 rw-p 00000000 00:00 0\n"
                             "00100000-00200000 rw-p 00000000 00:00 0\n"
                             "00200000-00201000 r--p 00000000 00:00 0\n"
                             "00201000-00400000 rw-p 00000000 00:00 0 [heap]\n")
        self.scan = ps2referee.Scan(0, (0x001FFFF0, 0x5A99D00E), (0x00100000, self.CODE), maps=self.maps, mem=self.mem)

    def tearDown(self):
        self.dir.cleanup()

    def test_ram_is_where_the_program_sits_beside_the_done_word(self):
        self.assertTrue(self.scan.find())
        self.assertEqual((self.scan.at, self.scan.hits), (0x002FFFF0, 2))

    def test_the_record_is_read_relative_to_the_done_word(self):
        self.scan.find()
        self.assertEqual(self.scan.words(0x00200000, 3), [0x80000000, 0xF8000000, 0x08000000])
        self.assertEqual(self.scan.words(0x001FFFF0, 1), [0x5A99D00E])

    def test_a_read_past_what_is_mapped_is_refused(self):
        self.scan.find()
        with self.assertRaises(OSError):
            self.scan.words(0x00300000, 4)

    def test_no_place_with_the_program_finds_nothing(self):
        scan = ps2referee.Scan(0, (0x001FFFF0, 0x5A99D00E), (0x00100000, bytes(16)), maps=self.maps, mem=self.mem)
        self.assertFalse(scan.find())
        self.assertEqual(scan.hits, 2)

    def test_the_anchor_is_the_elfs_loaded_segment(self):
        elf = Path(self.dir.name) / "p.elf"
        header = bytearray(84)
        header[:4] = b"\x7fELF"
        struct.pack_into("<I", header, 28, 52)
        struct.pack_into("<H", header, 44, 1)
        struct.pack_into("<5I", header, 52, 1, 84, 0x00100000, 0x00100000, 16)
        elf.write_bytes(bytes(header) + self.CODE)
        self.assertEqual(ps2referee.elf_segment(elf), (0x00100000, self.CODE))


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

    def test_any_other_image_copied_in_is_refused(self):
        folder = ps2referee.bios_folder(self.cache, self.home)
        (folder / "ps2-0100j-20000117.bin").write_bytes(b"\0")
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


class TtyTests(unittest.TestCase):
    EXPECTED = "-- TEST BEGIN\r\naddu:\r\n  addu 0, 0: 00000000\r\n\r\n-- TEST END\r\n"

    def test_only_the_programs_lines_are_returned_and_the_log_around_them_never(self):
        log = "boot line one\nkernel says hello\n-- TEST BEGIN\naddu:\n  addu 0, 0: 00000000\n\n-- TEST END\nafter the test\n"
        region, found, total = ps2referee.program_text(log, self.EXPECTED)
        self.assertEqual(region, ["-- TEST BEGIN", "addu:", "  addu 0, 0: 00000000", "", "-- TEST END"])
        self.assertEqual((found, total), (4, 4))
        self.assertFalse(any("boot" in line or "kernel" in line or "after" in line for line in region))

    def test_a_fixed_prefix_on_every_line_is_cut_by_the_first_lines(self):
        log = "[ 1.0] -- TEST BEGIN\n[ 1.1] addu:\n[ 1.2]   addu 0, 0: 00000000\n[ 1.3] \n[ 1.4] -- TEST END\n"
        region, found, total = ps2referee.program_text(log, self.EXPECTED)
        self.assertEqual((region[1], found, total), ("addu:", 4, 4))

    def test_a_log_without_the_program_returns_nothing(self):
        self.assertEqual(ps2referee.program_text("only the bios\n", self.EXPECTED), ([], 0, 4))
        self.assertEqual(ps2referee.program_text("-- TEST END\n-- TEST BEGIN\n", self.EXPECTED), ([], 0, 4))

    def test_a_wrong_line_is_kept_in_the_region_and_not_counted(self):
        log = "-- TEST BEGIN\naddu:\n  addu 0, 0: 00000001\n\n-- TEST END\n"
        region, found, total = ps2referee.program_text(log, self.EXPECTED)
        self.assertEqual((len(region), found, total), (5, 3, 4))

    def test_each_channel_switches_one_console_on(self):
        self.assertEqual({c: (v["EnableEEConsole"], v["EnableIOPConsole"]) for c, v in ps2referee.TTY_CHANNELS.items()},
                         {"ee": ("true", "false"), "iop": ("false", "true"), "none": ("false", "false")})

    def test_the_ee_recompiler_is_written_every_run_so_an_interpreter_run_does_not_persist(self):
        self.assertEqual(ps2referee.PCSX2_SETTINGS["EmuCore/CPU/Recompiler"], {"EnableEE": "true"})
        kept = ps2referee.ini_merge("[EmuCore/CPU/Recompiler]\nEnableEE = false\nEnableIOP = true\n", {"EmuCore/CPU/Recompiler": {"EnableEE": "true"}})
        self.assertEqual(kept, "[EmuCore/CPU/Recompiler]\nEnableEE = true\nEnableIOP = true\n\n")


if __name__ == "__main__":
    unittest.main()
