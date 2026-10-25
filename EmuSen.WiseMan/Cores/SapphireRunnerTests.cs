using System.Text;
using EmuSen.Cores;
using EmuSen.Cores.Native;
using EmuSen.Crystal.Media;
using EmuSen.Galaxia.Library;
using EmuSen.WiseMan.Fixtures;
using EmuSen.WiseMan.Fixtures.RomRunner;
using Xunit.Abstractions;

namespace EmuSen.WiseMan.Cores
{
    // Sapphire's stage S4 runners, each with a positive and a negative control: the synthetic program, the synthetic discs, the expected-text grader and the RAM-record reader - see Sapphire_Native.md §14.
    [Collection(TestCollections.ProcessGlobals)]
    public class SapphireRunnerTests : IDisposable
    {
        private readonly ITestOutputHelper _output;
        private readonly string _root = Path.Combine(Path.GetTempPath(), "EmuSenSapphireRunner_" + Guid.NewGuid().ToString("N"));

        public SapphireRunnerTests(ITestOutputHelper output)
        {
            _output = output;
            Directory.CreateDirectory(_root);
            DataStore.OverrideDirectory = Path.Combine(_root, "Home");
        }

        public void Dispose()
        {
            DataStore.OverrideDirectory = null;
            try { Directory.Delete(_root, recursive: true); } catch (IOException) { }
        }

        private string Write(string name, byte[] image)
        {
            string path = Path.Combine(_root, name);
            Directory.CreateDirectory(Path.GetDirectoryName(path)!);
            File.WriteAllBytes(path, image);
            return path;
        }

        private static readonly (uint, byte[])[] NoFiles = Array.Empty<(uint, byte[])>();

        // record.s's twelve words, the values the architecture fixes (Sapphire_Referees.md §4.1).
        public static readonly uint[] RecordWords = { 0x8000_0000, 0xF800_0000, 0x0800_0000, 1, 0, 1, 0x3FFF_FFFF, 0xFFFF_FFFD, 0xFFFF_FFFF, 0x13BA, 1, 0xEEDD_BB80 };

        private static (uint Entry, uint Offset, uint Address, uint File, uint Memory) Segment(byte[] elf) =>
            (BitConverter.ToUInt32(elf, 24), BitConverter.ToUInt32(elf, 52 + 4), BitConverter.ToUInt32(elf, 52 + 8), BitConverter.ToUInt32(elf, 52 + 16), BitConverter.ToUInt32(elf, 52 + 20));

        // The program's header read by the ELF format's fields, and Sapphire's reader taking it; spoiled forms refused with words.
        [Fact]
        public void The_synthetic_program_is_an_ee_executable_and_its_spoiled_forms_are_refused()
        {
            var library = CoreLibrary.Open(SapphireTests.LibraryPath);
            Assert.True(library.Available, library.Report);
            byte[] recording = SyntheticPs2Elf.Recording(RecordWords);
            uint[] code = RamRecordForm.Sapphire.Program(RecordWords);
            Assert.Equal((SyntheticPs2Elf.Entry, 84u, SyntheticPs2Elf.Entry, (uint)code.Length * 4, (uint)code.Length * 4), Segment(recording));
            Assert.Equal(code, Enumerable.Range(0, code.Length).Select(i => BitConverter.ToUInt32(recording, 84 + 4 * i)));
            byte[] withBss = SyntheticPs2Elf.Build(origin: 0x0020_0000, bss: 0x1000);
            Assert.Equal((0x0020_0000u, 84u, 0x0020_0000u, 8u, 8u + 0x1000), Segment(withBss));
            foreach (byte[] good in new[] { recording, withBss })
            {
                using var machine = new CoreMachine(library, good, "", NoFiles);
                Assert.Equal("ps2", machine.Info.System);
            }
            foreach (var (image, words) in new (byte[], string)[]
            {
                (SyntheticPs2Elf.Build(machine: 40), "not MIPS"),
                (SyntheticPs2Elf.Build(type: SyntheticPs2Elf.IopModule), "IOP module"),
                (SyntheticPs2Elf.Build(elfClass: 2), "64-bit"),
                (recording[..^8], "its headers and segments take"),
            })
            {
                var refused = Assert.Throws<CoreRefusedException>(() => new CoreMachine(library, image, "", NoFiles));
                _output.WriteLine(refused.Message);
                Assert.Contains(words, refused.Message);
            }
        }

