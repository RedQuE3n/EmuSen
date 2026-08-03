using System;
using System.Linq;
using EmuSen.DianaOS.DianaOS.Lib;
using EmuSen.DianaOS.DianaOS.Var;
using static EmuSen.DianaOS.DianaOS.Bin.Commands.EmuSen.DebugCommandHelpers;

namespace EmuSen.DianaOS.DianaOS.Bin.Commands.EmuSen
{
    // Evaluates one expression against live core state - see `man eval`.
    public class EvalCommand : IDianaOSCommand
    {
        public string Name => "eval";
        public bool IsReadOnly => true;
        public string Usage => string.Join('\n', new[]
        {
            "  eval <expr>                   evaluate an expression against live state (decimal, hex, binary)",
            "  eval symbols [<filter>]       list every symbol an expression can name",
            "  eval <cpu> <expr>             evaluate against another processor's registers - `cpus` lists them",
        });

        public DianaOSResult Execute(IDebugTarget? target, string[] parts, string? stdin)
        {
            IDebugTarget resolved;
            try { resolved = RequireTarget(target); }
            catch (Exception ex) { return DianaOSResult.Fail(ex.Message); }

            if (parts.Length < 2) return "Usage: eval <expr> | eval symbols [<filter>]";

            // A lone chip name is an expression, not a scope - see `man eval`.
            int at = 1;
            DebugCpu? cpu = null;
            if (parts.Length > 2 && DebugCpus.Find(resolved.DebugCpus, parts[1]) is { } named)
            {
                cpu = named;
                at = 2;
            }

            var context = ExpressionsFor(resolved, cpu);

            if (parts[at].Equals("symbols", StringComparison.OrdinalIgnoreCase))
            {
                string filter = parts.Length > at + 1 ? parts[at + 1] : string.Empty;
                var names = context.SymbolNames
                    .Where(n => filter.Length == 0 || n.Contains(filter, StringComparison.OrdinalIgnoreCase))
                    .ToList();
                if (names.Count == 0) return filter.Length == 0 ? "No symbols available." : $"No symbol matches '{filter}'.";
                return string.Join('\n', names.Select(n => $"  {n}"));
            }

            // Rejoined: the shell splits an unquoted `a == 5` - see `man eval`.
            string expression = string.Join(' ', parts.Skip(at));

            try
            {
                long value = new ExpressionEvaluator(context).Evaluate(expression);
                return ExpressionEvaluator.Format(value);
            }
            catch (ExpressionException ex) { return DianaOSResult.Fail(ex.Message); }
            catch (Exception ex) { return DianaOSResult.Fail($"Cannot evaluate '{expression}': {ex.Message}"); }
        }
    }
}
