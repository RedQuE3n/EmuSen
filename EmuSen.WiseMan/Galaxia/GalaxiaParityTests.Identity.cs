using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Security.Cryptography;
using System.Threading;
using EmuSen.Galaxia.Library;
using EmuSen.Galaxia.Native;

namespace EmuSen.WiseMan.Galaxia
{
    // A dump's identity asked of both halves: whole, ranged, held in memory and per track - see EmuSen_GamePackages.md §3.3 and EmuSen_Galaxia.md §5.4.
    public partial class GalaxiaParityTests
    {
        private static RomHashes Native(string path, long offset = 0, long length = -1, CancellationToken stop = default) =>
            RomHash.FromJson(GalaxiaNative.RomHashes(path, offset, length, stop));

        // The three hashes as .NET's own algorithms give them, the independent answer both halves are held to.
        private static RomHashes Expected(byte[] bytes)
        {
            uint crc = 0xFFFFFFFF;
            foreach (byte b in bytes)
            {
                crc ^= b;
                for (int k = 0; k < 8; k++) crc = (crc & 1) != 0 ? 0xEDB88320 ^ (crc >> 1) : crc >> 1;
            }
            return new RomHashes(Convert.ToHexStringLower(MD5.HashData(bytes)), (~crc).ToString("x8"), Convert.ToHexStringLower(SHA1.HashData(bytes)), bytes.Length);
        }

        [Theory]
        [InlineData(0)]
        [InlineData(1)]
        [InlineData(9)]
        [InlineData((1 << 16) - 1)]
        [InlineData(1 << 16)]
        [InlineData((1 << 16) + 1)]
        [InlineData(3_000_000)]
        public void A_dumps_three_hashes_are_the_same_whole_ranged_and_held(int length)
        {
            byte[] contents = new byte[length];
            new Random(length + 7).NextBytes(contents);
            string rom = At("identity", $"rom {length} é.bin");
            Directory.CreateDirectory(Path.GetDirectoryName(rom)!);
            File.WriteAllBytes(rom, contents);

            Assert.Equal(Expected(contents), RomHash.Managed.Of(rom, 0, -1, default));
            Assert.Equal(Expected(contents), Native(rom));
            Assert.Equal(Expected(contents), RomHash.Managed.Of(contents));
            Assert.Equal(Expected(contents), RomHash.FromJson(GalaxiaNative.RomHashesOf(contents)));
            Assert.Equal(RomHash.Md5(rom), Native(rom).Md5);
            foreach ((long offset, long take) in new (long, long)[] { (0, 0), (0, 1), (1, 3), (length / 2, -1), (length / 3, length / 3), (length, 5), (length + 10, -1), (0, length + 100), ((1 << 16) - 1, 2) })
            {
                byte[] part = contents.Skip((int)Math.Min(offset, length)).Take(take < 0 ? int.MaxValue : (int)Math.Min(take, int.MaxValue)).ToArray();
                Assert.Equal(Expected(part), RomHash.Managed.Of(rom, offset, take, default));
                Assert.Equal(Expected(part), Native(rom, offset, take));
            }
        }

        [Fact]
        public void A_dump_that_cannot_be_hashed_throws_alike_and_a_stop_is_a_cancellation()
        {
            Directory.CreateDirectory(At("identity"));
            foreach (string path in new[] { At("identity", "absent.bin"), At("no", "such", "folder", "rom.bin"), At("identity") })
            {
                Exception managed = Assert.ThrowsAny<Exception>(() => RomHash.Managed.Of(path, 0, -1, default));
                Exception native = Assert.ThrowsAny<Exception>(() => Native(path));
                Assert.True(managed.GetType() == native.GetType(), $"{path}: {managed.GetType().Name} and {native.GetType().Name}");
            }
            string rom = At("identity", "rom.bin");
            File.WriteAllBytes(rom, new byte[1 << 20]);
            using var stopped = new CancellationTokenSource();
            stopped.Cancel();
            Assert.ThrowsAny<OperationCanceledException>(() => RomHash.Managed.Of(rom, 0, -1, stopped.Token));
            Assert.ThrowsAny<OperationCanceledException>(() => Native(rom, stop: stopped.Token));
            Assert.ThrowsAny<OperationCanceledException>(() => GalaxiaNative.RomTracks(rom, stopped.Token));
        }

        [Fact]
        public void A_transform_is_hashed_only_where_it_changes_the_bytes()
        {
            byte[] file = "NES\u001aheader..body"u8.ToArray();
            Assert.Equal(Expected(file[16..]), RomHash.Transformed(file, f => f[16..]));
            Assert.Null(RomHash.Transformed(file, f => f.ToArray()));
            Assert.Null(RomHash.Transformed(file, _ => null));
        }

        // A sheet's track files, each of sectors whose bytes are their own, as a generated dump is.
        private static byte[] Sectors(int count, int size, int seed)
        {
            byte[] bytes = new byte[count * size];
            new Random(seed).NextBytes(bytes);
            return bytes;
        }

