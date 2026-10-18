using System;
using System.Collections.Generic;
using System.Linq;
using EmuSen.Crystal.Debugger;

namespace EmuSen.Cores.Native
{
    // A scope's own symbols first, then the machine's unit registers read live through debug_registers - see `man eval` and EmuSen_Backend.md §24.
    internal sealed class UnitExpressionContext : IExpressionContext
    {
        private readonly CoreEngine _engine;
        private readonly IExpressionContext _scope;
        private readonly IReadOnlyList<string> _scopeNames;
        private UnitSymbols _symbols = UnitSymbols.None;

        // <scopeNames> are what <scope> answers besides labels, which no unit spelling may take.
        public UnitExpressionContext(CoreEngine engine, IExpressionContext scope, IReadOnlyList<string> scopeNames) =>
            (_engine, _scope, _scopeNames) = (engine, scope, scopeNames);

        private UnitSymbols Symbols()
        {
            if (!_engine.IsRomLoaded) return UnitSymbols.None;
            CoreMachineInfo info = _engine.Machine.Info;
            UnitSymbols symbols = _symbols;
            return ReferenceEquals(symbols.Info, info) ? symbols : _symbols = new UnitSymbols(info, _scopeNames);
        }

        public IReadOnlyList<string> SymbolNames
        {
            get
            {
                IReadOnlyList<string> own = _scope.SymbolNames;
                IReadOnlyList<string> units = Symbols().Names;
                if (units.Count == 0) return own;
                return own.Concat(units.Where(n => !_engine.Labels.TryGetAddress(n, out _))).Distinct(StringComparer.OrdinalIgnoreCase).OrderBy(n => n, StringComparer.OrdinalIgnoreCase).ToList();
            }
        }

        // A unit's spelling is looked up first so that it never pays for the scope's miss; any other name takes the scope's path unchanged.
        public bool TryGetSymbol(string name, out long value)
        {
            if (!Symbols().TryFind(name, out uint unit, out int index, out int count) || _engine.Labels.TryGetAddress(name, out _))
                return _scope.TryGetSymbol(name, out value);
            value = 0;
            Span<long> values = count <= 256 ? stackalloc long[count] : new long[count];
            int n = _engine.Machine.Registers(unit, values);
            if (index >= n || n > values.Length) return false;
            value = values[index];
            return true;
        }

        public bool TryReadMemory(string? space, int address, int width, out long value) => _scope.TryReadMemory(space, address, width, out value);
    }

    // One machine info's unit registers by spelling: "<unit>.<register>" for each, and the register alone where only one unit has it; a spelling the scope takes is left out.
    internal sealed class UnitSymbols
    {
        public static readonly UnitSymbols None = new(null);

        private readonly Dictionary<string, (uint Unit, int Index, int Count)> _byName = new(StringComparer.OrdinalIgnoreCase);

        public CoreMachineInfo? Info { get; }

        // The qualified spellings, which `eval symbols` lists.
        public IReadOnlyList<string> Names { get; }

        public UnitSymbols(CoreMachineInfo? info, IReadOnlyList<string>? taken = null)
        {
            Info = info;
            var scope = new HashSet<string>(taken ?? Array.Empty<string>(), StringComparer.OrdinalIgnoreCase);
            var qualified = new Dictionary<string, (uint, int, int)?>(StringComparer.OrdinalIgnoreCase);
            var bare = new Dictionary<string, (uint, int, int)?>(StringComparer.OrdinalIgnoreCase);
            IReadOnlyList<CoreUnit> units = info?.Units ?? Array.Empty<CoreUnit>();
            foreach (CoreUnit unit in units)
            {
                string prefix = new string(unit.Name.Where(c => char.IsLetterOrDigit(c) || c == '_').ToArray()).ToLowerInvariant();
                for (int i = 0; i < unit.Registers.Count; i++)
                {
                    string register = unit.Registers[i].Name;
                    if (prefix.Length > 0) Note(qualified, prefix + "." + register, (unit.Id, i, unit.Registers.Count));
                    Note(bare, register, (unit.Id, i, unit.Registers.Count));
                }
            }
            var names = new List<string>();
            foreach (var (name, at) in qualified)
            {
                if (at is not { } found || scope.Contains(name)) continue;
                _byName[name] = found;
                names.Add(name);
            }
            foreach (var (name, at) in bare)
                if (at is { } found && !scope.Contains(name)) _byName.TryAdd(name, found);
            Names = names;
        }

        // A spelling the expression language cannot read is left out; one two registers share stands for neither.
        private static void Note(Dictionary<string, (uint, int, int)?> into, string spelling, (uint, int, int) at)
        {
            if (!IsIdentifier(spelling)) return;
            into[spelling] = into.ContainsKey(spelling) ? null : at;
        }

        // ExpressionEvaluator's identifier: a letter, '_' or '.', then letters, digits, '_' and '.'.
        private static bool IsIdentifier(string s) =>
            s.Length > 0 && (char.IsLetter(s[0]) || s[0] is '_' or '.') && s.All(c => char.IsLetterOrDigit(c) || c is '_' or '.');

        public bool TryFind(string name, out uint unit, out int index, out int count)
        {
            bool found = _byName.TryGetValue(name, out var at);
            (unit, index, count) = at;
            return found;
        }
    }
}
