using EmuSen.Common.Imaging;
using EmuSen.Cores.Nintendo.Venus;
using EmuSen.Cores.Nintendo.Venus.Controllers;

namespace EmuSen.Pharaoh90
{
    // The one shared frame-stepping primitive both the classic frame loop
    // and --commands mode drive - replaces two implementations
    // (Program.cs's old inline for-loop, and --commands mode's own
    // closure-based RunFrames/ApplyHeld) that did the same "apply held
    // input, apply cpuLog windowing, RunFrame(), autoshot check, progress
    // emit" sequence slightly differently, with real risk of drifting
    // further apart every time one got a bugfix the other didn't.
    //
    // CurrentFrame follows a post-increment "frames completed so far"
    // convention (matching --commands mode's pre-existing, already-more-
    // correct semantic) - callers that need the classic loop's old
    // pre-increment "frame about to run" numbering (tap/screenshot
    // frame-indexing, autoshot filenames) capture CurrentFrame themselves
    // before calling RunFrames(1), same as this class's own cpuLog window
    // check does internally.
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

        // Advances one frame at a time (reusing RunFrames(1) so held
        // input/cpuLog windowing/autoshot all still apply exactly as they
        // would for a plain `frames` line) until the framebuffer hash stops
        // changing for <quietFrames> in a row, or <maxFrames> is reached -
        // whichever comes first.
        //
        // Requires seeing at least one real change before a quiet streak
        // counts as "settled" - a real bug caught testing this against the
        // actual SMAS investigation: called right after a `tap Start` while
        // still sitting on the static Nintendo boot logo, the naive "N
        // identical frames in a row" version reported stable after only
        // ~16 frames, because the logo itself doesn't animate and was
        // already "stable" the instant it was checked - long before the
        // tap's own transition had even started, let alone finished. It
        // can't tell "hasn't reacted yet" apart from "finished reacting"
        // without this.
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
