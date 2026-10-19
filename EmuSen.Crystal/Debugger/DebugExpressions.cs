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

        // What every context reads its symbols from, each processor's registers and the video and sound rows, refreshed without the views - see `man bp`.
        public static void RefreshRegisters(IDebugTarget target)
        {
            var main = target.CpuRegisters;
            main.Refresh();
            target.VideoRegisters.Refresh();
            target.ApuRegisters.Refresh();
            var cpus = target.DebugCpus;
            for (int i = 0; i < cpus.Count; i++)
                if (cpus[i].Registers is { } registers && !ReferenceEquals(registers, main)) registers.Refresh();
        }
    }
}
