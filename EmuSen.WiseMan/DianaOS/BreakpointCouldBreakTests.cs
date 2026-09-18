using System;
using EmuSen.DianaOS.DianaOS.Var;

namespace EmuSen.WiseMan.DianaOS
{
    // CouldBreak is a promise about ShouldBreak, so every way to arm the registry has to make it true - see §3.26.
    public class BreakpointCouldBreakTests
    {
        [Fact]
        public void A_fresh_registry_cannot_break()
        {
            Assert.False(new BreakpointRegistry().CouldBreak);
        }

        [Fact]
        public void Every_way_to_arm_a_break_makes_it_possible()
        {
            Assert.True(Armed(r => r.AddBreakpoint(0x100)));
            Assert.True(Armed(r => r.ArmStep(3)));
            Assert.True(Armed(r => r.ArmStepToDepth(0)));
            Assert.True(Armed(r => r.ArmDepthGuard(2)));
            Assert.True(Armed(r => { r.AddDataBreakpoint("RAM", 0x10); r.NoteWrite("RAM", 0x10, 1); }));
            Assert.True(Armed(r => { r.ArmRunToFrame(5); r.NoteFrame(5); }));
            Assert.True(Armed(r => { r.ArmRunToScanline(9); r.NoteScanline(9); }));
            Assert.True(Armed(r => { r.ArmRunToInterrupt(CallFrameKind.Irq); r.NoteInterrupt(CallFrameKind.Irq); }));
            Assert.True(Armed(r => { r.ArmCondition("nmi", logOnly: false); r.NoteCondition("nmi", 0, "hit"); }));
        }

        [Fact]
        public void Arming_something_that_has_not_happened_yet_costs_nothing()
        {
            Assert.False(Armed(r => r.ArmRunToFrame(5)));
            Assert.False(Armed(r => r.AddDataBreakpoint("RAM", 0x10)));
            Assert.False(Armed(r => r.ArmCondition("nmi", logOnly: false)));
        }

        // Whenever it says no, the call it lets a core skip would have said no and changed nothing.
        [Fact]
        public void When_it_cannot_break_should_break_agrees_across_random_histories()
        {
            var noise = new Random(26);

            for (int history = 0; history < 400; history++)
            {
                var registry = new BreakpointRegistry { CallStack = new CallStackRegistry() };

                for (int step = 0; step < 30; step++)
                {
                    Operate(registry, noise.Next(12), noise.Next(4));

                    if (registry.CouldBreak) continue;

                    string reason = registry.LastBreakReason;
                    Assert.False(registry.ShouldBreak(noise.Next(4)));
                    Assert.Equal(reason, registry.LastBreakReason);
                    Assert.False(registry.CouldBreak);
                }
            }
        }

        [Fact]
        public void A_fresh_registry_is_quiet()
        {
            Assert.True(new BreakpointRegistry().IsQuiet);
        }

        // Quiet is the stronger promise: not only nothing pending, but nothing armed that a running frame could set off.
        [Fact]
        public void Arming_anything_at_all_ends_the_quiet()
        {
            Assert.False(Quiet(r => r.AddBreakpoint(0x100)));
            Assert.False(Quiet(r => r.ArmStep(3)));
            Assert.False(Quiet(r => r.ArmStepToDepth(0)));
            Assert.False(Quiet(r => r.ArmDepthGuard(2)));
            Assert.False(Quiet(r => r.AddDataBreakpoint("RAM", 0x10)));
            Assert.False(Quiet(r => r.ArmUninitializedReadBreak("RAM", 0x100)));
            Assert.False(Quiet(r => r.ArmRunToFrame(5)));
            Assert.False(Quiet(r => r.ArmRunToScanline(9)));
            Assert.False(Quiet(r => r.ArmRunToInterrupt(CallFrameKind.Irq)));
            Assert.False(Quiet(r => r.ArmCondition("nmi", logOnly: false)));
        }

        [Fact]
        public void Disarming_everything_brings_the_quiet_back()
        {
            Assert.True(Quiet(r =>
            {
                r.AddBreakpoint(0x100);
                r.AddDataBreakpoint("RAM", 0x10);
                r.ArmStep(3);
                r.ArmRunToFrame(5);
                r.ArmRunToScanline(9);
                r.ArmRunToInterrupt(CallFrameKind.Irq);
                r.ArmDepthGuard(2);
                r.ArmCondition("nmi", logOnly: false);
                r.ArmUninitializedReadBreak("RAM", 0x100);

                foreach (var bp in r.GetBreakpoints()) r.RemoveBreakpoint(bp.Id);
                foreach (var bp in r.GetDataBreakpoints()) r.RemoveBreakpoint(bp.Id);
                r.DisarmSteps();
                r.DisarmDepthGuard();
                r.DisarmCondition("nmi");
                r.DisarmUninitializedReadBreak();
            }));
        }

        // Whenever it says quiet, everything a running frame can do to the registry never halts it and never ends the quiet.
        [Fact]
        public void Nothing_a_frame_does_can_set_off_a_quiet_registry()
        {
            var noise = new Random(27);
            int quietChecks = 0;

            for (int history = 0; history < 400; history++)
            {
                var registry = new BreakpointRegistry { CallStack = new CallStackRegistry() };

                for (int step = 0; step < 30; step++)
                {
                    Command(registry, noise.Next(24), noise.Next(4));
                    if (!registry.IsQuiet) continue;

                    quietChecks++;
                    for (int happening = 0; happening < 8; happening++)
                    {
                        int value = noise.Next(4);
                        Happen(registry, noise.Next(8), value, noise);
                        Assert.False(registry.ShouldBreak(value));
                        Assert.True(registry.IsQuiet);
                    }
                }
            }

            Assert.True(quietChecks > 1000);
        }

