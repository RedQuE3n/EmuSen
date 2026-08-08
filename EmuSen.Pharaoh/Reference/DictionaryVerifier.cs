using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.Globalization;
using System.Linq;

namespace EmuSen.Pharaoh.Reference
{
    // Re-runs every assertion against a real pair of dump sets, and promotes or
    // demotes on the result. This is the only path to 'proven' - see §3.49.
    public static class DictionaryVerifier
    {
        public static int Run(string[] args)
        {
            DumpSet? ours = DumpSet.Load(args[1]);
            DumpSet? theirs = DumpSet.Load(args[2]);

            if (ours is null) { Console.WriteLine($"[ERROR] no dump set in {args[1]}"); return 1; }
            if (theirs is null) { Console.WriteLine($"[ERROR] no dump set in {args[2]}"); return 1; }

            using var dictionary = KnownDifferences.Open();
            string? commit = CurrentCommit();

            Comparator.DivergenceResult divergence = Comparator.FirstDivergence(ours, theirs);
            var rates = divergence.Profiles.ToDictionary(p => p.Name, p => p.AgreementRate);

            int passed = 0, failed = 0, skipped = 0;
            foreach (DictionaryAssertion assertion in dictionary.Assertions())
            {
                if (!AppliesTo(assertion, ours))
                {
                    Console.WriteLine($"  SKIP  {assertion.Slug}: needs fixture '{assertion.Fixture}'");
                    skipped++;
                    continue;
                }

                (bool? holds, string observed) = Evaluate(assertion, ours, theirs, rates);

                if (holds is null)
                {
                    Console.WriteLine($"  SKIP  {assertion.Slug}: {observed}");
                    skipped++;
                    continue;
                }

                dictionary.RecordVerification(assertion, holds.Value, observed, commit);

                if (holds.Value)
                {
                    bool promoted = dictionary.TryPromote(assertion.DefinitionId, out string error);
                    Console.WriteLine($"  PASS  {assertion.Slug}: {observed}" +
                                      (promoted ? "  [promoted to proven]" : error.Length > 0 ? $"  [{error}]" : ""));
                    passed++;
                }
                else
                {
                    // A definition that stops holding is demoted, not deleted: the
                    // claim may still be true elsewhere, but it has stopped being
                    // something this comparator may act on.
                    dictionary.Demote(assertion.DefinitionId);
                    Console.WriteLine($"  FAIL  {assertion.Slug}: {observed}  [demoted to provisional]");
                    failed++;
                }
            }

            (int total, int proven, int provisional, int retracted) = dictionary.Counts();
            Console.WriteLine();
            Console.WriteLine($"{passed} passed, {failed} failed, {skipped} skipped against this pair");
            Console.WriteLine($"dictionary: {total} definitions - {proven} proven, {provisional} provisional, {retracted} retracted");
            return failed > 0 ? 1 : 0;
        }

        // An assertion demonstrated on one image says nothing about another, and
        // running it anyway is how a correct claim gets demoted by an irrelevant
        // pair - which happened on the first run of this verifier.
        private static bool AppliesTo(DictionaryAssertion assertion, DumpSet ours)
        {
            if (string.IsNullOrWhiteSpace(assertion.Fixture)) return true;
            if (assertion.Fixture.StartsWith("any ", StringComparison.Ordinal)) return true;
            return ours.Rom.Contains(assertion.Fixture, StringComparison.OrdinalIgnoreCase);
        }

        // Null means the pair cannot decide this assertion, which is not a failure:
        // a claim about PAL images says nothing when handed two NTSC ones.
        private static (bool? Holds, string Observed) Evaluate(DictionaryAssertion assertion,
            DumpSet ours, DumpSet theirs, IReadOnlyDictionary<string, double> rates)
        {
            switch (assertion.Kind)
            {
                case "column-agreement":
                    if (!rates.TryGetValue(assertion.Subject, out double rate))
                    {
                        return (null, $"neither side exposes a '{assertion.Subject}' column");
                    }
                    return (Holds(rate, assertion.Op, assertion.Value),
                        $"{assertion.Subject} agreement {rate * 100:F1}% {assertion.Op} {assertion.Value * 100:F1}%");

                case "identity-field":
                    string mine = Field(ours, assertion.Subject);
                    string yours = Field(theirs, assertion.Subject);
                    if (mine.Length == 0 || yours.Length == 0)
                    {
                        return (null, $"'{assertion.Subject}' not reported by both sides");
                    }
                    bool differs = mine != yours;
                    return (assertion.Op == "<>" ? differs : !differs,
                        $"{assertion.Subject}: {ours.Backend}={mine} {theirs.Backend}={yours}");

                case "screen-shift":
                    ScreenImage? a = ours.Screen((long)assertion.Value);
                    ScreenImage? b = theirs.Screen((long)assertion.Value);
                    if (a is null || b is null) return (null, "no screen at the assertion's frame");
                    int? shift = a.VerticalShift(b);
                    return (shift is not null, shift is null ? "no whole-image shift found" : $"shift {shift:+0;-0}");

                default:
                    return (null, $"unknown assertion kind '{assertion.Kind}'");
            }
        }

        private static bool Holds(double observed, string op, double value) => op switch
        {
            "<=" => observed <= value,
            ">=" => observed >= value,
            "=" => Math.Abs(observed - value) < 1e-9,
            _ => Math.Abs(observed - value) >= 1e-9,
        };

        private static string Field(DumpSet set, string name) => name switch
        {
            "region" => set.Region,
            "board" => set.Board,
            "headerTrust" => set.HeaderTrust,
            "system" => set.SystemName,
            _ => "",
        };

        private static string? CurrentCommit()
        {
            try
            {
                using Process? git = Process.Start(new ProcessStartInfo("git", "rev-parse --short HEAD")
                {
                    RedirectStandardOutput = true,
                    RedirectStandardError = true,
                });
                if (git is null) return null;
                string sha = git.StandardOutput.ReadToEnd().Trim();
                git.WaitForExit();
                return sha.Length > 0 ? sha : null;
            }
            catch (Exception)
            {
                return null;
            }
        }
    }
}
