using System.Text.Json;
using EmuSen.Crystal.Client;
using EmuSen.Crystal.Client.Contract;
using EmuSen.Galaxia.Input;
using EmuSen.Pharaoh.Imaging;

namespace EmuSen.Pharaoh
{
    // The one frame-stepping primitive the classic loop and --commands drive, over a session of the frontend API; CurrentFrame counts frames completed - see EmuSen_Debugging_Tools_Reference_v5.md §3.15 and EmuSen_Backend.md §14.8.
    public sealed class FrameRunner
    {
        private const int ProgressEvery = 600;

        public CrystalSession Session { get; }
        public long FrameCap { get; }
        public long CurrentFrame { get; private set; }

        public bool Verbose { get; set; }

        private readonly Action<string> emit;
        private readonly Action<long>? onFrameAdvanced;
        private readonly Dictionary<(PadButton Button, int Controller), bool> held = new();
        private CrystalFrameInfo _info;
        private byte[] _picture = Array.Empty<byte>();
        private short[] _sound = Array.Empty<short>();

        public FrameRunner(CrystalSession session, long frameCap, Action<string> emit, Action<long>? onFrameAdvanced = null)
        {
            Session = session;
            FrameCap = frameCap;
            this.emit = emit;
            this.onFrameAdvanced = onFrameAdvanced;
        }

        // The last frame's facts as the frame call gave them.
        public ref readonly CrystalFrameInfo Last => ref _info;

        // The snapshot interval `rewind on` set, which a step back counts frames by.
        public int RewindInterval { get; set; } = 4;

        private void ApplyHeld()
        {
            foreach (var kv in held) Session.Button(kv.Key.Controller - 1, kv.Key.Button, kv.Value);
        }

        public void Hold(PadButton button, int controller = 1)
        {
            held[(button, controller)] = true;
            Session.Button(controller - 1, button, true);
        }

        public void Release(PadButton button, int controller = 1)
        {
            held[(button, controller)] = false;
            Session.Button(controller - 1, button, false);
        }

        public void Tap(PadButton button, int controller, long duration)
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
                    emit($"[WARN] Hit the {FrameCap}-frame safety cap - ignoring the rest of this request.");
                    return;
                }
                // The next frame after a halt resumes it, as a script's next verb always has - see §3.23.
                if (_info.halted != 0) Session.Run(CrystalSessionAbi.RunResume);
                ApplyHeld();
                Session.Frame(ref _info);
                if (_info.halted != 0)
                {
                    HaltedAtBreakpoint = true;
                    string processor = JsonDocument.Parse(Session.Info()).RootElement.GetProperty("halted").GetProperty("processor").GetString() ?? "CPU";
                    emit($"[BREAK] {processor} halted at ${unchecked((int)(uint)_info.halted_address):X6} (frame {CurrentFrame}).");
                    return;
                }

                CurrentFrame++;
                onFrameAdvanced?.Invoke(CurrentFrame);
                if (CurrentFrame % ProgressEvery == 0) emit($"[frame {CurrentFrame}/{FrameCap}]");
            }
        }

        // True once RunFrames stopped early on a breakpoint - see §3.23.
        public bool HaltedAtBreakpoint { get; private set; }

        // The picture on show, in a buffer kept between calls and valid until the next; its width and height.
        public (byte[] Rgba, int Width, int Height) Picture()
        {
            JsonElement shape = JsonDocument.Parse(Session.Info()).RootElement.GetProperty("picture");
            int width = shape.GetProperty("width").GetInt32(), height = shape.GetProperty("height").GetInt32();
            long bytes = Session.PictureBytes;
            if (_picture.Length != bytes) _picture = new byte[bytes];
            Session.CopyPicture(_picture);
            return (_picture, width, height);
        }

        // Every sound sample waiting, drained into a buffer kept between calls; the span is valid until the next.
        public ReadOnlySpan<short> DrainSound()
        {
            int frames = Session.AudioFramesReady;
            if (_sound.Length < frames * 2) _sound = new short[frames * 2];
            int got = Session.DrainAudio(_sound, out _);
            return _sound.AsSpan(0, got * 2);
        }

        // Returns steps actually taken - see EmuSen_Rewind_And_FastForward.md §3.
        public int StepBack(int steps)
        {
            var info = new CrystalFrameInfo();
            int done = (int)Session.RewindStep((uint)Math.Max(0, steps), ref info);
            CurrentFrame = Math.Max(0, CurrentFrame - (long)done * RewindInterval);
            return done;
        }

        // Frames until the picture's hash has held for quietFrames after a first change, or maxFrames - see §3.15's waitstable.
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
                ulong hash = FrameHash.Compute(Picture().Rgba);
                if (lastHash.HasValue && hash != lastHash.Value) sawChange = true;
                if (hash == lastHash) quietCount++;
                else { quietCount = 0; lastHash = hash; }
                if (sawChange && quietCount >= quietFrames) break;
            }
            return stepped;
        }
    }
}
