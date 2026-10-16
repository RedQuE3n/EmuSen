using System;
using System.Collections.Generic;

namespace EmuSen.Crystal.Debugger
{
    // A breakpoint's condition and logpoint read what `eval` reads at the registry's scope, refreshed at each hit - see `man bp`.
    public static class BreakpointExpressions
    {
        public static int Add(IDebugTarget target, BreakpointRegistry registry, int address, int end, string? condition, string? log)
        {
            Prepare(target, registry, condition, log);
            return registry.AddBreakpoint(address, end, condition, log);
        }

        public static int AddData(IDebugTarget target, BreakpointRegistry registry, string space, int address, int end, int value, bool onRead, bool changedOnly,
            string? condition, string? log)
        {
            Prepare(target, registry, condition, log);
            return registry.AddDataBreakpoint(space, address, end, value, onRead, changedOnly, condition, log);
        }

        // Both expressions checked before anything is added, then the registry's evaluators pointed at this target.
        private static void Prepare(IDebugTarget target, BreakpointRegistry registry, string? condition, string? log)
        {
            if (condition is null && log is null) return;
            var live = new Live(target, registry);
            IExpressionContext context = live.Scope();
            if (condition is not null) Check(context, condition, "Condition", condition);
            if (log is not null) foreach (string part in Parts(log)) Check(context, part, "Log expression", log);
            registry.ConditionEvaluator = live.Condition;
            registry.LogEvaluator = live.Render;
        }

        private static void Check(IExpressionContext context, string expression, string what, string whole)
        {
            try { new ExpressionEvaluator(context).Check(expression); }
            catch (ExpressionException e) { throw new ExpressionException($"{what} '{whole}' refused: {e.Message} Nothing was added."); }
        }

        // A logpoint's comma-separated parts, each rendered as expr=value.
        private static string[] Parts(string log) => log.Split(',', StringSplitOptions.RemoveEmptyEntries | StringSplitOptions.TrimEntries);

        // The registry's scope as `eval` reads it, its registers refreshed once an evaluation first names a symbol.
        private sealed class Live(IDebugTarget target, BreakpointRegistry registry) : IExpressionContext
        {
            private IExpressionContext _scope = null!;
            private bool _fresh;

            // The main processor's registry reads the target's context, as an unscoped `eval` does; another's reads its chip's.
            public IExpressionContext Scope()
            {
                if (ReferenceEquals(registry, target.Breakpoints)) return DebugExpressions.For(target);
                foreach (DebugCpu cpu in target.DebugCpus)
                    if (ReferenceEquals(cpu.Breakpoints, registry)) return DebugExpressions.For(target, cpu);
                return DebugExpressions.For(target);
            }

            private Live Begin()
            {
                _scope = Scope();
                _fresh = false;
                return this;
            }

            public (bool Result, string? Error) Condition(string condition)
                => new ExpressionEvaluator(Begin()).TryEvaluateBool(condition, out bool result, out string error) ? (result, null) : (false, error);

            public (string Text, string? Error) Render(string log)
            {
                var evaluator = new ExpressionEvaluator(Begin());
                var rendered = new List<string>();
                foreach (string part in Parts(log))
                {
                    try { rendered.Add($"{part}=0x{evaluator.Evaluate(part):X}"); }
                    catch (Exception e) { return (string.Empty, $"{part}: {e.Message}"); }
                }
                return (string.Join("  ", rendered), null);
            }

            public IReadOnlyList<string> SymbolNames => _scope.SymbolNames;

            public bool TryGetSymbol(string name, out long value)
            {
                if (!_fresh)
                {
                    _fresh = true;
                    target.RefreshProviders();
                }
                return _scope.TryGetSymbol(name, out value);
            }

            public bool TryReadMemory(string? space, int address, int width, out long value) => _scope.TryReadMemory(space, address, width, out value);
        }
    }
}
