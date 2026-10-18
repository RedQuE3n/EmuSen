using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Threading;
using EmuSen.Common.Imaging;
using EmuSen.Cores;
using EmuSen.Crystal.Recording;
using EmuSen.Crystal.Session;

namespace EmuSen.Crystal.Managed
{
    // The frame call and what it hands over: the frame-end order once, the picture into the caller's buffer, the sound drained - see EmuSen_Backend.md §3.4 and §14.
    internal sealed unsafe partial class ManagedSession
    {
        private FrameRecorder? _recorder;
        private long _loggedLines;

        public void Frame(FrameInfoV1* info)
        {
            if (info != null && info->size < sizeof(FrameInfoV1)) throw CrystalException.BadStruct("emusen_crystal_frame_info", info->size, sizeof(FrameInfoV1));
            int me = Environment.CurrentManagedThreadId;
            int other = Interlocked.CompareExchange(ref _inFrame, me, 0);
            if (other != 0 && other != me) throw new CrystalException(CrystalNumbers.Busy, $"another thread is inside a frame of session {Id}; a session has one frame thread");
            try
            {
                // A change waiting on another thread lands before this frame, not after it.
                var spin = new SpinWait();
                while (Volatile.Read(ref _pending) > 0) spin.SpinOnce();
                Enter();
                try
                {
                    Volatile.Write(ref _frameThread, me);
                    bool ran = (!_paused && !_halted) || _runRequested ? Advance() : false;
                    Fill(info, ran);
                }
                finally { Monitor.Exit(_lock); }
            }
            finally
            {
                if (other == 0) Volatile.Write(ref _inFrame, 0);
            }
        }

        // The frame-end order, written once from Mistress's, Hotaru's and Pharaoh's loops - see EmuSen_Backend.md §14.
        private bool Advance()
        {
            _runRequested = false;
            bool recording = _recorder is { IsRecording: true };
            Core.SkipRendering = !recording && (DrawNone || !Speed.ShouldRender(Core.TotalFrames));
            _halted = false;
            Core.RunFrame();
            if (Core.IsHaltedAtBreakpoint)
            {
                _halted = true;
                Bundle.DebugTarget.RefreshProviders();
                Runtime.Events.Raise(Id, CrystalNumbers.EventRun, "run", w =>
                {
                    w.WriteString("op", "halted");
                    w.WriteString("processor", HaltedName);
                    w.WriteNumber("address", unchecked((uint)Core.HaltedAddress));
                });
                return false;
            }
            Bundle.DebugTarget.RefreshProviders();
            if (Rewind.Enabled && EngineFeatures.Of(Core).RewindCapture && Rewind.OnFrameCompleted(Core) && Rewind.ThumbnailWidth > 0 && !Core.SkipRendering) AttachThumbnail();
            if (recording && !Core.SkipRendering) _recorder!.CaptureFrame(WritePicture);
            if (Engine is { } engine && engine.LoggedLines != _loggedLines) RaiseLog(engine);
            return true;
        }

        private void Fill(FrameInfoV1* info, bool ran)
        {
            if (info == null) return;
            double hz = Core.FrameRateHz;
            info->ran = ran ? 1u : 0u;
            info->halted = _halted ? 1u : 0u;
            info->rendered = ran && !Core.SkipRendering ? 1u : 0u;
            info->picture_serial = Core is IFrameSerial serial ? unchecked((ulong)serial.FrameSerial) : unchecked((ulong)Core.TotalFrames);
            info->width = Core.ScreenWidth;
            info->height = Core.ScreenHeight;
            info->row_repeat = Core is IRepeatedRows rows ? rows.RowRepeat : 1;
            info->display_lines = Core.DisplayLines;
            info->display_aspect = Core.DisplayAspect;
            info->audio_frames = (uint)Math.Max(0, Engine?.BufferedAudioFrames ?? 0);
            info->audio_rate = (uint)Math.Max(0, Core.AudioSampleRate);
            info->play_audio = Speed.ShouldPlayAudio ? 1u : 0u;
            info->speed_percent = (uint)Speed.SpeedPercent;
            info->frame_interval_ns = (ulong)(Speed.FrameInterval(hz).Ticks * 100);
            info->total_frames = (ulong)Math.Max(0, Core.TotalFrames);
            info->frame_rate_hz = hz;
            info->paused = _paused ? 1u : 0u;
            info->halted_processor = _halted ? HaltedIndex() : 0;
            info->halted_address = _halted ? unchecked((uint)Core.HaltedAddress) : 0;
        }

        private uint HaltedIndex()
        {
            if (Engine is not { IsRomLoaded: true } engine) return 0;
            int index = engine.Machine.Info.Processors.ToList().FindIndex(p => p.Id == engine.HaltedProcessor);
            return index < 0 ? 0 : (uint)index;
        }

