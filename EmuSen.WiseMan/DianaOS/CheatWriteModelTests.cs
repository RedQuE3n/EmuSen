using System;
using System.Collections.Generic;
using System.Linq;
using EmuSen.DianaOS.DianaOS.Var;
using EmuSen.Galaxia.Models;

namespace EmuSen.WiseMan.DianaOS
{
    // The RetroArch-parity write model: widths, byte order, cheat types,
    // repeat runs and bit positions - see `man cheat`.
    public class CheatWriteModelTests
    {
        // A stand-in for one named memory space, so the registry can be
        // driven without a core - which is the whole point of it taking
        // read/write delegates rather than touching memory itself.
        private sealed class FakeSpace
        {
            private readonly Dictionary<int, byte> _bytes = new();

            public byte Read(string space, int address) => _bytes.TryGetValue(address, out byte b) ? b : (byte)0;
            public void Write(string space, int address, byte value) => _bytes[address] = value;

            public byte this[int address] => Read("", address);
            public void Set(int address, byte value) => _bytes[address] = value;
        }

        private static FakeSpace ApplyOnce(CheatRegistry registry)
        {
            var space = new FakeSpace();
            registry.ApplyAll(space.Read, space.Write);
            return space;
        }

        // --- Widths and byte order ---

        [Fact]
        public void A_two_byte_write_lands_little_endian_by_default()
        {
            var registry = new CheatRegistry();
            registry.AddCheat(CheatKind.RamPoke, new[] { CheatWrite.Poke("WRAM", 0x100, 0x1234, width: 2) }, null, "hp");

            FakeSpace space = ApplyOnce(registry);

            Assert.Equal(0x34, space[0x100]);
            Assert.Equal(0x12, space[0x101]);
        }

        [Fact]
        public void Big_endian_reverses_the_bytes_and_nothing_else()
        {
            var registry = new CheatRegistry();
            registry.AddCheat(CheatKind.RamPoke,
                new[] { new CheatWrite { Space = "WRAM", Address = 0x100, Value = 0x1234, Width = 2, BigEndian = true, RepeatCount = 1 } },
                null, "hp");

            FakeSpace space = ApplyOnce(registry);

            Assert.Equal(0x12, space[0x100]);
            Assert.Equal(0x34, space[0x101]);
        }

        [Fact]
        public void A_four_byte_write_covers_four_addresses()
        {
            var registry = new CheatRegistry();
            registry.AddCheat(CheatKind.RamPoke, new[] { CheatWrite.Poke("WRAM", 0x200, 0xDEADBEEF, width: 4) }, null, "score");

            FakeSpace space = ApplyOnce(registry);

            Assert.Equal(0xEF, space[0x200]);
            Assert.Equal(0xBE, space[0x201]);
            Assert.Equal(0xAD, space[0x202]);
            Assert.Equal(0xDE, space[0x203]);
        }

        // --- Cheat types ---

        [Fact]
        public void Increase_adds_to_whatever_is_already_there()
        {
            var registry = new CheatRegistry();
            registry.AddCheat(CheatKind.RamPoke,
                new[] { new CheatWrite { Space = "WRAM", Address = 0x10, Value = 5, Width = 1, Type = CheatWriteType.Increase, RepeatCount = 1 } },
                null, "creep up");

            var space = new FakeSpace();
            space.Set(0x10, 10);

            registry.ApplyAll(space.Read, space.Write);
            Assert.Equal(15, space[0x10]);

            // Applied every frame, so it keeps accumulating.
            registry.ApplyAll(space.Read, space.Write);
            Assert.Equal(20, space[0x10]);
        }

