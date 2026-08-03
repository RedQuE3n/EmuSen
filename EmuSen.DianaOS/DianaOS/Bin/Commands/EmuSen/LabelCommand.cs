using System;
using System.IO;
using System.Linq;
using EmuSen.DianaOS.DianaOS.Etc;
using EmuSen.DianaOS.DianaOS.Lib;
using EmuSen.Galaxia.Text;
using static EmuSen.DianaOS.DianaOS.Bin.Commands.EmuSen.DebugCommandHelpers;

namespace EmuSen.DianaOS.DianaOS.Bin.Commands.EmuSen
{
    // Named addresses, shown by disasm/bt/bp and usable in expressions - see `man label`.
    public class LabelCommand : IDianaOSCommand
    {
        public string Name => "label";
        public bool IsReadOnly => false;
        public string Usage => string.Join('\n', new[]
        {
            "  label add <addr> <name> [<comment...>]   name an address",
            "  label list [<filter>]                    list labels, address order",
            "  label at <addr>                          what is named at (or nearest below) <addr>",
            "  label remove <name|addr>                 drop one label",
            "  label clear                              drop every label",
            "  label load <path>                        merge a label file in",
            "  label save <path>                        write every label out",
        });

        public DianaOSResult Execute(IDebugTarget? target, string[] parts, string? stdin)
        {
            IDebugTarget resolved;
            try { resolved = RequireTarget(target); }
            catch (Exception ex) { return DianaOSResult.Fail(ex.Message); }

            var labels = resolved.Labels;
            if (labels == null) return DianaOSResult.Fail("This core does not support labels.");
            if (parts.Length < 2) return "Usage: label add|list|at|remove|clear|load|save ...";

            string sub = parts[1].ToLowerInvariant();
            switch (sub)
            {
                case "add":
                {
                    if (parts.Length < 4) return "Usage: label add <addr> <name> [<comment...>]";
                    int address = ParseHex(parts[2]);
                    string comment = parts.Length > 4 ? string.Join(' ', parts.Skip(4)) : string.Empty;
                    labels.Add(address, parts[3], comment);
                    return $"${address:X6} is now '{parts[3]}'.";
                }
                case "list":
                {
                    string filter = parts.Length > 2 ? parts[2] : string.Empty;
                    var rows = labels.All()
                        .Where(l => filter.Length == 0 || l.Name.Contains(filter, StringComparison.OrdinalIgnoreCase))
                        .ToList();
                    if (rows.Count == 0) return filter.Length == 0 ? "No labels defined." : $"No label matches '{filter}'.";
                    return string.Join('\n', rows.Select(l => $"  ${l.Address:X6}  {l.Name}{(l.Comment == null ? string.Empty : $"  ; {l.Comment}")}"));
                }
                case "at":
                {
                    if (parts.Length < 3) return "Usage: label at <addr>";
                    int address = ParseHex(parts[2]);
                    if (labels.TryGetName(address, out string exact)) return $"  ${address:X6}  {exact}";
                    var below = labels.All().LastOrDefault(l => l.Address <= address);
                    if (below.Name == null) return $"Nothing labelled at or below ${address:X6}.";
                    return $"  ${below.Address:X6}  {below.Name}+{address - below.Address}";
                }
                case "remove":
                {
                    if (parts.Length < 3) return "Usage: label remove <name|addr>";
                    string key = parts[2];
                    if (labels.RemoveByName(key)) return $"Label '{key}' removed.";
                    try
                    {
                        int address = ParseHex(key);
                        if (labels.RemoveByAddress(address)) return $"Label at ${address:X6} removed.";
                    }
                    catch (Exception) { /* not an address either - fall through */ }
                    return DianaOSResult.Fail($"No label named '{key}'.");
                }
                case "clear":
                {
                    int count = labels.Count;
                    labels.Clear();
                    return $"{count} label(s) cleared.";
                }
                case "load":
                {
                    if (parts.Length < 3) return "Usage: label load <path>";
                    if (!DianaOSSandbox.TryResolve(parts[2], out string path)) return DianaOSResult.Fail($"Path outside this sandbox: {parts[2]}");
                    if (!File.Exists(path)) return DianaOSResult.Fail($"No such file: {parts[2]}");
                    var (added, errors) = labels.LoadFromLines(File.ReadAllLines(path));
                    string report = $"{added} label(s) loaded from {parts[2]}.";
                    return errors.Count == 0 ? report : $"{report}\n" + string.Join('\n', errors.Select(e => $"  {e}"));
                }
                case "save":
                {
                    if (parts.Length < 3) return "Usage: label save <path>";
                    if (!DianaOSSandbox.TryResolve(parts[2], out string path)) return DianaOSResult.Fail($"Path outside this sandbox: {parts[2]}");
                    File.WriteAllLines(path, labels.ToLines());
                    return $"{labels.Count} label(s) written to {parts[2]}.";
                }
                default:
                {
                    string[] subcommands = { "add", "list", "at", "remove", "clear", "load", "save" };
                    return $"Unknown 'label' subcommand '{sub}'.{Suggestion.Hint(sub, subcommands)} Try {string.Join('/', subcommands)}.";
                }
            }
        }
    }
}
