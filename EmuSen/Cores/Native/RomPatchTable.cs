using System.Collections.Generic;
using EmuSen.DianaOS.DianaOS.Var;

namespace EmuSen.Cores.Native
{
    // CheatRegistry.TryPatchRom flattened for the 8-bit Rust cores: the patched addresses, and 256 entries for each - see Moon_Native.md §2.4.
    public static class RomPatchTable
    {
        // Each entry is 0x100 | patched for an original byte a patch replaces, and 0 where none does; lo and hi are inclusive.
        public static (ushort[] Addresses, ushort[] Tables) Build(CheatRegistry cheats, int lo, int hi)
        {
            var addresses = new List<ushort>();
            var tables = new List<ushort>();
            if (cheats.EnabledRomPatches > 0)
            {
                var probes = new HashSet<byte> { 0 };
                foreach (var cheat in cheats.GetCheats())
                    if (cheat.Kind == CheatKind.RomPatch && cheat.Compare is byte compare) { probes.Add(compare); probes.Add(unchecked((byte)(compare + 1))); }

                for (int address = lo; address <= hi; address++)
                {
                    bool touched = false;
                    foreach (byte probe in probes)
                        if (cheats.TryPatchRom((uint)address, probe, out _)) { touched = true; break; }
                    if (!touched) continue;

                    addresses.Add((ushort)address);
                    for (int value = 0; value < 256; value++)
                        tables.Add(cheats.TryPatchRom((uint)address, (byte)value, out byte patched) ? (ushort)(0x100 | patched) : (ushort)0);
                }
            }
            return (addresses.ToArray(), tables.ToArray());
        }
    }
}
