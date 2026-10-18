using System;
using System.IO;
using System.Linq;
using EmuSen.Common;
using EmuSen.Cores;
using EmuSen.Crystal.Session;
using EmuSen.Galaxia.Library;
using EmuSen.Galaxia.Models;

namespace EmuSen.Crystal.Managed
{
    // States and rewind, with the one sequence after a load: the history cleared, the debugger's snapshots refreshed, an event raised - see EmuSen_Backend.md §3.12's C7 and §14.
    internal sealed unsafe partial class ManagedSession
    {
        public const int Slots = 8;

        // The player's state folder, read at each use, so every program names a state alike - see EmuSen_Backend.md §4.3's F4.
        private static string? StateFolder => AppSettings.Load().StateDirectory;

        public string StatePath(int slot) => SaveLibrary.StatePathFor(GameKey, slot, StateFolder);

        public string ResumePath => SaveLibrary.ResumeStatePathFor(GameKey, StateFolder);

        public void SaveState(string path) => Change(() =>
        {
            if (Path.GetDirectoryName(Path.GetFullPath(path)) is { } folder) Directory.CreateDirectory(folder);
            Core.SaveState(path);
            Runtime.Events.Raise(Id, CrystalNumbers.EventState, "state", w => { w.WriteString("op", "saved"); w.WriteString("path", path); });
        });

        public void LoadState(string path) => Change(() =>
        {
            if (!File.Exists(path)) throw new CrystalException(SessionAbi.Absent, $"no state at {path}");
            Core.LoadState(path);
            AfterJump();
            Runtime.Events.Raise(Id, CrystalNumbers.EventState, "state", w => { w.WriteString("op", "loaded"); w.WriteString("path", path); });
        });

        // After the machine jumped: no history leads to it, no halt holds it, and the debugger reads where it is now.
        private void AfterJump()
        {
            Rewind.Clear();
            _halted = false;
            Bundle.DebugTarget.RefreshProviders();
        }

        private void SaveWithPicture(string path) => Change(() =>
        {
            SaveState(path);
            WritePicture(SaveLibrary.PicturePathFor(path));
        });

        public void SaveSlot(uint slot) => SaveWithPicture(StatePath(CheckSlot(slot)));

        public void LoadSlot(uint slot) => LoadState(StatePath(CheckSlot(slot)));

        public void SaveResume() => SaveWithPicture(ResumePath);

        public void LoadResume() => LoadState(ResumePath);

        private static int CheckSlot(uint slot) =>
            slot is >= 1 and <= Slots ? (int)slot : throw new CrystalException(SessionAbi.BadArgument, $"no state slot {slot}; the slots are 1 to {Slots}");

        public byte[] SlotsJson() => CrystalJson.Write(w =>
        {
            w.WriteStartArray();
            foreach (int slot in Enumerable.Range(0, Slots + 1))
            {
                string path = slot == 0 ? ResumePath : StatePath(slot);
                w.WriteStartObject();
                w.WriteNumber("slot", slot);
                w.WriteString("path", path);
                bool exists = File.Exists(path);
                w.WriteBoolean("exists", exists);
                if (exists) w.WriteString("saved_at", File.GetLastWriteTime(path).ToString("o"));
                else w.WriteNull("saved_at");
                w.WriteEndObject();
            }
            w.WriteEndArray();
        });

        public void SetRewind(bool enabled, uint interval, ulong budget, uint thumbnailWidth) => Change(() =>
        {
            if (interval > 0) Rewind.IntervalFrames = (int)Math.Min(interval, int.MaxValue);
            if (budget > 0) Rewind.BudgetBytes = (long)Math.Min(budget, long.MaxValue);
            Rewind.ThumbnailWidth = (int)Math.Min(thumbnailWidth, int.MaxValue);
            if (enabled && !Rewind.Enabled)
            {
                Rewind.Enabled = true;
                if (EngineFeatures.Of(Core).RewindCapture) Rewind.CaptureNow(Core);
            }
            else if (!enabled && Rewind.Enabled)
            {
                Rewind.Enabled = false;
                Rewind.Clear();
                Runtime.Events.Raise(Id, CrystalNumbers.EventRewind, "rewind", w => w.WriteString("op", "cleared"));
            }
        });

