using System;
using System.Collections.Generic;
using System.Globalization;
using System.Linq;
using System.Text;

namespace EmuSen.Pharaoh.Reference
{
    public enum Verdict
    {
        Pass,

        // The frames agree, but the frame could not have disagreed.
        PassVacuous,

        // The two sides were never running the same machine.
        NotComparable,

        Diverged,

        // The screens genuinely differ, and no gate explains it away.
        Differs,
    }

    // Three gates that decide whether a difference means anything, run before the
    // difference is measured - see EmuSen_Debugging_Tools_Reference_v5.md §3.48.
    public static class Comparator
    {
        // A near-uniform frame agrees with almost anything; these are the thresholds
        // below which a match is reported as vacuous rather than as a pass.
        public const int MinimumColours = 6;
        public const double MaximumDominantFraction = 0.90;

        // How close after an input a divergence has to be to be blamed on it.
        public const long InputBlameWindow = 30;

        public sealed class Report
        {
            public Verdict Verdict { get; init; }
            public string Summary { get; init; } = "";
            public List<string> Lines { get; } = new();
        }

        public static Report Compare(DumpSet left, DumpSet right, long frame, int phaseWindow,
            IReadOnlyList<long> inputFrames)
        {
            var report = new Report { Verdict = Verdict.Pass };

            // Proven entries only, and they annotate rather than suppress: a known
            // difference that silently removed a finding would hide the day its
            // cause was fixed - see §3.49.
            using var dictionary = KnownDifferences.Open();

            // Gate 1. Identity, before a single pixel is read.
            List<string> mismatches = IdentityMismatches(left, right).ToList();
            foreach (string warning in IdentityWarnings(left, right)) report.Lines.Add($"  [warn]  {warning}");

            if (mismatches.Count > 0)
            {
                var notComparable = new Report
                {
                    Verdict = Verdict.NotComparable,
                    Summary = $"NOT-COMPARABLE: {mismatches[0]}",
                };
                notComparable.Lines.AddRange(report.Lines);
                foreach (string mismatch in mismatches)
                {
                    notComparable.Lines.Add($"  [ident] {mismatch}");
                    string subject = mismatch.Split(' ')[0];
                    foreach (KnownDifference known in dictionary.Proven("identity", subject, left.SystemName,
                                 left.Backend, right.Backend))
                    {
                        notComparable.Lines.Add($"          known: {known.Slug} - {known.Cause ?? known.Claim}");
                    }
                }
                notComparable.Lines.Add("  The two sides were not running the same machine, so any pixel");
                notComparable.Lines.Add("  difference below would describe that and not a rendering bug.");
                return notComparable;
            }

            // Gate 2. Where they first parted, which is the question worth asking -
            // but only once we know which columns can answer it at all.
            DivergenceResult divergence = FirstDivergence(left, right);
            foreach (ColumnProfile profile in divergence.Profiles)
            {
                var known = dictionary.Proven("column", profile.Name, left.SystemName,
                    left.Backend, right.Backend);

                string note = profile.Rating == Comparability.Stable ? ""
                    : known.Count > 0 ? $"  known: {known[0].Slug}"
                    : "  UNEXPLAINED";

                report.Lines.Add($"  [column] {profile.Name,-10} {profile.AgreementRate * 100,5:F1}% " +
                                 $"{profile.Rating.ToString().ToLowerInvariant()}{note}");
            }

            if (divergence.Columns.Count == 0)
            {
                report.Lines.Add("  [diverge] no shared signature columns; run both sides with --sig");
            }
            else if (divergence.Inconclusive)
            {
                report.Lines.Add("  [diverge] INCONCLUSIVE: no column agrees reliably enough to carry a");
                report.Lines.Add("            verdict, so no divergence frame is claimed. The two");
                report.Lines.Add("            emulators sample memory at different instants; see §3.48a.");
            }
            else if (divergence.Frame is long first)
            {
                long? blamed = inputFrames
                    .Where(f => first >= f && first - f <= InputBlameWindow)
                    .Cast<long?>()
                    .LastOrDefault();

                string where = string.Join(",", divergence.Differing);
                var diverged = new Report
                {
                    Verdict = Verdict.Diverged,
                    Summary = blamed is long input
                        ? $"DIVERGED-AT {first} ({first - input} frames after input@{input}) in {where}"
                        : $"DIVERGED-AT {first} in {where}",
                };
                diverged.Lines.AddRange(report.Lines);
                diverged.Lines.Add($"  [diverge] {divergence.Columns.Count} shared column(s), streams aligned at " +
                                   $"offset {divergence.Offset:+0;-0;0}, agreeing for {divergence.AgreedFrames} frames");
                diverged.Lines.Add(blamed is not null
                    ? "  An input landed just before this, so the two sides are in different"
                    : "  No input precedes this, so the two sides differ deterministically");
                diverged.Lines.Add(blamed is not null
                    ? "  states rather than rendering the same state differently."
                    : "  and this is a real difference worth chasing.");
                AppendScreen(diverged, left, right, frame, phaseWindow);
                return diverged;
            }
            else
            {
                report.Lines.Add($"  [diverge] none across {divergence.FramesCompared} frames of " +
                                 $"{divergence.Columns.Count} shared column(s), offset {divergence.Offset:+0;-0;0}");
            }

            int differing = AppendScreen(report, left, right, frame, phaseWindow);

            // Gate 3 grades an *agreement*. A frame that plainly disagreed is never
            // vacuous, however uniform either side happens to be - reporting a blank
            // screen as a pass because it carries no information is the exact
            // inversion this gate exists to prevent.
            if (differing > 0)
            {
                ScreenImage? shown = left.Screen(frame);
                double percent = shown is null ? 100 : 100.0 * differing / (shown.Width * shown.Height);
                var differs = new Report
                {
                    Verdict = Verdict.Differs,
                    Summary = $"DIFFERS: {differing} px ({percent:F2}%) and no gate explains it",
                };
                differs.Lines.AddRange(report.Lines);
                return differs;
            }

            ScreenImage? ours = left.Screen(frame);
            bool vacuous = ours is null;
            if (ours is not null)
            {
                (int colours, double dominant) = ours.Information();
                vacuous = colours < MinimumColours || dominant > MaximumDominantFraction;
            }
            if (!ReferenceMoves(right, frame, phaseWindow)) vacuous = true;

            var final = new Report
            {
                Verdict = vacuous ? Verdict.PassVacuous : Verdict.Pass,
                Summary = vacuous ? "PASS(vacuous): the frames agree, but could not have disagreed" : "PASS",
            };
            final.Lines.AddRange(report.Lines);
            return final;
        }

