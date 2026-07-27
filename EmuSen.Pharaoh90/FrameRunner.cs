using EmuSen.Common.Imaging;
using EmuSen.Cores.Nintendo.Venus;
using EmuSen.Cores.Nintendo.Venus.Controllers;

namespace EmuSen.Pharaoh90
{
    // The one shared frame-stepping primitive both the classic frame loop
    // and --commands mode drive - replaces two separately-drifting
    // implementations that used to do this same sequence slightly
    // differently. CurrentFrame is post-increment ("frames completed so
    // far"); a caller needing the classic loop's old pre-increment
    // numbering captures CurrentFrame before calling RunFrames(1). Full
    // design reasoning: Man pages/EmuSen_Debugging_Tools_Reference_v5.md §3.15.
    public sealed class FrameRunner
    {
        private const int ProgressEvery = 600; // ~10s of real 60fps gameplay

        public VenusCore Core { get; }
        public long FrameCap { get; }
        public long CurrentFrame { get; private set; }

        public long CpuLogStart { get; set; } = -1;
        public long CpuLogEnd { get; set; } = -1;
        public bool Verbose { get; set; }

        private readonly Action<string> emit;
        private readonly Action<long>? onFrameAdvanced;
        private readonly Dictionary<(SnesButton Button, int Controller), bool> held = new();

        public FrameRunner(VenusCore core, long frameCap, Action<string> emit, Action<long>? onFrameAdvanced = null)
        {
            Core = core;
            FrameCap = frameCap;
            this.emit = emit;
            this.onFrameAdvanced = onFrameAdvanced;
        }

        private void ApplyHeld()
        {
            foreach (var kv in held) Core.Bus!.Input.SetButton(kv.Key.Button, kv.Value, kv.Key.Controller);
        }

        // Sets a button's held state without advancing any frames - applied
        // immediately (not just queued for the next RunFrames call) so a
        // caller that checks Bus.Input right after this, with no frames
        // advance in between, sees the up-to-date state, matching the old
        // --commands hold/release verb's own explicit ApplyHeld() call.
        public void Hold(SnesButton button, int controller = 1)
        {
            held[(button, controller)] = true;
            Core.Bus!.Input.SetButton(button, true, controller);
        }

        public void Release(SnesButton button, int controller = 1)
        {
            held[(button, controller)] = false;
            Core.Bus!.Input.SetButton(button, false, controller);
        }

        public void Tap(SnesButton button, int controller, long duration)
        {
            Hold(button, controller);
            RunFrames(duration);
            Release(button, controller);
        }

        public void RunFrames(long count)
        {
            for (long i = 0; i < count; i++)
            {
                if (CurrentFrame >= FrameCap)
                {
                    // Generalized wording - not just the `frames` verb hits
                    // this: waitstable/contactsheet's own inner RunFrames(1)
                    // calls can too.
                    emit($"[WARN] Hit the {FrameCap}-frame safety cap - ignoring the rest of this request.");
                    return;
                }
                ApplyHeld();
                if (CpuLogStart >= 0)
                {
                    // CpuVerboseLogging's own getter is gated by
                    // MasterLoggingEnabled (same pattern as DmaVerboseLogging),
                    // so both need setting for this window to actually
                    // produce output.
                    bool inWindow = CurrentFrame >= CpuLogStart && CurrentFrame < CpuLogEnd;
                    EmuSen.Debug.DebugSettings.MasterLoggingEnabled = inWindow || Verbose;
                    EmuSen.Debug.DebugSettings.CpuVerboseLogging = inWindow;
                }
                Core.RunFrame();
                CurrentFrame++;
                onFrameAdvanced?.Invoke(CurrentFrame);
                if (CurrentFrame % ProgressEvery == 0) emit($"[frame {CurrentFrame}/{FrameCap}]");
            }
        }

        // Advances one frame at a time (via RunFrames(1), so held input/
        // cpuLog windowing/autoshot all still apply) until the framebuffer
        // hash stops changing for <quietFrames> in a row, or <maxFrames> is
        // reached. Requires seeing a real change first, not just N identical
        // frames - a real bug this once shipped with, see §3.15's own
        // waitstable note for the failure case that caught it.
        public long WaitStable(long maxFrames, long quietFrames)
        {
            ulong? lastHash = null;
            long quietCount = 0;
            long stepped = 0;
            bool sawChange = false;
            while (stepped < maxFrames && CurrentFrame < FrameCap)
            {
                RunFrames(1);
                stepped++;
                ulong hash = FrameHash.Compute(Core.GetFrameBufferRgba());
                if (lastHash.HasValue && hash != lastHash.Value) sawChange = true;
                if (hash == lastHash) quietCount++;
                else { quietCount = 0; lastHash = hash; }
                if (sawChange && quietCount >= quietFrames) break;
            }
            return stepped;
        }
    }
}
