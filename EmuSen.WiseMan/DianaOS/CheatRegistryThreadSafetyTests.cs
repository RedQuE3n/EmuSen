using System;
using System.Threading;
using EmuSen.DianaOS.DianaOS.Var;

namespace EmuSen.WiseMan.DianaOS
{
    // The registry is read by the emulation thread and written by whoever
    // clicked - see EmuSen_Settings_Reference.md §4.15.
    public class CheatRegistryThreadSafetyTests
    {
        private static void Fill(CheatRegistry registry, int count)
        {
            for (int i = 0; i < count; i++) registry.AddRamPoke("CpuBus", 0x7E0000 + i, 0x01, $"c{i}");
        }

        // The reported bug: loading a game's cheats while it ran threw out of
        // ApplyAll on the emulation thread, which Mistress reports as a CPU
        // halt - so the game froze and every later tick did nothing.
        [Fact]
        public void Loading_a_cheat_file_while_frames_apply_never_throws()
        {
            var registry = new CheatRegistry();
            Fill(registry, 50);

            Exception? caught = null;
            var stop = new CancellationTokenSource();

            // A real thread, not a pool task - this stands in for the
            // emulation thread and runs for the whole test.
            var emulation = new Thread(() =>
            {
                try
                {
                    while (!stop.IsCancellationRequested)
                    {
                        registry.ApplyAll((_, _) => 0, (_, _, _) => { });
                        registry.TryPatchRom(0x008000, 0x00, out _);
                        _ = registry.GetCheats().Count;
                    }
                }
                catch (Exception ex) { caught = ex; }
            });
            emulation.Start();

            // 435 is what the real A Link to the Past .cht holds.
            for (int round = 0; round < 100 && caught is null; round++)
            {
                registry.Clear();
                Fill(registry, 435);
                foreach (CheatInfo c in registry.GetCheats()) registry.SetEnabled(c.Id, true);
                registry.MasterEnabled = round % 2 == 0;
            }

            stop.Cancel();
            emulation.Join();

            Assert.Null(caught);
        }

        // A snapshot taken before a Clear stays walkable to its end rather
        // than emptying out underneath the caller.
        [Fact]
        public void A_frame_already_applying_finishes_against_the_list_it_started_with()
        {
            var registry = new CheatRegistry();
            Fill(registry, 10);

            int written = 0;
            registry.ApplyAll((_, _) => 0, (_, _, _) =>
            {
                // Mid-walk, standing in for the UI thread landing here.
                if (written == 0) registry.Clear();
                written++;
            });

            Assert.Equal(10, written);
        }
    }
}
