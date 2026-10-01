using System;
using System.Diagnostics;
using System.Globalization;
using System.IO;
using System.Text;
using System.Threading;

namespace EmuSen.Serenity
{
    // Per-frame and per-draw timestamps written to $EMUSEN_PRESENT_TRACE, for the cadence measurement of EmuSen_Settings_Reference.md §4.87.1.
    public static class PresentationTrace
    {
        private const int Capacity = 1 << 20;

        public static readonly string? Path = Environment.GetEnvironmentVariable("EMUSEN_PRESENT_TRACE") is { Length: > 0 } p ? p : null;
        public static bool Enabled => Path is not null;

        private static readonly long[] _frames = Enabled ? new long[Capacity * 4] : Array.Empty<long>();
        private static readonly long[] _draws = Enabled ? new long[Capacity * 7] : Array.Empty<long>();
        private static int _frameCount, _drawCount, _flushed;
        private static string _note = "";

        static PresentationTrace()
        {
            if (Enabled) AppDomain.CurrentDomain.ProcessExit += (_, _) => Flush();
        }

        // Free text written at the head of the file: the rate, the mode, the refresh as measured.
        public static void Note(string note) { if (Enabled) _note = note; }

        // The emulation thread: frame seq ran from start to offered (Stopwatch ticks); offered 0 when nothing new was handed over.
        public static void Frame(long seq, long start, long offered, bool handedOver)
        {
            if (!Enabled) return;
            int i = Interlocked.Increment(ref _frameCount) - 1;
            if (i >= Capacity) return;
            int o = i * 4;
            _frames[o] = seq; _frames[o + 1] = start; _frames[o + 2] = offered; _frames[o + 3] = handedOver ? 1 : 0;
        }

        // The render thread: a draw at time t of frame seq, fresh when it is a picture not drawn before, with the vblank sample if any.
        public static void Draw(long t, long seq, bool fresh, bool sampled, long ust, long msc, long sbc)
        {
            if (!Enabled) return;
            int i = Interlocked.Increment(ref _drawCount) - 1;
            if (i >= Capacity) return;
            int o = i * 7;
            _draws[o] = t; _draws[o + 1] = seq; _draws[o + 2] = fresh ? 1 : 0; _draws[o + 3] = sampled ? 1 : 0;
            _draws[o + 4] = ust; _draws[o + 5] = msc; _draws[o + 6] = sbc;
        }

        public static void Flush()
        {
            if (!Enabled || Interlocked.Exchange(ref _flushed, 1) == 1) return;
            var sb = new StringBuilder();
            sb.Append("# freq ").Append(Stopwatch.Frequency.ToString(CultureInfo.InvariantCulture)).Append(' ').Append(_note).Append('\n');
            int frames = Math.Min(_frameCount, Capacity), draws = Math.Min(_drawCount, Capacity);
            for (int i = 0; i < frames; i++)
            {
                int o = i * 4;
                sb.Append("F,").Append(_frames[o]).Append(',').Append(_frames[o + 1]).Append(',').Append(_frames[o + 2]).Append(',').Append(_frames[o + 3]).Append('\n');
            }
            for (int i = 0; i < draws; i++)
            {
                int o = i * 7;
                sb.Append("D");
                for (int k = 0; k < 7; k++) sb.Append(',').Append(_draws[o + k]);
                sb.Append('\n');
            }
            File.WriteAllText(Path!, sb.ToString());
        }
    }
}
