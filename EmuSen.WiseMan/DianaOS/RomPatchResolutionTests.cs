using System;
using System.Collections.Generic;
using System.Linq;
using EmuSen.DianaOS.DianaOS.Var;

namespace EmuSen.WiseMan.DianaOS
{
    // The ROM patches resolved to bytes for a core that holds its own copy, against TryPatchRom itself - see Mars_Native.md §6.6.1.
    public class RomPatchResolutionTests
    {
        private const int Span = 0x200;

        // First entry at the address whose compare is none or the original, as MarsRT's list is read.
        private static bool Lookup(IReadOnlyList<RomPatchByte> bytes, uint address, byte original, out byte value)
        {
            foreach (RomPatchByte b in bytes)
            {
                if (b.Address != address || (b.Compare is byte compare && compare != original)) continue;
                value = b.Value;
                return true;
            }
            value = 0;
            return false;
        }

        private static void AddRandom(CheatRegistry registry, Random random)
        {
            int writes = random.Next(1, 4);
            bool compared = random.Next(3) == 0;
            var list = new List<CheatWrite>();
            for (int i = 0; i < writes; i++)
            {
                int width = compared ? 1 : new[] { 1, 2, 4, 3 }[random.Next(4)];
                list.Add(new CheatWrite
                {
                    Address = random.Next(Span),
                    Value = (uint)random.Next(),
                    Width = width,
                    BigEndian = random.Next(2) == 0,
                    RepeatCount = random.Next(4) == 0 ? random.Next(0, 6) : 1,
                    RepeatAddAddress = random.Next(-5, 9),
                    RepeatAddValue = (uint)random.Next(4),
                });
            }
            int id = registry.AddCheat(CheatKind.RomPatch, list, compared ? (byte)random.Next(4) : null, "random", enabled: random.Next(5) != 0);
            if (random.Next(8) == 0) registry.SetEnabled(id, false);
        }

        // Every address and every cartridge byte, over registries of overlapping, repeated, compared and disabled patches.
        [Fact]
        public void The_resolved_bytes_answer_every_read_as_TryPatchRom_does()
        {
            var random = new Random(0x0B5E55ED);
            int compared = 0, patched = 0;
            for (int round = 0; round < 60; round++)
            {
                var registry = new CheatRegistry();
                for (int i = random.Next(1, 7); i > 0; i--) AddRandom(registry, random);
                registry.AddRamPoke("RDRAM", 0x10, 0x63, "a poke is no patch");
                if (round % 10 == 9) registry.MasterEnabled = false;

                IReadOnlyList<RomPatchByte> bytes = registry.ResolveRomPatches();
                for (uint address = 0; address < Span + 64; address++)
                {
                    for (int original = 0; original < 8; original++)
                    {
                        bool want = registry.TryPatchRom(address, (byte)original, out byte wanted);
                        bool got = Lookup(bytes, address, (byte)original, out byte value);
                        Assert.True(want == got && wanted == value, $"round {round}, {address:X}, over {original:X2}: TryPatchRom {want} {wanted:X2}, resolved {got} {value:X2}");
                        compared++;
                        patched += want ? 1 : 0;
                    }
                }
            }
            Assert.True(patched > 1000, $"only {patched} of {compared} reads were patched");
        }

        [Fact]
        public void The_limit_drops_what_lies_past_it_and_the_version_moves_on_every_change()
        {
            var registry = new CheatRegistry();
            int start = registry.Version;
            int id = registry.AddCheat(CheatKind.RomPatch, new[] { CheatWrite.Patch(0x0FFE, 0xAABBCCDD, width: 4) }, null, "across the end");
            Assert.Equal(new uint[] { 0x0FFE, 0x0FFF }, registry.ResolveRomPatches(limit: 0x1000).Select(b => b.Address));

            int[] versions = { start, registry.Version, 0, 0, 0, 0 };
            registry.SetEnabled(id, false);
            versions[2] = registry.Version;
            registry.MasterEnabled = false;
            versions[3] = registry.Version;
            registry.RemoveCheat(id);
            versions[4] = registry.Version;
            registry.Clear();
            versions[5] = registry.Version;
            Assert.Equal(versions.Length, versions.Distinct().Count());
            Assert.Empty(registry.ResolveRomPatches());
        }
    }
}