        private static IEnumerable<string> IdentityMismatches(DumpSet a, DumpSet b)
        {
            if (Both(a.Board, b.Board) && a.Board != b.Board)
            {
                yield return $"board differs: {a.Backend}={a.Board} {b.Backend}={b.Board}";
            }
            if (Both(a.Region, b.Region) && a.Region != b.Region && a.Region != "auto" && b.Region != "auto")
            {
                yield return $"region differs: {a.Backend}={a.Region} {b.Backend}={b.Region}";
            }
            if (a.PrgBytes > 0 && b.PrgBytes > 0 && a.PrgBytes != b.PrgBytes)
            {
                yield return $"PRG size differs: {a.PrgBytes} vs {b.PrgBytes}";
            }
            if (a.ChrBytes > 0 && b.ChrBytes > 0 && a.ChrBytes != b.ChrBytes)
            {
                yield return $"CHR size differs: {a.ChrBytes} vs {b.ChrBytes}";
            }
            if (Both(a.SystemName, b.SystemName) && a.SystemName != b.SystemName)
            {
                yield return $"system differs: {a.SystemName} vs {b.SystemName}";
            }
        }

        // Not mismatches: things that lower confidence without invalidating the run.
        private static IEnumerable<string> IdentityWarnings(DumpSet a, DumpSet b)
        {
            foreach (DumpSet set in new[] { a, b })
            {
                if (set.HeaderTrust is "archaic")
                {
                    yield return $"{set.Backend} read a {set.HeaderTrust} header; the board it chose is a guess";
                }
            }

            if (!Both(a.Board, b.Board))
            {
                string silent = a.Board.Length == 0 ? a.Backend : b.Backend;
                yield return $"{silent} cannot report its board, so the identity gate is one-sided";
            }
            if (a.SaveLoaded != b.SaveLoaded)
            {
                yield return $"save RAM present on only one side ({a.Backend}={a.SaveLoaded}, {b.Backend}={b.SaveLoaded})";
            }
        }

        private static bool Both(string a, string b) => a.Length > 0 && b.Length > 0;

        // How reliably a column can be compared between two given emulators, which
        // is measured rather than assumed - see §3.48a.
        public enum Comparability
        {
            // Agrees whenever the machines agree: usable as a divergence signal.
            Stable,

            // Agrees sometimes. Two emulators do not put their frame boundary at the
            // same instant, so a space the game is mid-way through writing is caught
            // at different points and disagrees without anything being wrong.
            Intermittent,

            // Never agrees, at any alignment. Undefined power-on contents, a
            // different mirror size, or a different internal layout.
            Never,
        }