        // Every form read back by the host's ISO 9660 reader and taken by Sapphire; a dual-layer image's file past the break read whole, layer 0's descriptor naming its sectors and layer 1's at its sector 16.
        [Fact]
        public void Each_synthetic_disc_reads_back_through_the_hosts_reader_and_sapphire_takes_it()
        {
            var library = CoreLibrary.Open(SapphireTests.LibraryPath);
            var dual = SyntheticPs2Disc.DualLayerDvd();
            foreach (var (label, image) in new (string, byte[])[] { ("cd iso", SyntheticPs2Disc.Cd().Iso), ("cd track", SyntheticPs2Disc.Cd().DataTrack), ("dvd", SyntheticPs2Disc.Dvd().Iso), ("dual-layer dvd", dual.Iso) })
            {
                var sectors = new DiscSectors(image);
                Assert.StartsWith("BOOT2 = cdrom0:\\EMUS_000.02;1", Encoding.ASCII.GetString(sectors.RootFile("SYSTEM.CNF") ?? Array.Empty<byte>()));
                Assert.Equal(SyntheticPs2Elf.Build(), sectors.RootFile(SyntheticPs2Disc.Program));
                using var machine = new CoreMachine(library, image, "", NoFiles);
                Assert.Equal("ps2", machine.Info.System);
                _output.WriteLine($"{label}: {image.Length} bytes");
            }
            Assert.Equal(1024, dual.LayerBreak);
            Assert.Equal(SyntheticPs2Disc.Layer1Pattern(3 * CdSector.User + 100), new DiscSectors(dual.Iso).RootFile(SyntheticPs2Disc.Layer1File));
            byte[] first = new DiscSectors(dual.Iso).Sector(16).ToArray(), second = new DiscSectors(dual.Iso).Sector(dual.LayerBreak!.Value + 16).ToArray();
            Assert.Equal(0xFF, new DiscSectors(dual.Iso).Sector(dual.LayerBreak.Value + 17)[0]);
            Assert.Equal(dual.LayerBreak, BitConverter.ToInt32(first, 80));
            Assert.Equal(dual.Iso.Length / CdSector.User - dual.LayerBreak, BitConverter.ToInt32(second, 80));
            Assert.True(first.AsSpan(0, 80).SequenceEqual(second.AsSpan(0, 80)) && first.AsSpan(88).SequenceEqual(second.AsSpan(88)), "the descriptors differ in their volume space size alone");

            string cue = SyntheticPs2Disc.Cd().WriteSet(_root, "cd");
            Assert.Equal(("cd.cue", "FILE \"cd.bin\" BINARY\n  TRACK 01 MODE2/2352\n    INDEX 01 00:00:00\n"), (Path.GetFileName(cue), File.ReadAllText(cue)));
            Assert.Equal(SyntheticPs2Disc.Cd().DataTrack, File.ReadAllBytes(Path.Combine(_root, "cd.bin")));
            Assert.Equal("dvd.iso", Path.GetFileName(SyntheticPs2Disc.Dvd().WriteSet(_root, "dvd")));
        }

        // A disc without BOOT2 refused; a single-layer image has no second volume; a layer 0 that breaks ECMA-267's rule of 16, or cannot hold the volume's start, refused by the builder.
        [Fact]
        public void A_disc_that_names_no_program_is_refused_and_the_builder_refuses_a_layer_break_off_the_rule()
        {
            var library = CoreLibrary.Open(SapphireTests.LibraryPath);
            var refused = Assert.Throws<CoreRefusedException>(() => new CoreMachine(library, SyntheticPs2Disc.DualLayerDvd(config: "VMODE = PAL\r\n").Iso, "", NoFiles));
            Assert.Contains("BOOT2", refused.Message);
            var single = SyntheticPs2Disc.Dvd();
            Assert.Null(single.LayerBreak);
            var sectors = new DiscSectors(single.Iso);
            Assert.DoesNotContain(Enumerable.Range(1, single.Iso.Length / CdSector.User / 16), n => sectors.Sector(n * 16 + 16) is { Length: > 6 } s && s[0] == 1 && s.Slice(1, 5).SequenceEqual("CD001"u8));
            Assert.Null(sectors.RootFile(SyntheticPs2Disc.Layer1File));
            Assert.Throws<ArgumentException>(() => SyntheticPs2Disc.DualLayerDvd(1000));
            Assert.Throws<ArgumentException>(() => SyntheticPs2Disc.DualLayerDvd(256));
        }

