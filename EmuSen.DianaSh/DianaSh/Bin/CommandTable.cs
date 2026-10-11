using System;
using System.Collections.Generic;
using System.Diagnostics.CodeAnalysis;
using System.Linq;
using EmuSen.DianaSh.DianaSh.Lib;

namespace EmuSen.DianaSh.DianaSh.Bin
{
    // The one owner of which name runs which command, the aliases, and which commands are read-only - see EmuSen_DianaSh.md §13.5.
    internal sealed class CommandTable
    {
        private static readonly Dictionary<string, string> Aliases = new(StringComparer.Ordinal)
        {
            ["c"] = "resume",
            ["quit"] = "shutdown",
            ["s"] = "step",
            ["jobs"] = "ps",
            ["hexdump"] = "xxd",
        };

        private readonly Dictionary<string, IDianaShCommand> _byName;

        public CommandTable(IReadOnlyList<IDianaShCommand> ordered)
        {
            Ordered = ordered;
            _byName = ordered.ToDictionary(c => c.Name, StringComparer.OrdinalIgnoreCase);
        }

        // In registration order, which is the order `help` lists them.
        public IReadOnlyList<IDianaShCommand> Ordered { get; }

        public IEnumerable<string> Names => _byName.Keys;

        // A name lower-cased, then mapped through the aliases; a name that is no alias comes back lower-cased.
        public static string Canonical(string name)
        {
            string lower = name.ToLowerInvariant();
            return Aliases.TryGetValue(lower, out string? command) ? command : lower;
        }

        public bool TryFind(string canonicalName, [MaybeNullWhen(false)] out IDianaShCommand command) => _byName.TryGetValue(canonicalName, out command);

        public bool IsReadOnly(string name) => TryFind(Canonical(name), out IDianaShCommand? command) && command.IsReadOnly;
    }
}