        public sealed record ColumnProfile(string Name, Comparability Rating, double AgreementRate);

        public sealed class DivergenceResult
        {
            public long? Frame { get; init; }
            public List<string> Columns { get; init; } = new();
            public List<string> Differing { get; init; } = new();
            public long FramesCompared { get; init; }
            public long Offset { get; init; }
            public long AgreedFrames { get; init; }
            public List<ColumnProfile> Profiles { get; init; } = new();

            // Nothing agreed reliably enough to carry a verdict either way.
            // Structural columns cannot carry evidence however well they rate.
            public List<string> Usable { get; init; } = new();

            public bool Inconclusive => Usable.Count == 0;
        }

        // How far apart two emulators may be on the boot sequence before the search
        // gives up; the observed offset against the reference is two frames.
        public const int MaxStreamOffset = 16;

        // Intersects on column names rather than assuming two backends expose the
        // same spaces, and ignores the screen column unless both formats agree.
        //
        // The offset search is not optional. Two emulators do not agree on how many
        // frames a boot takes - ours runs two behind the reference - so comparing
        // frame N to frame N would report a divergence at the first row for every
        // ROM ever tested, which is a gate that only ever cries wolf. The stream is
        // aligned first, on the offset that agrees longest, and the divergence is
        // whatever survives that.
        public static DivergenceResult FirstDivergence(DumpSet a, DumpSet b)
        {
            var shared = a.Columns.Intersect(b.Columns).ToList();
            if (a.ScreenFormat != b.ScreenFormat) shared.Remove("screen");

            // Our nametable mirror is 4 KB against the reference's physical 2 KB, and
            // CHR banking differs in layout without differing in effect, so neither
            // column can agree byte for byte even when the machines do. They are
            // still rated, because a dictionary claim about them has to be
            // verifiable; they are only barred from carrying divergence evidence.
            var structural = new[] { "nametable", "chr" };

            if (shared.Count == 0 || a.Signature.Count == 0 || b.Signature.Count == 0)
            {
                return new DivergenceResult { Columns = shared };
            }

            // Aligned on whichever column agrees most, because no column is known in
            // advance to be the trustworthy one - see §3.48a.
            // A column that never changes agrees at every offset, so it cannot say
            // where the streams line up - alignment uses only columns that move.
            var informative = shared.Where(c => Varies(a, c) && Varies(b, c)).ToList();
            if (informative.Count == 0) informative = shared;

            int bestOffset = 0;
            double bestRate = -1;
            foreach (int offset in Enumerable.Range(-MaxStreamOffset, (MaxStreamOffset * 2) + 1))
            {
                double rate = informative.Max(c => AgreementRate(a, b, c, offset));
                if (rate > bestRate) { bestRate = rate; bestOffset = offset; }
            }

            // Rated over a leading window rather than the whole stream. Rating over
            // everything is self-defeating: a real divergence half way through drags
            // the column's rate down and disqualifies the one column that would have
            // revealed it. Calibrate on the early frames, then judge the rest.
            long calibrationEnd = CalibrationEnd(a, b, bestOffset);
            var profiles = shared
                .Select(c =>
                {
                    double rate = AgreementRate(a, b, c, bestOffset, calibrationEnd);
                    return new ColumnProfile(c, Rate(rate), rate);
                })
                .ToList();

            var usable = profiles
                .Where(p => p.Rating == Comparability.Stable && !structural.Contains(p.Name))
                .Select(p => p.Name)
                .ToList();
            DivergenceResult scan = ScanAt(a, b, usable, bestOffset);

            return new DivergenceResult
            {
                Frame = scan.Frame,
                Columns = shared,
                Differing = scan.Differing,
                FramesCompared = scan.FramesCompared,
                Offset = bestOffset,
                AgreedFrames = scan.AgreedFrames,
                Profiles = profiles,
                Usable = usable,
            };
        }

        // Above this a column tracks the machine; below the lower bound it is telling
        // us about the emulators rather than about the game.
        public const double StableRate = 0.95;
        public const double NeverRate = 0.05;

        private static Comparability Rate(double rate) =>
            rate >= StableRate ? Comparability.Stable
            : rate <= NeverRate ? Comparability.Never
            : Comparability.Intermittent;

        private static bool Varies(DumpSet set, string column) =>
            set.Signature.Values
                .Where(r => r.ContainsKey(column))
                .Select(r => r[column])
                .Distinct()
                .Take(2)
                .Count() > 1;

        // The share of the run treated as known-good for rating purposes.
        public const double CalibrationShare = 0.25;
        public const long MinimumCalibrationFrames = 10;