        [Fact]
        public void Decrease_subtracts_and_respects_width()
        {
            var registry = new CheatRegistry();
            registry.AddCheat(CheatKind.RamPoke,
                new[] { new CheatWrite { Space = "WRAM", Address = 0x10, Value = 0x0100, Width = 2, Type = CheatWriteType.Decrease, RepeatCount = 1 } },
                null, "drain");

            var space = new FakeSpace();
            space.Set(0x10, 0x00);
            space.Set(0x11, 0x05); // 0x0500 little-endian

            registry.ApplyAll(space.Read, space.Write);

            Assert.Equal(0x00, space[0x10]);
            Assert.Equal(0x04, space[0x11]); // 0x0500 - 0x0100
        }

        // --- Bit positions ---

        [Fact]
        public void A_bit_write_leaves_every_other_bit_alone()
        {
            var registry = new CheatRegistry();
            registry.AddCheat(CheatKind.RamPoke,
                new[] { new CheatWrite { Space = "WRAM", Address = 0x40, Value = 1, Width = 1, BitPosition = 3, RepeatCount = 1 } },
                null, "set flag");

            var space = new FakeSpace();
            space.Set(0x40, 0b0100_0001);

            registry.ApplyAll(space.Read, space.Write);

            Assert.Equal(0b0100_1001, space[0x40]);
        }

        [Fact]
        public void A_bit_write_of_zero_clears_that_bit_only()
        {
            var registry = new CheatRegistry();
            registry.AddCheat(CheatKind.RamPoke,
                new[] { new CheatWrite { Space = "WRAM", Address = 0x40, Value = 0, Width = 1, BitPosition = 6, RepeatCount = 1 } },
                null, "clear flag");

            var space = new FakeSpace();
            space.Set(0x40, 0b0100_0001);

            registry.ApplyAll(space.Read, space.Write);

            Assert.Equal(0b0000_0001, space[0x40]);
        }

        // --- Repeat runs ---

        [Fact]
        public void A_repeat_run_writes_a_whole_stride_of_addresses()
        {
            var registry = new CheatRegistry();
            registry.AddCheat(CheatKind.RamPoke,
                new[] { new CheatWrite { Space = "WRAM", Address = 0x300, Value = 0x63, Width = 1, RepeatCount = 8, RepeatAddAddress = 2 } },
                null, "max every item");

            FakeSpace space = ApplyOnce(registry);

            for (int i = 0; i < 8; i++) Assert.Equal(0x63, space[0x300 + i * 2]);
            Assert.Equal(0x00, space[0x301]); // the gaps stay untouched
            Assert.Equal(0x00, space[0x310]); // one past the end
        }

        [Fact]
        public void A_repeat_run_can_step_its_value_as_well_as_its_address()
        {
            var registry = new CheatRegistry();
            registry.AddCheat(CheatKind.RamPoke,
                new[] { new CheatWrite { Space = "WRAM", Address = 0x400, Value = 1, Width = 1, RepeatCount = 4, RepeatAddAddress = 1, RepeatAddValue = 1 } },
                null, "one of each item");

            FakeSpace space = ApplyOnce(registry);

            Assert.Equal(1, space[0x400]);
            Assert.Equal(2, space[0x401]);
            Assert.Equal(3, space[0x402]);
            Assert.Equal(4, space[0x403]);
        }

        // --- Multi-write cheats ---

        [Fact]
        public void One_cheat_can_drive_several_unrelated_addresses_under_one_toggle()
        {
            var registry = new CheatRegistry();
            int id = registry.AddCheat(CheatKind.RamPoke, new[]
            {
                CheatWrite.Poke("WRAM", 0x10, 0x63),
                CheatWrite.Poke("WRAM", 0x20, 0x09),
            }, null, "lives and coins");

            FakeSpace space = ApplyOnce(registry);
            Assert.Equal(0x63, space[0x10]);
            Assert.Equal(0x09, space[0x20]);

            // One toggle silences both - the whole reason writes are a list.
            registry.SetEnabled(id, false);
            var after = new FakeSpace();
            registry.ApplyAll(after.Read, after.Write);
            Assert.Equal(0x00, after[0x10]);
            Assert.Equal(0x00, after[0x20]);

            Assert.Single(registry.GetCheats());
            Assert.Equal(2, registry.GetCheats()[0].Writes.Count);
        }

