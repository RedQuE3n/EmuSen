using System;
using System.Collections.Generic;
using EmuSen.DianaOS.DianaOS.Lib;

namespace EmuSen.DianaOS.DianaOS.Etc
{
    // Commands parked while a published root contains its own install - see `man rm`.
    public static class DianaOSCommandSuspensions
    {
        private static readonly HashSet<string> Names = new(StringComparer.OrdinalIgnoreCase) { "rm", "mv" };

        public static bool IsSuspended(string name) => Names.Contains(name);

        public static DianaOSResult Refuse(string name) => DianaOSResult.Fail(
            $"{name}: suspended - a published build's own binaries live inside this root now " +
            $"(/lib/EmuSen), and nothing here distinguishes them from your data. See 'man {name}' and 'man hier'.");
    }
}