        // The picture as RGBA8888, read whole: from another thread it waits for the frame in progress.
        public long CopyPicture(byte* o, nuint n)
        {
            lock (_lock)
            {
                if (_closed) throw new CrystalException(SessionAbi.NoSession, $"session {Id} was closed");
                var into = o == null ? Span<byte>.Empty : new Span<byte>(o, (int)Math.Min(n, (nuint)int.MaxValue));
                if (Engine is { } engine) return engine.CopyPicture(into);
                byte[] rgba = Core.GetFrameBufferRgba();
                if (into.Length >= rgba.Length) rgba.CopyTo(into);
                if (Core is IFrameBufferPool pool) pool.ReturnFrameBuffer(rgba);
                return rgba.Length;
            }
        }

        public long DrainAudio(short* o, nuint frames, uint* rate)
        {
            lock (_lock)
            {
                if (_closed) throw new CrystalException(SessionAbi.NoSession, $"session {Id} was closed");
                if (o == null)
                {
                    if (rate != null) *rate = (uint)Math.Max(0, Core.AudioSampleRate);
                    return Engine?.BufferedAudioFrames ?? 0;
                }
                int written;
                int r;
                if (Engine is { } engine) written = engine.DrainAudio(new Span<short>(o, (int)Math.Min(frames * 2, (nuint)int.MaxValue)), out r);
                else
                {
                    short[] samples = Core.DequeueAudioSamples((int)Math.Min(frames, (nuint)int.MaxValue));
                    samples.CopyTo(new Span<short>(o, samples.Length));
                    written = samples.Length / 2;
                    r = Core.AudioSampleRate;
                }
                if (rate != null) *rate = (uint)Math.Max(0, r);
                return written;
            }
        }

        public byte[] PhaseNames() => CrystalJson.Write(w =>
        {
            w.WriteStartArray();
            if (Core is IFrameProfiler profiler)
                foreach (var (name, _) in profiler.LastFramePhases) w.WriteStringValue(name);
            w.WriteEndArray();
        });

        public long Phases(double* o, nuint count)
        {
            if (Core is not IFrameProfiler profiler) return 0;
            var phases = profiler.LastFramePhases;
            if (o != null)
                for (int i = 0; i < phases.Count && (nuint)i < count; i++) o[i] = phases[i].Milliseconds;
            return phases.Count;
        }

        // The picture on show as one RGBA buffer with its rows repeated as the console shows them, for a state's or a recording's picture.
        private (byte[] Rgba, int Width, int Height) ShownPicture()
        {
            int width = Core.ScreenWidth, height = Core.ScreenHeight, repeat = Core is IRepeatedRows rows ? Math.Max(1, rows.RowRepeat) : 1;
            var picture = new byte[Math.Max(0, width * 4 * height)];
            fixed (byte* p = picture) CopyPicture(p, (nuint)picture.Length);
            if (repeat == 1) return (picture, width, height);
            var tall = new byte[width * 4 * height * repeat];
            int row = width * 4;
            for (int y = 0; y < height; y++)
                for (int r = 0; r < repeat; r++)
                    Buffer.BlockCopy(picture, y * row, tall, (y * repeat + r) * row, row);
            return (tall, width, height * repeat);
        }

        private void WritePicture(string path)
        {
            var (rgba, width, height) = ShownPicture();
            if (width > 0 && height > 0) PngFile.Write(path, rgba, width, height);
        }

        private void AttachThumbnail()
        {
            int width = Core.ScreenWidth, rows = Core.ScreenHeight, repeat = Core is IRepeatedRows r ? Math.Max(1, r.RowRepeat) : 1;
            var picture = new byte[Math.Max(0, width * 4 * rows)];
            fixed (byte* p = picture) CopyPicture(p, (nuint)picture.Length);
            Rewind.AttachThumbnail(picture, width, rows, repeat);
        }

        private void RaiseLog(Cores.Native.CoreEngine engine)
        {
            long total = engine.LoggedLines;
            int fresh = (int)Math.Min(total - _loggedLines, engine.RecentLog.Count);
            _loggedLines = total;
            if (!Runtime.Events.Wanted(Id, CrystalNumbers.EventLog)) return;
            string[] lines = engine.RecentLog.Skip(engine.RecentLog.Count - fresh).ToArray();
            Runtime.Events.Raise(Id, CrystalNumbers.EventLog, "log", w => CrystalJson.Strings(w, "lines", lines));
        }

        public string StartRecording(string baseDir, uint stride) => Change(() =>
        {
            if (_recorder is { IsRecording: true }) throw new CrystalException(SessionAbi.BadArgument, $"session {Id} is already recording to {_recorder.SessionDir}");
            _recorder = new FrameRecorder(Bundle.DebugTarget);
            return _recorder.Start(baseDir, (int)Math.Max(1, Math.Min(stride, int.MaxValue)));
        });

        public string StopRecording() => Change(() =>
        {
            if (_recorder is not { IsRecording: true } recorder) throw new CrystalException(SessionAbi.Absent, $"session {Id} is not recording");
            string dir = recorder.SessionDir!;
            StopRecorder();
            return dir;
        });

        private void StopRecorder()
        {
            _recorder?.Stop();
            _recorder = null;
        }
    }
}
