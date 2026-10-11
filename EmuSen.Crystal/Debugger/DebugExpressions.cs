namespace EmuSen.Crystal.Debugger
{
    // Which context an expression is evaluated in, for `eval`, conditional breakpoints and the session service alike - see `man eval`.
    public static class DebugExpressions
    {
        // The target's own context when it publishes one.
        public static IExpressionContext For(IDebugTarget target)
            => target.Expressions ?? new DebugTargetExpressionContext(target);

        // A chip's own context, else one derived from its reported registers.
        public static IExpressionContext For(IDebugTarget target, DebugCpu? cpu)
            => cpu == null ? For(target) : cpu.Expressions ?? DebugCpuExpressionContext.For(target, cpu);
    }
}
