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
