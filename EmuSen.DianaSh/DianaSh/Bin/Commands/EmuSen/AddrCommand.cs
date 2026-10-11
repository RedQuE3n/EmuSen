using System;
using EmuSen.DianaSh.DianaSh.Bin;
using EmuSen.DianaSh.DianaSh.Etc;
using EmuSen.DianaSh.DianaSh.Lib;
using EmuSen.Crystal.Debugger;
using EmuSen.DianaSh.DianaSh.Dev;
using static EmuSen.DianaSh.DianaSh.Bin.Commands.EmuSen.DebugCommandHelpers;

namespace EmuSen.DianaSh.DianaSh.Bin.Commands.EmuSen
{
    // Decodes a CPU-bus address to the physical location it reaches - see `man addr`.
    public class AddrCommand : IDianaShCommand
    {
        public string Name => "addr";
        public bool IsReadOnly => true;
        public string Usage => "  addr <addr>                   what a CPU-bus address decodes to (ROM offset, RAM, a register)";

        public DianaShResult Execute(IDebugTarget? target, string[] parts, string? stdin)
        {
            IDebugTarget resolved;
            try { resolved = RequireTarget(target); }
            catch (Exception ex) { return DianaShResult.Fail(ex.Message); }

            if (parts.Length < 2) return "Usage: addr <addr>";

            int address;
            try { address = ParseHex(parts[1]); }
            catch (Exception) { return DianaShResult.Fail($"'{parts[1]}' is not an address."); }

            if (resolved.ResolvePhysical(address) is not { } physical)
                return $"{resolved.CoreName} does not report an address decode, or ${address:X6} is unmapped.";

            string label = resolved.Labels?.Describe(address) ?? $"${address:X6}";
            if (!physical.IsAddressable) return $"{label} -> {physical.Space}";

            return $"{label} -> {physical.Space} 0x{physical.Offset:X}"
                 + $"\n  mem {physical.Space} {physical.Offset:X} 10";
        }
    }
}
