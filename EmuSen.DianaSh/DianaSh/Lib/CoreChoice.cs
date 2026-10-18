using System;
using System.Collections.Generic;
using System.Linq;

namespace EmuSen.DianaSh.DianaSh.Lib
{
    // A core `core <name>` may name: its display name, its console and the extensions it loads, as a frontend reads them from the frontend API's systems - see `man core`.
    public sealed record CoreChoice(string DisplayName, string Console, IReadOnlyList<string> Extensions)
    {
        public bool SupportsExtension(string extension) => Extensions.Any(e => e.Equals(extension, StringComparison.OrdinalIgnoreCase));
    }
}
