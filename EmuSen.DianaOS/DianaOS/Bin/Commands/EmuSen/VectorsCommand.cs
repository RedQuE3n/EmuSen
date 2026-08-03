using System;
using System.Collections.Generic;
using System.Linq;
using EmuSen.DianaOS.DianaOS.Bin;
using EmuSen.DianaOS.DianaOS.Etc;
using EmuSen.DianaOS.DianaOS.Lib;
using EmuSen.DianaOS.DianaOS.Var;
using EmuSen.DianaOS.DianaOS.Dev;
using static EmuSen.DianaOS.DianaOS.Bin.Commands.EmuSen.DebugCommandHelpers;

namespace EmuSen.DianaOS.DianaOS.Bin.Commands.EmuSen
{
    // Dereferences the interrupt vector table - see `man vectors`.
    public class VectorsCommand : IDianaOSCommand
    {
        public string Name => "vectors";
        public bool IsReadOnly => true;
        public string Usage => "  vectors                       where each interrupt vector points, resolved through labels";

        public DianaOSResult Execute(IDebugTarget? target, string[] parts, string? stdin)
        {
            IDebugTarget resolved;
            try { resolved = RequireTarget(target); }
            catch (Exception ex) { return DianaOSResult.Fail(ex.Message); }

            var vectors = resolved.InterruptVectors;
            if (vectors.Count == 0) return $"{resolved.CoreName} does not report an interrupt vector table.";

            var space = resolved.GetMemorySpaces()
                .FirstOrDefault(s => string.Equals(s.Name, "CpuBus", StringComparison.OrdinalIgnoreCase));
            if (space == null) return DianaOSResult.Fail("This core publishes no CpuBus space to read the vectors from.");

            var labels = resolved.Labels;
            var coverage = resolved.Coverage;
            var lines = new List<string>();
            string mode = string.Empty;

            foreach (var vector in vectors)
            {
                if (vector.Mode != mode)
                {
                    mode = vector.Mode;
                    if (mode.Length > 0) lines.Add($"{mode}:");
                }

                int destination = 0;
                for (int i = 0; i < vector.Width; i++) destination |= space.Read(vector.Address + i) << (8 * i);

                // A vector's own bank is the one it lives in, which for a 65816 is always zero.
                string where = labels?.Describe(destination) ?? $"${destination:X6}";
                string ran = coverage is { IsArmed: true } && coverage.WasExecuted(destination) ? "  (ran)" : string.Empty;
                lines.Add($"  {vector.Name,-12} ${vector.Address:X6} -> {where}{ran}");
            }

            return string.Join('\n', lines);
        }
    }
}