        private static byte[] Wave(byte[] samples)
        {
            using var w = new MemoryStream();
            using var b = new BinaryWriter(w);
            b.Write("RIFF"u8); b.Write(36 + samples.Length); b.Write("WAVE"u8);
            b.Write("fmt "u8); b.Write(16); b.Write((short)1); b.Write((short)2); b.Write(44100); b.Write(44100 * 4); b.Write((short)4); b.Write((short)16);
            b.Write("data"u8); b.Write(samples.Length); b.Write(samples);
            b.Flush();
            return w.ToArray();
        }

        [Fact]
        public void Each_tracks_hashes_are_its_generated_files_or_its_range_of_one_both_ways()
        {
            byte[] data = Sectors(40, 2352, 1), audio = Sectors(30, 2352, 2), cooked = Sectors(12, 2048, 3), last = Sectors(7, 2352, 4);
            byte[] wave = Wave(Sectors(5, 2352, 5));
            var cases = new List<(string Sheet, Dictionary<string, byte[]> Files, (int Track, string File, long Offset, byte[] Bytes)[] Expected)>
            {
                ("FILE \"G (Track 1).bin\" BINARY\n  TRACK 01 MODE2/2352\n    INDEX 01 00:00:00\nFILE \"G (Track 2).bin\" BINARY\n  TRACK 02 AUDIO\n    INDEX 00 00:00:00\n    INDEX 01 00:00:20\n",
                    new() { ["G (Track 1).bin"] = data, ["g (track 2).BIN"] = audio },
                    new[] { (1, "G (Track 1).bin", 0L, data), (2, "G (Track 2).bin", 0L, audio) }),
                ("FILE \"one.bin\" BINARY\nTRACK 01 MODE1/2048\nINDEX 01 00:00:00\nTRACK 02 AUDIO\nINDEX 00 00:00:12\nINDEX 01 00:00:14\nTRACK 03 MODE2/2352\nINDEX 01 00:00:42\n",
                    new() { ["one.bin"] = cooked.Concat(audio).Concat(last).ToArray() },
                    new[] { (1, "one.bin", 0L, cooked), (2, "one.bin", (long)cooked.Length, audio), (3, "one.bin", (long)(cooked.Length + audio.Length), last) }),
                ("FILE \"d.bin\" BINARY\nTRACK 01 MODE1/2352\nINDEX 01 00:00:00\nFILE \"m.wav\" WAVE\nTRACK 02 AUDIO\nPREGAP 00:02:00\nINDEX 01 00:00:00\n",
                    new() { ["d.bin"] = data, ["m.wav"] = wave },
                    new[] { (1, "d.bin", 0L, data), (2, "m.wav", 0L, wave) }),
            };
            int n = 0;
            foreach (var (sheet, files, expected) in cases)
            {
                string folder = At("tracks", $"case {n++}");
                Directory.CreateDirectory(folder);
                foreach (var (name, bytes) in files) File.WriteAllBytes(Path.Combine(folder, name), bytes);
                string cue = Path.Combine(folder, "game.cue");
                File.WriteAllText(cue, sheet);
                var want = expected.Select(e => new TrackHashes(e.Track, e.File, e.Offset, Expected(e.Bytes))).ToArray();
                Assert.Equal(want, RomHash.Managed.Tracks(cue, default));
                Assert.Equal(want, RomHash.TracksFromJson(GalaxiaNative.RomTracks(cue, default)));
            }
        }

        [Fact]
        public void A_sheet_that_cannot_be_hashed_by_track_throws_alike()
        {
            string folder = At("tracks", "bad");
            Directory.CreateDirectory(folder);
            var sheets = new Dictionary<string, string>
            {
                ["not a sheet.cue"] = "TRACK 01 AUDIO\n",
                ["two in a wave.cue"] = "FILE \"a.wav\" WAVE\nTRACK 01 AUDIO\nINDEX 01 00:00:00\nTRACK 02 AUDIO\nINDEX 01 00:00:10\n",
                ["a file gone.cue"] = "FILE \"gone.bin\" BINARY\nTRACK 01 AUDIO\nINDEX 01 00:00:00\n",
            };
            foreach (var (name, text) in sheets) File.WriteAllText(Path.Combine(folder, name), text);
            foreach (string path in sheets.Keys.Select(k => Path.Combine(folder, k)).Append(Path.Combine(folder, "absent.cue")))
            {
                Exception managed = Assert.ThrowsAny<Exception>(() => RomHash.Managed.Tracks(path, default));
                Exception native = Assert.ThrowsAny<Exception>(() => GalaxiaNative.RomTracks(path, default));
                Assert.True(managed.GetType() == native.GetType(), $"{path}: {managed.GetType().Name} and {native.GetType().Name}");
            }
        }
    }
}
