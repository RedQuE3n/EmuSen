using System;
using System.Collections.Generic;
using System.Linq;
using EmuSen.Cauldron;
using EmuSen.DianaOS.DianaOS.Var;

namespace EmuSen.DianaOS.DianaOS.Lib
{
    // The zero-per-chip context: symbols from whatever registers a DebugCpu already reports - see `man eval`.
    public sealed class DebugCpuExpressionContext : IExpressionContext
    {
        private readonly IDebugTarget _target;
        private readonly Func<IReadOnlyList<DebugRegisterValue>> _registers;
        private readonly string? _defaultSpace;

        public DebugCpuExpressionContext(
            IDebugTarget target,
            Func<IReadOnlyList<DebugRegisterValue>> registers,
            string? defaultSpace = null)
        {
            _target = target;
            _registers = registers;
            _defaultSpace = defaultSpace;
        }

        public static DebugCpuExpressionContext For(IDebugTarget target, DebugCpu cpu)
            => new(target,
                   () => cpu.Registers?.Current ?? Array.Empty<DebugRegisterValue>(),
                   cpu.CodeSpace);

        public IReadOnlyList<string> SymbolNames
            => _registers().Select(r => r.Name)
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

            foreach (var register in _registers())
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

        // An unnamed read goes to this chip's own space, not the S-CPU's - see `man eval`.
        public bool TryReadMemory(string? space, int address, int width, out long value)
        {
            value = 0;
            var spaces = _target.GetMemorySpaces();
            if (spaces.Count == 0 || address < 0) return false;

            string? wanted = space ?? _defaultSpace;
            var resolved = wanted == null
                ? spaces[0]
                : spaces.FirstOrDefault(s => string.Equals(s.Name, wanted, StringComparison.OrdinalIgnoreCase));
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