        // The grader's rule on text: line ends normalised, a kernel's lines before the first left out, a cut-off run incomplete, any other difference a failure named by its line.
        [Fact]
        public void The_expected_text_grader_passes_only_the_whole_text_with_line_ends_normalised()
        {
            const string expected = "-- TEST BEGIN\r\nalu:\r\n  addu 1, 1: 00000002\r\n\r\n-- TEST END\r\n";
            foreach (var (output, outcome, detail) in new (string?, RomOutcome, string)[]
            {
                ("-- TEST BEGIN\nalu:\n  addu 1, 1: 00000002\n\n-- TEST END\n", RomOutcome.Passed, "5 lines"),
                ("kernel boot\n-- TEST BEGIN\ralu:\r  addu 1, 1: 00000002\r\r-- TEST END", RomOutcome.Passed, "5 lines"),
                ("-- TEST BEGIN\nalu:\n  addu 1, 1: 00000003\n\n-- TEST END\n", RomOutcome.Failed, "line 3"),
                ("-- TEST BEGIN\nalu:\n  addu 1, 1: 00000002\n-- TEST END\n", RomOutcome.Failed, "line 4"),
                ("-- TEST BEGIN\nalu:\n  addu 1, 1: 00000002\n\n-- TEST END\nmore\n", RomOutcome.Failed, "past the end"),
                ("-- TEST BEGIN\nalu:\n  addu 1, 1: 0000", RomOutcome.Incomplete, "line 3"),
                ("-- TEST BEGIN\nalu:\n", RomOutcome.Incomplete, "2 of 5"),
                ("kernel boot only\n", RomOutcome.Incomplete, "never printed"),
                (null, RomOutcome.NoDump, "no output"),
            })
            {
                var verdict = ExpectedText.Grade("alu", output, expected);
                _output.WriteLine($"{verdict.Outcome}: {verdict.Detail}");
                Assert.Equal(outcome, verdict.Outcome);
                Assert.Contains(detail, verdict.Detail);
            }
        }

        // End to end on a core that prints: Diamond's TTY through the generic runner, graded by a protocol given as data; the corpus form claims only programs with an expected text.
        [Fact]
        public void The_expected_text_grader_grades_a_cores_tty_through_the_generic_runner()
        {
            var engine = new CoreAbiTestRomEngine(DiamondTests.LibraryPath);
            Assert.True(engine.Available, engine.Report);
            const string expected = "-- TEST BEGIN\r\nlw: 13371337\r\n-- TEST END\r\n";
            var grader = new TestRomGrader(new[] { ExpectedText.Protocol("cpu/ee/lsudelay.elf", expected) });
            foreach (var (printed, outcome) in new[] { ("-- TEST BEGIN\nlw: 13371337\n-- TEST END\n", RomOutcome.Passed), ("-- TEST BEGIN\nlw: 13371338\n-- TEST END\n", RomOutcome.Failed) })
            {
                var run = engine.Run(Write("lsudelay.exe", SyntheticPsExe.Printing(printed)), new[] { 2 });
                var verdict = grader.Grade("tests/cpu/ee/lsudelay.elf", run);
                _output.WriteLine($"{verdict.Outcome}: {verdict.Detail}");
                Assert.Equal(outcome, verdict.Outcome);
            }
            Assert.Equal(RomOutcome.Visual, grader.Grade("tests/cpu/ee/alu.elf", engine.Run(Write("alu.exe", SyntheticPsExe.Printing("x")), new[] { 1 })).Outcome);

            string corpus = Path.Combine(_root, "corpus");
            Write(Path.Combine("corpus", "cpu", "ee", "alu.elf"), SyntheticPs2Elf.Build());
            File.WriteAllText(Path.Combine(corpus, "cpu", "ee", "alu.expected"), expected);
            Write(Path.Combine("corpus", "cpu", "iop", "lone.irx"), SyntheticPs2Elf.Build(type: SyntheticPs2Elf.IopModule));
            Assert.Equal(new[] { "expected:cpu/ee/alu.elf" }, ExpectedText.Corpus(corpus).Select(p => p.Name));
        }

        // ps2autotests where fetched (opt-in): every program has its expected text, 55 for the EE and 23 for the IOP (Sapphire_Plan.md §3.1).
        [Fact]
        public void The_fetched_ps2autotests_are_78_programs_with_their_expected_texts()
        {
            if (Environment.GetEnvironmentVariable("EMUSEN_PS2AUTOTESTS") is not { Length: > 0 } root || !Directory.Exists(root)) return;
            var corpus = ExpectedText.Corpus(root);
            Assert.Equal((78, 55, 23), (corpus.Count, corpus.Count(p => p.Name.EndsWith(".elf")), corpus.Count(p => p.Name.EndsWith(".irx"))));
        }

