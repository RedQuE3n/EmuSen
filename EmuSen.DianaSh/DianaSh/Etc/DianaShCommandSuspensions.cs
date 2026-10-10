using System;
using System.Collections.Generic;
using EmuSen.DianaSh.DianaSh.Lib;

namespace EmuSen.DianaSh.DianaSh.Etc
{
    // Commands parked while a published root contains its own install - see `man rm`.
    public static class DianaShCommandSuspensions
    {
        private static readonly HashSet<string> Names = new(StringComparer.OrdinalIgnoreCase) { "rm", "mv" };

        public static bool IsSuspended(string name) => Names.Contains(name);

        public static DianaShResult Refuse(string name) => DianaShResult.Fail(
            $"{name}: suspended - a published build's own binaries live inside this root now " +
            $"(/lib/EmuSen), and nothing here distinguishes them from your data. See 'man {name}' and 'man hier'.");
    }
}
