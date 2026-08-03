using System;
using System.Collections.Generic;
using System.Linq;
using EmuSen.DianaOS.DianaOS.Var;

namespace EmuSen.DianaOS.DianaOS.Lib
{
    // The zero-per-core context: symbols from a target's own registers - see `man eval`.
    public sealed class DebugTargetExpressionContext : IExpressionContext
    {
        private readonly IDebugTarget _target;

        public DebugTargetExpressionContext(IDebugTarget target) => _target = target;

        private IEnumerable<DebugRegisterValue> Registers()
            => _target.CpuRegisters.Current
                .Concat(_target.VideoRegisters.Current)
                .Concat(_target.ApuRegisters.Current)
                .Concat(_target.CoprocessorRegisters.Current);

        public IReadOnlyList<string> SymbolNames
            => Registers().Select(r => r.Name)
                .Concat(new[] { "frame" })
                .Concat(_target.Labels?.All().Select(l => l.Name) ?? Enumerable.Empty<string>())
                .Distinct(StringComparer.OrdinalIgnoreCase)
                .OrderBy(n => n, StringComparer.OrdinalIgnoreCase)
                .ToList();

        public bool TryGetSymbol(string name, out long value)
        {
            if (string.Equals(name, "frame", StringComparison.OrdinalIgnoreCase))
            {
                value = _target.FrameCount;
                return true;
            }

            foreach (var register in Registers())
            {
                if (!string.Equals(register.Name, name, StringComparison.OrdinalIgnoreCase)) continue;
                value = (long)register.Value;
                return true;
            }

            if (_target.Labels is { } labels && labels.TryGetAddress(name, out int address))
            {
                value = address;
                return true;
            }

            value = 0;
            return false;
        }

        public bool TryReadMemory(string? space, int address, int width, out long value)
        {
            value = 0;
            var spaces = _target.GetMemorySpaces();
            if (spaces.Count == 0 || address < 0) return false;

            // No name means the target's first space, its main CPU bus.
            var resolved = space == null
                ? spaces[0]
                : spaces.FirstOrDefault(s => string.Equals(s.Name, space, StringComparison.OrdinalIgnoreCase));
            if (resolved == null) return false;

            for (int i = 0; i < width; i++)
            {
                int at = address + i;
                if (at < 0 || at >= resolved.Size) return false;
                value |= (long)resolved.Read(at) << (8 * i);
            }
            return true;
        }
    }
}
