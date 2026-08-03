using System;
using System.Collections.Generic;

namespace EmuSen.Validation
{
    // Result of running one file's worth of test cases (typically all the
    // vectors for a single opcode) against a target.
    public class SingleStepFileResult
    {
        public string FileName = "";
        public int Pass;
        public int Fail;
        public int Total => Pass + Fail;

        // A handful of "<test name>: <field>: got <X> want <Y>" strings -
        // capped (see SingleStepTestRunner.RunFile's maxExamples) since a
        // systematically wrong opcode can fail all several thousand of
        // its own tests and nobody needs to see all of them to diagnose
        // it; the first few are almost always enough, as every real bug
        // found this way so far (in both the 65816 and SPC700 CPUs) has
        // shown.
        public List<string> Examples = new();
    }

    // The shared engine behind "run these ground-truth vectors against
    // this CPU and tell me exactly what's wrong" - core-agnostic, built
    // once against ISingleStepTarget and reused for the 65816
    // (SingleStepTests) and SPC700 (ProcessorTests) validation passes
    // already done, and for any future core's own CPU-like component the
    // same way. Deliberately checks registers before memory, and stops at
    // the first mismatched field - a single wrong flag is just as
    // diagnosable as reporting every difference at once, and keeping only
    // the first keeps failure examples short and readable.
    public static class SingleStepTestRunner
    {
        public static SingleStepFileResult RunFile(
            ISingleStepTarget target,
            string fileName,
            IReadOnlyList<SingleStepTest> tests,
            int maxExamples = 3)
        {
            var result = new SingleStepFileResult { FileName = fileName };

            foreach (var t in tests)
            {
                target.Reset();
                foreach (var (address, value) in t.InitialMemory)
                {
                    target.SetMemory(address, value);
                }
                foreach (var (name, value) in t.InitialRegisters)
                {
                    target.SetRegister(name, value);
                }

                string? mismatch = null;
                try
                {
                    target.Step();

                    foreach (var (name, want) in t.FinalRegisters)
                    {
                        int got = target.GetRegister(name);
                        if (got != want)
                        {
                            mismatch = $"{name}: got {got:X} want {want:X}";
                            break;
                        }
                    }

                    if (mismatch is null)
                    {
                        foreach (var (address, want) in t.FinalMemory)
                        {
                            byte got = target.GetMemory(address);
                            if (got != want)
                            {
                                mismatch = $"MEM[{address:X}]: got {got:X2} want {want:X2}";
                                break;
                            }
                        }
                    }

                    // Checked last, so a state mismatch outranks the cycle bug it usually causes.
                    if (mismatch is null && t.ExpectedTrace is not null && target is IBusTraceTarget traceTarget)
                    {
                        mismatch = CompareTrace(t.ExpectedTrace, traceTarget.LastStepTrace);
                    }
                }
                catch (Exception ex)
                {
                    mismatch = $"EXCEPTION: {ex.Message}";
                }

                if (mismatch is null)
                {
                    result.Pass++;
                }
                else
                {
                    result.Fail++;
                    if (result.Examples.Count < maxExamples)
                    {
                        result.Examples.Add($"{t.Name}: {mismatch}");
                    }
                }
            }

            return result;
        }

        // Count first, then the first differing access - the count names a wrong dummy read fastest.
        private static string? CompareTrace(List<BusAccess> want, IReadOnlyList<BusAccess> got)
        {
            if (got.Count != want.Count)
            {
                return $"CYCLES: got {got.Count} want {want.Count}";
            }

            for (int i = 0; i < want.Count; i++)
            {
                if (got[i] != want[i])
                {
                    return $"CYCLE[{i}]: got {Describe(got[i])} want {Describe(want[i])}";
                }
            }

            return null;
        }

        private static string Describe(BusAccess access) =>
            $"{(access.IsWrite ? "write" : "read")} {access.Value:X2} @ {access.Address:X4}";
    }
}