        // --- ROM patches ---

        [Fact]
        public void A_multi_byte_rom_patch_answers_each_byte_of_its_value()
        {
            var registry = new CheatRegistry();
            registry.AddCheat(CheatKind.RomPatch, new[] { CheatWrite.Patch(0x00C05F, 0xEAEA, width: 2) }, null, "nop out");

            Assert.True(registry.TryPatchRom(0x00C05F, 0x00, out byte first));
            Assert.True(registry.TryPatchRom(0x00C060, 0x00, out byte second));
            Assert.False(registry.TryPatchRom(0x00C061, 0x00, out _));

            Assert.Equal(0xEA, first);
            Assert.Equal(0xEA, second);
        }

        [Fact]
        public void A_repeated_rom_patch_resolves_the_right_repetition()
        {
            var registry = new CheatRegistry();
            registry.AddCheat(CheatKind.RomPatch,
                new[] { new CheatWrite { Address = 0x1000, Value = 0x10, Width = 1, RepeatCount = 4, RepeatAddAddress = 0x10, RepeatAddValue = 1 } },
                null, "run");

            Assert.True(registry.TryPatchRom(0x1000, 0, out byte a));
            Assert.True(registry.TryPatchRom(0x1020, 0, out byte b));
            Assert.True(registry.TryPatchRom(0x1030, 0, out byte c));

            Assert.Equal(0x10, a);
            Assert.Equal(0x12, b);
            Assert.Equal(0x13, c);

            Assert.False(registry.TryPatchRom(0x1008, 0, out _)); // inside the stride, past the width
            Assert.False(registry.TryPatchRom(0x1040, 0, out _)); // one repetition past the end
        }

        // The hot path: every cartridge read calls this.
        [Fact]
        public void With_no_rom_patches_the_read_hook_declines_immediately()
        {
            var registry = new CheatRegistry();
            registry.AddRamPoke("WRAM", 0x10, 0x63, "poke only");

            Assert.False(registry.TryPatchRom(0x10, 0, out _));

            int id = registry.AddRomPatch(0x2000, 0xEA, null, "patch");
            Assert.True(registry.TryPatchRom(0x2000, 0, out _));

            registry.SetEnabled(id, false);
            Assert.False(registry.TryPatchRom(0x2000, 0, out _));
        }

        // --- The two combinations that cannot be honestly implemented ---

        [Fact]
        public void A_rom_patch_cannot_increase_because_there_is_nowhere_to_accumulate()
        {
            var registry = new CheatRegistry();

            var ex = Assert.Throws<ArgumentException>(() => registry.AddCheat(CheatKind.RomPatch,
                new[] { new CheatWrite { Address = 0x10, Value = 1, Width = 1, Type = CheatWriteType.Increase, RepeatCount = 1 } },
                null, "nonsense"));

            Assert.Contains("increase/decrease", ex.Message);
        }

        [Fact]
        public void A_compare_byte_is_rejected_on_a_multi_byte_rom_patch()
        {
            var registry = new CheatRegistry();

            var ex = Assert.Throws<ArgumentException>(() => registry.AddCheat(CheatKind.RomPatch,
                new[] { CheatWrite.Patch(0x10, 0x1234, width: 2) }, compare: 0x1F, "torn"));

            Assert.Contains("single-byte", ex.Message);
        }

        [Fact]
        public void A_cheat_needs_at_least_one_write()
        {
            var registry = new CheatRegistry();
            Assert.Throws<ArgumentException>(() =>
                registry.AddCheat(CheatKind.RamPoke, Array.Empty<CheatWrite>(), null, "empty"));
        }

        // --- Persistence ---