        // Held rewind's step, in place of a frame: the sound it skipped drained and dropped, the picture the snapshot left on show.
        public long StepBack(uint count, FrameInfoV1* info)
        {
            if (info != null && info->size < sizeof(FrameInfoV1)) throw CrystalException.BadStruct("emusen_crystal_frame_info", info->size, sizeof(FrameInfoV1));
            return Change(() =>
            {
                Core.SkipRendering = false;
                long done = 0;
                while (done < count && Rewind.Rewind(Core)) done++;
                Bundle.DebugTarget.RefreshProviders();
                Core.DequeueAudioSamples(int.MaxValue);
                Fill(info, false);
                if (info != null) info->rendered = done > 0 ? 1u : 0u;
                if (done > 0) Runtime.Events.Raise(Id, CrystalNumbers.EventRewind, "rewind", w => { w.WriteString("op", "stepped"); w.WriteNumber("frame", Rewind.Frame); });
                return done;
            });
        }

        public void RewindTo(ulong frame) => Change(() =>
        {
            if (!Rewind.RewindTo(Core, (long)Math.Min(frame, long.MaxValue))) throw new CrystalException(SessionAbi.Absent, $"no moment at frame {frame} is held");
            _halted = false;
            Bundle.DebugTarget.RefreshProviders();
            Runtime.Events.Raise(Id, CrystalNumbers.EventRewind, "rewind", w => { w.WriteString("op", "moved"); w.WriteNumber("frame", frame); });
        });

        public byte[] ReelJson() => Change(() => CrystalJson.Write(w =>
        {
            w.WriteStartObject();
            w.WriteBoolean("enabled", Rewind.Enabled);
            w.WriteNumber("interval_frames", Rewind.IntervalFrames);
            w.WriteNumber("budget_bytes", Rewind.BudgetBytes);
            w.WriteNumber("depth", Rewind.Depth);
            w.WriteNumber("buffered_bytes", Rewind.BufferedBytes);
            w.WriteNumber("snapshot_bytes", Rewind.SnapshotBytes);
            w.WriteNumber("seconds", Rewind.BufferedSeconds(Core.FrameRateHz));
            w.WriteStartArray("moments");
            foreach (RewindMoment m in Rewind.Moments())
            {
                w.WriteStartObject();
                w.WriteNumber("frame", m.Frame);
                w.WriteNumber("core_frames", m.CoreFrames);
                if (m.Thumbnail is null) w.WriteNull("picture");
                else
                {
                    w.WriteStartObject("picture");
                    w.WriteNumber("width", m.Thumbnail.Width);
                    w.WriteNumber("height", m.Thumbnail.Height);
                    w.WriteEndObject();
                }
                w.WriteEndObject();
            }
            w.WriteEndArray();
            w.WriteEndObject();
        }));

        public byte[] RewindPicture(ulong frame) => Change(() =>
        {
            RewindThumbnail thumbnail = Rewind.Moments().FirstOrDefault(m => (ulong)m.Frame == frame).Thumbnail
                ?? throw new CrystalException(SessionAbi.Absent, $"no picture is held for frame {frame}");
            byte[] rgba = thumbnail.ToRgba();
            var bytes = new byte[8 + rgba.Length];
            System.Buffers.Binary.BinaryPrimitives.WriteUInt32LittleEndian(bytes.AsSpan(0, 4), (uint)thumbnail.Width);
            System.Buffers.Binary.BinaryPrimitives.WriteUInt32LittleEndian(bytes.AsSpan(4, 4), (uint)thumbnail.Height);
            rgba.CopyTo(bytes, 8);
            return bytes;
        });
    }
}
