using EmuSen.Cores;
using EmuSen.Common.Imaging;
using EmuSen.Cores.Nintendo.Venus;
using EmuSen.Cores.Nintendo.Venus.Controllers;
using EmuSen.Cores.Nintendo.Venus.Debug;
using EmuSen.Galaxia.Input;

namespace EmuSen.Pharaoh
{
    // The one shared frame-stepping primitive both the classic frame loop
    // and --commands mode drive - replaces two separately-drifting
    // implementations that used to do this same sequence slightly
    // differently. CurrentFrame is post-increment ("frames completed so
    // far"); a caller needing the classic loop's old pre-increment
    // numbering captures CurrentFrame before calling RunFrames(1). Full
    // design reasoning: EmuSen.DianaOS/DianaOS/Usr/Home/Documents/EmuSen Manual/EmuSen_Debugging_Tools_Reference_v5.md §3.15.
    public sealed class FrameRunner
    {
        private const int ProgressEvery = 600; // ~10s of real 60fps gameplay

        public ICore Core { get; }
        public long FrameCap { get; }
        public long CurrentFrame { get; private set; }

        public long CpuLogStart { get; set; } = -1;
        public long CpuLogEnd { get; set; } = -1;
        public bool Verbose { get; set; }

        // --cputrace: binary S-CPU trace from boot, written once this frame completes - see §3.40.
        public long CpuTraceEnd { get; set; } = -1;
        public string? CpuTracePath { get; set; }
        private bool _cpuTraceWritten;

        private readonly Action<string> emit;
        private readonly Action<long>? onFrameAdvanced;
        private readonly Dictionary<(PadButton Button, int Controller), bool> held = new();

        // Fed by RunFrames() below - see EmuSen_Rewind_And_FastForward.md §3.
        public EmuSen.Common.RewindBuffer Rewind { get; } = new();

        public FrameRunner(ICore core, long frameCap, Action<string> emit, Action<long>? onFrameAdvanced = null)
        {
            Core = core;
            FrameCap = frameCap;
            this.emit = emit;
            this.onFrameAdvanced = onFrameAdvanced;
        }

        private void ApplyHeld()
        {
            foreach (var kv in held) Core.SetButton(kv.Key.Controller - 1, kv.Key.Button, kv.Value);
        }

        // Sets a button's held state without advancing any frames - applied
        // immediately (not just queued for the next RunFrames call) so a
        // caller that checks Bus.Input right after this, with no frames
        // advance in between, sees the up-to-date state, matching the old
        // --commands hold/release verb's own explicit ApplyHeld() call.
        public void Hold(PadButton button, int controller = 1)
        {
            held[(button, controller)] = true;
            Core.SetButton(controller - 1, button, true);
        }

        public void Release(PadButton button, int controller = 1)
        {
            held[(button, controller)] = false;
            Core.SetButton(controller - 1, button, false);
        }

        public void Tap(PadButton button, int controller, long duration)
        {
            Hold(button, controller);
            RunFrames(duration);
            Release(button, controller);
        }

        // Armed before any instruction executes, where Mesen's starts too - see §3.40.
        private void EnsureCpuTraceArmed()
        {
            if (CpuTracePath == null || _cpuTraceArmed) return;
            _cpuTraceArmed = true;
            CpuBinaryTrace.Start();
        }

        private bool _cpuTraceArmed;

        private void WriteCpuTraceIfDue()
        {
            if (CpuTracePath == null || _cpuTraceWritten || CurrentFrame < CpuTraceEnd) return;
            _cpuTraceWritten = true;
            CpuBinaryTrace.Stop();
            CpuBinaryTrace.WriteTo(CpuTracePath);
            emit($"[CPUTRACE] {CpuBinaryTrace.Count} steps through frame {CurrentFrame} -> {CpuTracePath}");
            if (CpuBinaryTrace.Overflowed)
            {
                emit("[WARN] The trace buffer filled and recording stopped early - lower --cputrace's frame.");
            }
        }

        public void RunFrames(long count)
        {
            EnsureCpuTraceArmed();
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

                // A breakpoint returns from RunFrame() mid-frame, so this
                // frame is NOT complete - counting it, snapshotting it for
                // rewind or reporting it as advanced would all be wrong.
                // Stops the batch here and leaves the core halted, so the
                // next verb in the script inspects the moment it stopped at
                // - see EmuSen_Debugging_Tools_Reference_v5.md §3.23.
                if (Core.IsHaltedAtBreakpoint)
                {
                    HaltedAtBreakpoint = true;
                    // The providers `regs`/`sprites`/`pal` read are refreshed
                    // per completed frame, so without this they would report
                    // the PREVIOUS frame's end state at a mid-frame halt -
                    // exactly the moment a breakpoint exists to inspect.
                    OnHalted?.Invoke();
                    string processor = Core is ICoprocessorHalt halt ? halt.HaltedProcessorName : "CPU";
                    emit($"[BREAK] {processor} halted at ${Core.HaltedAddress:X6} (frame {CurrentFrame}).");
                    return;
                }

                CurrentFrame++;
                WriteCpuTraceIfDue();
                Rewind.OnFrameCompleted(Core);
                onFrameAdvanced?.Invoke(CurrentFrame);
                AfterFrame?.Invoke();
                if (CurrentFrame % ProgressEvery == 0) emit($"[frame {CurrentFrame}/{FrameCap}]");
            }
        }

        // True once RunFrames stopped early on a breakpoint - see §3.23.
        public bool HaltedAtBreakpoint { get; private set; }

        // Run when a breakpoint halts mid-frame - see the call site above.
        public Action? OnHalted { get; set; }

        // Ticked after every completed frame, alongside the constructor's own hook - see §3.15b.
        public Action? AfterFrame { get; set; }

        // Returns steps actually taken - see EmuSen_Rewind_And_FastForward.md §3.
        public int StepBack(int steps)
        {
            int done = 0;
            while (done < steps && Rewind.Rewind(Core)) done++;
            CurrentFrame = Math.Max(0, CurrentFrame - (long)done * Rewind.IntervalFrames);
            return done;
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