        private static long CalibrationEnd(DumpSet a, DumpSet b, int offset)
        {
            var common = a.Signature.Keys.Where(f => b.Signature.ContainsKey(f + offset)).OrderBy(f => f).ToList();
            if (common.Count == 0) { return long.MaxValue; }

            long span = (long)(common.Count * CalibrationShare);
            int index = (int)System.Math.Min(common.Count - 1, System.Math.Max(MinimumCalibrationFrames, span));
            return common[index];
        }

        private static double AgreementRate(DumpSet a, DumpSet b, string column, int offset,
            long throughFrame = long.MaxValue)
        {
            long common = 0, agreed = 0;
            foreach ((long frame, Dictionary<string, uint> left) in a.Signature)
            {
                if (frame > throughFrame) continue;
                if (!b.Signature.TryGetValue(frame + offset, out Dictionary<string, uint>? right)) continue;
                if (!left.TryGetValue(column, out uint x) || !right.TryGetValue(column, out uint y)) continue;
                common++;
                if (x == y) agreed++;
            }
            return common == 0 ? 0 : agreed / (double)common;
        }

        private static DivergenceResult ScanAt(DumpSet a, DumpSet b, List<string> shared, int offset)
        {
            var frames = a.Signature.Keys
                .Where(f => b.Signature.ContainsKey(f + offset))
                .OrderBy(f => f)
                .ToList();

            long agreed = 0;
            foreach (long frame in frames)
            {
                Dictionary<string, uint> left = a.Signature[frame];
                Dictionary<string, uint> right = b.Signature[frame + offset];

                var differing = shared
                    .Where(c => left.TryGetValue(c, out uint x) && right.TryGetValue(c, out uint y) && x != y)
                    .ToList();

                if (differing.Count > 0)
                {
                    return new DivergenceResult
                    {
                        Frame = frame,
                        Columns = shared,
                        Differing = differing,
                        FramesCompared = frames.Count,
                        Offset = offset,
                        AgreedFrames = agreed,
                    };
                }
                agreed++;
            }

            return new DivergenceResult
            {
                Columns = shared,
                FramesCompared = frames.Count,
                Offset = offset,
                AgreedFrames = agreed,
            };
        }

        private static int AppendScreen(Report report, DumpSet left, DumpSet right, long frame, int phaseWindow)
        {
            ScreenImage? ours = left.Screen(frame);
            if (ours is null)
            {
                report.Lines.Add($"  [screen] {left.Backend} has no screen at frame {frame}");
                return 0;
            }

            (int colours, double dominant) = ours.Information();
            report.Lines.Add($"  [screen] {colours} colours, dominant {dominant * 100:F1}%");

            int best = int.MaxValue;
            long bestFrame = frame;
            for (long f = frame - phaseWindow; f <= frame + phaseWindow; f++)
            {
                ScreenImage? theirs = right.Screen(f);
                if (theirs is null) continue;
                int differing = ours.DifferingPixels(theirs);
                if (differing < best) { best = differing; bestFrame = f; }
            }

            if (best == int.MaxValue)
            {
                report.Lines.Add($"  [screen] {right.Backend} has no screen within +/-{phaseWindow} of {frame}");
                return 0;
            }

            double percent = 100.0 * best / (ours.Width * ours.Height);
            report.Lines.Add($"  [screen] best {best} px ({percent:F2}%) at {right.Backend} frame {bestFrame}, " +
                             $"phase {bestFrame - frame:+0;-0;0}");

            if (!ReferenceMoves(right, frame, phaseWindow))
            {
                report.Lines.Add("  [screen] the reference is static across the phase window, so a phase");
                report.Lines.Add("           match here proves nothing about timing");
            }

            if (best > 0)
            {
                ScreenImage? theirs = right.Screen(bestFrame);
                if (theirs is not null && ours.VerticalShift(theirs) is int dy)
                {
                    report.Lines.Add($"  [screen] the whole image is offset by {dy:+0;-0} scanline(s), which is a");
                    report.Lines.Add("           timing difference rather than a drawing one");
                }
            }

            return best;
        }

        // A reference that does not change across the window cannot falsify a phase.
        private static bool ReferenceMoves(DumpSet right, long frame, int phaseWindow)
        {
            ScreenImage? first = null;
            for (long f = frame - phaseWindow; f <= frame + phaseWindow; f++)
            {
                ScreenImage? image = right.Screen(f);
                if (image is null) continue;
                if (first is null) { first = image; continue; }
                if (first.DifferingPixels(image) > 0) return true;
            }
            return false;
        }

        public static string Render(Report report)
        {
            var text = new StringBuilder();
            text.AppendLine(report.Summary);
            foreach (string line in report.Lines) text.AppendLine(line);
            return text.ToString();
        }
    }
}
