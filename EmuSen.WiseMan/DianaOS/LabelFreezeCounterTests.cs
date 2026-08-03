using System.Linq;
using EmuSen.Cores.Nintendo.Venus.Debug;
using EmuSen.DianaOS;
using EmuSen.DianaOS.DianaOS.Bin;
using EmuSen.DianaOS.DianaOS.Var;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.DianaOS
{
    // `label`, `freeze` and `counters` - see their man pages.
    public class LabelFreezeCounterTests
    {
        private static SnesDebugTarget BuildTarget()
        {
            var core = SyntheticRom.LoadCore(SyntheticRom.BuildBlank());
            return new SnesDebugTarget(core.Cpu!, core.Bus!, core.Renderer!);
        }

        [Fact]
        public void A_label_names_an_address_in_both_directions()
        {
            var labels = new LabelRegistry();
            labels.Add(0x00A3B2, "NmiHandler");

            Assert.True(labels.TryGetName(0x00A3B2, out string name));
            Assert.Equal("NmiHandler", name);
            Assert.True(labels.TryGetAddress("nmihandler", out int address));
            Assert.Equal(0x00A3B2, address);
        }

        // Neither direction may end up with two entries - see `man label`.
        [Fact]
        public void Renaming_an_address_replaces_rather_than_duplicates()
        {
            var labels = new LabelRegistry();
            labels.Add(0x8000, "First");
            labels.Add(0x8000, "Second");

            Assert.Equal(1, labels.Count);
            Assert.False(labels.TryGetAddress("First", out _));
        }

        [Fact]
        public void Repointing_a_name_moves_it_rather_than_duplicating()
        {
            var labels = new LabelRegistry();
            labels.Add(0x8000, "Routine");
            labels.Add(0x9000, "Routine");

            Assert.Equal(1, labels.Count);
            Assert.False(labels.TryGetName(0x8000, out _));
            Assert.True(labels.TryGetName(0x9000, out _));
        }

        [Fact]
        public void A_label_file_round_trips()
        {
            var labels = new LabelRegistry();
            labels.Add(0x00A3B2, "NmiHandler", "main vblank entry");
            labels.Add(0x7E13C6, "CoinCount");

            var reloaded = new LabelRegistry();
            var (added, errors) = reloaded.LoadFromLines(labels.ToLines());

            Assert.Equal(2, added);
            Assert.Empty(errors);
            Assert.Equal("main vblank entry", reloaded.CommentAt(0x00A3B2));
        }

        [Fact]
        public void A_label_file_reports_bad_lines_without_dropping_good_ones()
        {
            var labels = new LabelRegistry();

            var (added, errors) = labels.LoadFromLines(new[]
            {
                "# a comment",
                "",
                "008000 Good",
                "notahexaddress Bad",
                "008010",
            });

            Assert.Equal(1, added);
            Assert.Equal(2, errors.Count);
        }

        [Fact]
        public void Label_at_reports_the_nearest_label_below_with_an_offset()
        {
            var target = BuildTarget();
            var shell = DianaOSInterpreter.CreateDefault(target);
            shell.Submit("label add 008000 MainLoop");

            Assert.Contains("MainLoop+18", shell.Submit("label at 008012").Output);
        }

        [Fact]
        public void Disasm_prints_a_label_on_its_own_line()
        {
            var target = BuildTarget();
            var shell = DianaOSInterpreter.CreateDefault(target);
            shell.Submit("label add 008000 ResetEntry");

            Assert.Contains("ResetEntry:", shell.Submit("disasm CpuBus 008000 2").Output);
        }

        [Fact]
        public void Bp_list_resolves_a_labelled_breakpoint_address()
        {
            var target = BuildTarget();
            var shell = DianaOSInterpreter.CreateDefault(target);
            shell.Submit("label add 008000 ResetEntry");
            shell.Submit("bp add 008000");

            Assert.Contains("<ResetEntry>", shell.Submit("bp list").Output);
        }

        // The difference from a per-frame cheat poke - see `man freeze`.
        [Fact]
        public void A_frozen_address_is_restored_the_instant_it_is_written()
        {
            var target = BuildTarget();
            var wram = target.GetMemorySpaces().First(s => s.Name == "WRAM");
            wram.Write(0x40, 0x63);
            DianaOSInterpreter.CreateDefault(target).Submit("freeze add WRAM 40");

            target.OnWrite("WRAM", 0x40, 0x00);

            Assert.Equal(0x63, wram.Read(0x40));
        }

        [Fact]
        public void Freeze_list_counts_the_writes_it_undid()
        {
            var target = BuildTarget();
            var shell = DianaOSInterpreter.CreateDefault(target);
            shell.Submit("freeze add WRAM 40 63");

            target.OnWrite("WRAM", 0x40, 0x01);
            target.OnWrite("WRAM", 0x40, 0x02);

            Assert.Contains("2 write(s) undone", shell.Submit("freeze list").Output);
        }

        // A write of the frozen value changes nothing and must not be counted.
        [Fact]
        public void A_write_of_the_frozen_value_is_not_counted_as_blocked()
        {
            var freezes = new FreezeRegistry();
            freezes.Add("WRAM", 0x40, 0x63);

            Assert.Null(freezes.NoteWrite("WRAM", 0x40, 0x63));
            Assert.Equal(0, freezes.All()[0].Blocked);
        }

        [Fact]
        public void Freezing_a_read_only_space_is_refused()
        {
            var target = BuildTarget();
            var readOnly = target.GetMemorySpaces().FirstOrDefault(s => !s.IsWritable);
            if (readOnly == null) return; // this cartridge exposes no read-only space

            var result = DianaOSInterpreter.CreateDefault(target).Submit($"freeze add {readOnly.Name} 0");

            Assert.Contains("not writable", result.Output);
        }

        [Fact]
        public void Counters_tally_reads_and_writes_separately()
        {
            var counters = new AccessCounterRegistry();
            counters.Arm("WRAM", 0x100);

            counters.NoteWrite("WRAM", 0x40);
            counters.NoteRead("WRAM", 0x40);
            counters.NoteRead("WRAM", 0x40);

            var at = counters.At(0x40);
            Assert.Equal(1u, at.Writes);
            Assert.Equal(2u, at.Reads);
        }

        // The read-before-write signal - see `man counters`.
        [Fact]
        public void A_read_before_any_write_counts_as_uninitialized()
        {
            var counters = new AccessCounterRegistry();
            counters.Arm("WRAM", 0x100);

            counters.NoteRead("WRAM", 0x40);
            counters.NoteWrite("WRAM", 0x40);
            counters.NoteRead("WRAM", 0x40);

            Assert.Equal(1u, counters.At(0x40).UninitializedReads);
        }

        [Fact]
        public void Counters_ignore_a_space_they_are_not_armed_for()
        {
            var counters = new AccessCounterRegistry();
            counters.Arm("WRAM", 0x100);

            counters.NoteWrite("VRAM", 0x40);

            Assert.Equal(0u, counters.At(0x40).Writes);
        }

        [Fact]
        public void A_disarmed_counter_records_nothing()
        {
            var counters = new AccessCounterRegistry();
            counters.Arm("WRAM", 0x100);
            counters.Disarm();

            counters.NoteWrite("WRAM", 0x40);

            Assert.Equal(0u, counters.At(0x40).Writes);
        }

        [Fact]
        public void Counters_cold_reports_addresses_nothing_touched()
        {
            var counters = new AccessCounterRegistry();
            counters.Arm("WRAM", 0x100);
            counters.NoteWrite("WRAM", 0x40);

            var cold = counters.Untouched(0x40, 4, 16);

            Assert.Equal(new[] { 0x41, 0x42, 0x43 }, cold);
        }

        [Fact]
        public void Counters_top_ranks_the_busiest_addresses()
        {
            var counters = new AccessCounterRegistry();
            counters.Arm("WRAM", 0x100);
            counters.NoteWrite("WRAM", 0x10);
            for (int i = 0; i < 5; i++) counters.NoteWrite("WRAM", 0x20);

            var top = counters.Hottest(AccessCounterRegistry.SortBy.Writes, 2);

            Assert.Equal(0x20, top[0].Address);
            Assert.Equal(0x10, top[1].Address);
        }

        // A real bus write must reach the counters, not just a direct call.
        [Fact]
        public void A_real_bus_write_reaches_the_counters()
        {
            var target = BuildTarget();
            DianaOSInterpreter.CreateDefault(target).Submit("counters on WRAM");

            target.OnWrite("WRAM", 0x1234, 0x5A);

            Assert.Equal(1u, target.AccessCounters!.At(0x1234).Writes);
        }
    }
}