        [Fact]
        public void Every_write_field_survives_a_save_and_load()
        {
            var saved = new CheatRegistry();
            saved.AddCheat(CheatKind.RamPoke, new[]
            {
                new CheatWrite { Space = "WRAM", Address = 0x300, Value = 0x1234, Width = 2, BigEndian = true, Type = CheatWriteType.Increase, RepeatCount = 6, RepeatAddAddress = 4, RepeatAddValue = 2 },
                new CheatWrite { Space = "WRAM", Address = 0x40, Value = 1, Width = 1, BitPosition = 5, RepeatCount = 1 },
            }, null, "everything at once", enabled: false);

            var reloaded = new CheatRegistry();
            (int loaded, int skipped) = reloaded.LoadFrom(saved.ToCheatFile());

            Assert.Equal(1, loaded);
            Assert.Equal(0, skipped);

            CheatInfo cheat = reloaded.GetCheats().Single();
            Assert.False(cheat.Enabled);
            Assert.Equal("everything at once", cheat.Description);
            Assert.Equal(2, cheat.Writes.Count);

            CheatWrite first = cheat.Writes[0];
            Assert.Equal("WRAM", first.Space);
            Assert.Equal(0x300, first.Address);
            Assert.Equal(0x1234u, first.Value);
            Assert.Equal(2, first.EffectiveWidth);
            Assert.True(first.BigEndian);
            Assert.Equal(CheatWriteType.Increase, first.Type);
            Assert.Equal(6, first.EffectiveRepeatCount);
            Assert.Equal(4, first.RepeatAddAddress);
            Assert.Equal(2u, first.RepeatAddValue);

            Assert.Equal(5, cheat.Writes[1].BitPosition);
        }

        // Files written before a cheat became a list of writes.
        [Fact]
        public void A_pre_multi_write_file_still_loads_as_a_single_write()
        {
            var file = new CheatFile();
            file.Cheats.Add(new CheatFileEntry
            {
                Kind = "RamPoke",
                Space = "WRAM",
                Address = "7E0019",
                Value = "09",
                Description = "99 lives",
                Enabled = true,
            });
            file.Cheats.Add(new CheatFileEntry
            {
                Kind = "RomPatch",
                Address = "00C05F",
                Value = "EA",
                Compare = "1F",
                Description = "no clip",
                Enabled = false,
            });

            var registry = new CheatRegistry();
            (int loaded, int skipped) = registry.LoadFrom(file);

            Assert.Equal(2, loaded);
            Assert.Equal(0, skipped);

            CheatInfo poke = registry.GetCheats().Single(c => c.Kind == CheatKind.RamPoke);
            Assert.Equal("WRAM", poke.Writes[0].Space);
            Assert.Equal(0x7E0019, poke.Writes[0].Address);
            Assert.Equal(0x09u, poke.Writes[0].Value);
            Assert.Equal(1, poke.Writes[0].EffectiveWidth);

            CheatInfo patch = registry.GetCheats().Single(c => c.Kind == CheatKind.RomPatch);
            Assert.Equal((byte?)0x1F, patch.Compare);
        }

        [Fact]
        public void One_broken_entry_does_not_stop_the_rest_loading()
        {
            var file = new CheatFile();
            file.Cheats.Add(new CheatFileEntry { Kind = "RamPoke", Writes = { new CheatFileWrite { Space = "WRAM", Address = "ZZZZ", Value = "09" } }, Description = "broken" });
            file.Cheats.Add(new CheatFileEntry { Kind = "RamPoke", Writes = { new CheatFileWrite { Space = "WRAM", Address = "10", Value = "09" } }, Description = "fine" });

            var registry = new CheatRegistry();
            (int loaded, int skipped) = registry.LoadFrom(file);

            Assert.Equal(1, loaded);
            Assert.Equal(1, skipped);
            Assert.Equal("fine", registry.GetCheats().Single().Description);
        }
    }
}
