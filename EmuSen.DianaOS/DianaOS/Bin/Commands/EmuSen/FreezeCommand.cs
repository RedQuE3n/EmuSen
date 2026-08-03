using System;
using System.Linq;
using EmuSen.DianaOS.DianaOS.Lib;
using EmuSen.Galaxia.Text;
using static EmuSen.DianaOS.DianaOS.Bin.Commands.EmuSen.DebugCommandHelpers;

namespace EmuSen.DianaOS.DianaOS.Bin.Commands.EmuSen
{
    // Pins an address by undoing every write to it - see `man freeze`.
    public class FreezeCommand : IDianaOSCommand
    {
        public string Name => "freeze";
        public bool IsReadOnly => false;
        public string Usage => string.Join('\n', new[]
        {
            "  freeze add <space> <addr> [<value>]  pin <addr>; without <value>, at whatever it holds now",
            "  freeze list                          list frozen addresses and how many writes each blocked",
            "  freeze remove <id>                   unfreeze one address",
            "  freeze clear                         unfreeze everything",
        });

        public DianaOSResult Execute(IDebugTarget? target, string[] parts, string? stdin)
        {
            IDebugTarget resolved;
            try { resolved = RequireTarget(target); }
            catch (Exception ex) { return DianaOSResult.Fail(ex.Message); }

            var freezes = resolved.Freezes;
            if (freezes == null) return DianaOSResult.Fail("This core does not support frozen addresses.");
            if (parts.Length < 2) return "Usage: freeze add|list|remove|clear ...";

            switch (parts[1].ToLowerInvariant())
            {
                case "add":
                {
                    if (parts.Length < 4) return "Usage: freeze add <space> <addr> [<value>]";
                    var space = FindSpace(resolved, parts[2]);
                    if (!space.IsWritable) return DianaOSResult.Fail($"{space.Name} is not writable, so nothing can write to it to be undone.");
                    int address = ParseHex(parts[3]);
                    if (address < 0 || address >= space.Size) return DianaOSResult.Fail($"0x{address:X} is outside {space.Name} (0x0-0x{space.Size - 1:X}).");
                    byte value = parts.Length > 4 ? (byte)ParseHex(parts[4]) : space.Read(address);
                    int id = freezes.Add(space.Name, address, value);
                    return $"Frozen #{id}: {space.Name} 0x{address:X} = 0x{value:X2}.";
                }
                case "list":
                {
                    var all = freezes.All();
                    if (all.Count == 0) return "No frozen addresses.";
                    return string.Join('\n', all.Select(f =>
                        $"  #{f.Id}: {f.Space} 0x{f.Address:X} = 0x{f.Value:X2} ({f.Blocked} write(s) undone)"));
                }
                case "remove":
                {
                    if (parts.Length < 3) return "Usage: freeze remove <id>";
                    int id = ParseHex(parts[2]);
                    return freezes.Remove(id) ? $"Frozen address #{id} released." : DianaOSResult.Fail($"No frozen address #{id}.");
                }
                case "clear":
                {
                    int count = freezes.Count;
                    freezes.Clear();
                    return $"{count} frozen address(es) released.";
                }
                default:
                {
                    string[] subcommands = { "add", "list", "remove", "clear" };
                    return $"Unknown 'freeze' subcommand '{parts[1]}'.{Suggestion.Hint(parts[1], subcommands)} Try {string.Join('/', subcommands)}.";
                }
            }
        }
    }
}