        // The reader on snapshots laid out at each form's documented addresses, not the form's own: the words once the done word is there, each wrong word named, nothing without it.
        [Fact]
        public void The_ram_record_reader_reads_words_only_behind_the_done_word()
        {
            foreach (var (form, size, doneAt, doneWord, from) in new[] { (RamRecordForm.Sapphire, 32 << 20, 0x1F_FFF0, 0x5A99_D00Eu, 0x20_0000), (RamRecordForm.Diamond, 2 << 20, 0xF_FFF0, 0xD1A0_D00Eu, 0x10_0000) })
            {
                byte[] Ram(bool done, int spoil = -1)
                {
                    var ram = new byte[size];
                    for (int i = 0; i < RecordWords.Length; i++) BitConverter.TryWriteBytes(ram.AsSpan(from + 4 * i), RecordWords[i] ^ (i == spoil ? 1u : 0));
                    if (done) BitConverter.TryWriteBytes(ram.AsSpan(doneAt), doneWord);
                    return ram;
                }
                Assert.Equal(RecordWords, form.Read(Run(Ram(true)).Snapshots[0], RecordWords.Length));
                foreach (var (run, outcome, detail) in new (RomRun, RomOutcome, string)[]
                {
                    (Run(Ram(true)), RomOutcome.Passed, "12 words"),
                    (Run(Ram(true, spoil: 7)), RomOutcome.Failed, "word 7: FFFFFFFC against FFFFFFFD"),
                    (Run(Ram(false)), RomOutcome.Incomplete, "no done word"),
                    (Run(null), RomOutcome.NoDump, "no ram"),
                })
                {
                    var verdict = form.Grade(run, RecordWords);
                    Assert.Equal((form.Name, outcome, true), (form.Name, verdict.Outcome, verdict.Detail.Contains(detail)));
                }
            }
            var small = new byte[0x20_0000];
            BitConverter.TryWriteBytes(small.AsSpan(0x1F_FFF0), 0x5A99_D00Eu);
            Assert.Null(RamRecordForm.Sapphire.Read(Run(small).Snapshots[0], 4));
        }

        private static RomRun Run(byte[]? ram) => new("synthetic", new[] { new RomSnapshot(1, ram is null ? new Dictionary<string, byte[]>() : new Dictionary<string, byte[]> { ["ram"] = ram }, null) }, 0, Array.Empty<short>());

        // End to end on Diamond, the form's other console: its program leaves the words where the reader finds them; read in Sapphire's form, the same run has no record.
        [Fact]
        public void The_ram_record_reader_reads_a_record_diamond_left_and_not_one_in_another_form()
        {
            var engine = new CoreAbiTestRomEngine(DiamondTests.LibraryPath);
            Assert.True(engine.Available, engine.Report);
            var run = engine.Run(Write("record.exe", SyntheticPsExe.Build(SyntheticPsExe.Words(RamRecordForm.Diamond.Program(RecordWords)))), new[] { 2 });
            Assert.Equal(RomOutcome.Passed, new TestRomGrader(new[] { RamRecordForm.Diamond.Protocol("record.exe", RecordWords) }).Grade(Path.Combine(_root, "record.exe"), run).Outcome);
            Assert.Equal(RomOutcome.Incomplete, RamRecordForm.Sapphire.Grade(run, RecordWords).Outcome);
            var silent = engine.Run(Write("silent.exe", SyntheticPsExe.Build()), new[] { 2 });
            Assert.Equal(RomOutcome.Incomplete, RamRecordForm.Diamond.Grade(silent, RecordWords).Outcome);
        }

        // The recording program for the referees, written where EMUSEN_SAPPHIRE_PROGRAMS names a folder, for ps2referee.py to run (Sapphire_Native.md §14.3).
        [Fact]
        public void The_recording_program_is_written_for_the_referees_when_asked()
        {
            if (Environment.GetEnvironmentVariable("EMUSEN_SAPPHIRE_PROGRAMS") is not { Length: > 0 } dir || !Directory.Exists(dir)) return;
            File.WriteAllBytes(Path.Combine(dir, "recording.elf"), SyntheticPs2Elf.Recording(RecordWords));
            File.WriteAllBytes(Path.Combine(dir, "machine40.elf"), SyntheticPs2Elf.Build(RamRecordForm.Sapphire.Program(RecordWords), machine: 40));
        }
    }
}
