using System;
using EmuSen.DianaOS.DianaOS.Bin;
using EmuSen.DianaOS.DianaOS.Etc;
using EmuSen.DianaOS.DianaOS.Lib;
using EmuSen.DianaOS.DianaOS.Var;
using EmuSen.DianaOS.DianaOS.Dev;
using static EmuSen.DianaOS.DianaOS.Bin.Commands.EmuSen.DebugCommandHelpers;

namespace EmuSen.DianaOS.DianaOS.Bin.Commands.EmuSen
{
    // Decodes a CPU-bus address to the physical location it reaches - see `man addr`.
    public class AddrCommand : IDianaOSCommand
    {
        public string Name => "addr";
        public bool IsReadOnly => true;
        public string Usage => "  addr <addr>                   what a CPU-bus address decodes to (ROM offset, RAM, a register)";

        public DianaOSResult Execute(IDebugTarget? target, string[] parts, string? stdin)
        {
            IDebugTarget resolved;
            try { resolved = RequireTarget(target); }
            catch (Exception ex) { return DianaOSResult.Fail(ex.Message); }

            if (parts.Length < 2) return "Usage: addr <addr>";

            int address;
            try { address = ParseHex(parts[1]); }
            catch (Exception) { return DianaOSResult.Fail($"'{parts[1]}' is not an address."); }

            if (resolved.ResolvePhysical(address) is not { } physical)
                return $"{resolved.CoreName} does not report an address decode, or ${address:X6} is unmapped.";

            string label = resolved.Labels?.Describe(address) ?? $"${address:X6}";
            if (!physical.IsAddressable) return $"{label} -> {physical.Space}";

            return $"{label} -> {physical.Space} 0x{physical.Offset:X}"
                 + $"\n  mem {physical.Space} {physical.Offset:X} 10";
        }
    }
}