        // What a command does: arms or disarms one kind of halt, or clears the lot.
        private static void Command(BreakpointRegistry r, int operation, int value)
        {
            switch (operation)
            {
                case 0: r.AddBreakpoint(value); break;
                case 1: r.AddDataBreakpoint("RAM", value); break;
                case 2: r.ArmStep(value + 1); break;
                case 3: r.ArmStepToDepth(value); break;
                case 4: r.ArmDepthGuard(value); break;
                case 5: r.ArmRunToFrame(value); break;
                case 6: r.ArmRunToScanline(value); break;
                case 7: r.ArmRunToInterrupt((CallFrameKind)(value % 5)); break;
                case 8: r.ArmCondition("brk", logOnly: value == 0); break;
                case 9: r.ArmUninitializedReadBreak("RAM", 4); break;
                case 10: foreach (var bp in r.GetBreakpoints()) r.RemoveBreakpoint(bp.Id); break;
                case 11: foreach (var bp in r.GetDataBreakpoints()) r.RemoveBreakpoint(bp.Id); break;
                case 12: r.DisarmDepthGuard(); break;
                case 13: r.DisarmCondition("brk"); break;
                case 14: r.DisarmUninitializedReadBreak(); break;
                case 15: r.ShouldBreak(value); break;
                case 16: r.DisarmSteps(); break;
                default:
                    foreach (var bp in r.GetBreakpoints()) r.RemoveBreakpoint(bp.Id);
                    foreach (var bp in r.GetDataBreakpoints()) r.RemoveBreakpoint(bp.Id);
                    r.DisarmSteps();
                    r.DisarmDepthGuard();
                    r.DisarmCondition("brk");
                    r.DisarmUninitializedReadBreak();
                    break;
            }
        }

        // What a running frame does: writes, reads, frames, lines, interrupts, conditions, calls and returns.
        private static void Happen(BreakpointRegistry r, int operation, int value, Random noise)
        {
            switch (operation)
            {
                case 0: r.NoteWrite("RAM", value, (byte)value); break;
                case 1: r.NoteRead("RAM", value, (byte)value); break;
                case 2: r.NoteFrame(value); break;
                case 3: r.NoteScanline(value); break;
                case 4: r.NoteInterrupt((CallFrameKind)noise.Next(5)); break;
                case 5: r.NoteCondition("brk", value, "hit"); break;
                case 6: r.CallStack!.NotePush(0, value, CallFrameKind.Call); break;
                default: r.CallStack!.NotePop(); break;
            }
        }

        // WatchesWrites is the same kind of promise about NoteWrite: false only when it would do nothing.
        [Fact]
        public void Writes_are_watched_exactly_while_a_data_breakpoint_or_the_uninitialised_read_check_is_armed()
        {
            var registry = new BreakpointRegistry();
            Assert.False(registry.WatchesWrites);

            int id = registry.AddDataBreakpoint("RAM", 0x10);
            Assert.True(registry.WatchesWrites);
            registry.RemoveBreakpoint(id);
            Assert.False(registry.WatchesWrites);

            registry.ArmUninitializedReadBreak("RAM", 0x100);
            Assert.True(registry.WatchesWrites);
            registry.DisarmUninitializedReadBreak();
            Assert.False(registry.WatchesWrites);

            // A code breakpoint or a step never reads a write, so neither ends it.
            registry.AddBreakpoint(0x100);
            registry.ArmStep(1);
            Assert.False(registry.WatchesWrites);
        }

        [Fact]
        public void A_watch_registry_has_watches_exactly_while_one_exists()
        {
            var watches = new WatchRegistry();
            Assert.False(watches.HasWatches);

            int id = watches.AddWatch("RAM", 0x10, 4);
            Assert.True(watches.HasWatches);

            watches.RemoveWatch(id);
            Assert.False(watches.HasWatches);
        }

        private static bool Quiet(Action<BreakpointRegistry> arm)
        {
            var registry = new BreakpointRegistry { CallStack = new CallStackRegistry() };
            arm(registry);
            return registry.IsQuiet;
        }

        private static bool Armed(Action<BreakpointRegistry> arm)
        {
            var registry = new BreakpointRegistry { CallStack = new CallStackRegistry() };
            arm(registry);
            return registry.CouldBreak;
        }

        private static void Operate(BreakpointRegistry r, int operation, int value)
        {
            switch (operation)
            {
                case 0: r.AddBreakpoint(value); break;
                case 1: foreach (var bp in r.GetBreakpoints()) r.RemoveBreakpoint(bp.Id); break;
                case 2: r.ArmStep(value + 1); break;
                case 3: r.DisarmSteps(); break;
                case 4: r.AddDataBreakpoint("RAM", value); break;
                case 5: r.NoteWrite("RAM", value, (byte)value); break;
                case 6: r.ArmRunToFrame(value); break;
                case 7: r.NoteFrame(value); break;
                case 8: r.ArmDepthGuard(value); break;
                case 9: r.DisarmDepthGuard(); break;
                case 10: r.ShouldBreak(value); break;
                default: r.CallStack!.NotePush(0, value, CallFrameKind.Call); break;
            }
        }
    }
}
