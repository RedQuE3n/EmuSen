using System;
using System.Collections.Generic;
using System.Linq;
using EmuSen.DianaOS.DianaOS.Bin;
using EmuSen.DianaOS.DianaOS.Etc;
using EmuSen.DianaOS.DianaOS.Lib;
using EmuSen.DianaOS.DianaOS.Var;
using EmuSen.DianaOS.DianaOS.Dev;

namespace EmuSen.DianaOS.DianaOS.Bin.Commands.EmuSen
{
    // Stateless helpers several commands share, the same shape DebugTools.cs has.
    public static class DebugCommandHelpers
    {
        public static int ParseHex(string s)
        {
            s = s.Trim();
            if (s.StartsWith("0x", StringComparison.OrdinalIgnoreCase)) s = s.Substring(2);
            else if (s.StartsWith("$")) s = s.Substring(1);
            return Convert.ToInt32(s, 16);
        }

        // "<start>-<end>", or a lone address that becomes a one-byte range - see `man bp`.
        public static (int Start, int End) ParseRange(string s)
        {
            s = s.Trim();
            int split = s.IndexOf('-', 1);
            if (split < 0) { int only = ParseHex(s); return (only, only); }

            int start = ParseHex(s.Substring(0, split));
            int end = ParseHex(s.Substring(split + 1));
            return end < start ? (end, start) : (start, end);
        }

        // One null check, so every debug command gets "no ROM loaded" instead of an NRE - see §3.3.
        public static IDebugTarget RequireTarget(IDebugTarget? target)
        {
            if (target == null) throw new InvalidOperationException("No ROM loaded - this command needs an active debug target.");
            return target;
        }

        // The target's own context when it publishes one - see `man eval`.
        public static IExpressionContext ExpressionsFor(IDebugTarget target)
            => target.Expressions ?? new DebugTargetExpressionContext(target);

        // A chip's own context, else one derived from its reported registers - see `man eval`.
        public static IExpressionContext ExpressionsFor(IDebugTarget target, DebugCpu? cpu)
            => cpu == null ? ExpressionsFor(target)
             : cpu.Expressions ?? DebugCpuExpressionContext.For(target, cpu);

        // Which chip a scoped command is aimed at, and where its real args start - see `man cpus`.
        public static (DebugCpu? Cpu, int Next) ResolveCpu(IDebugTarget target, string[] parts, int at)
        {
            var cpus = target.DebugCpus;
            if (cpus.Count == 0 || at >= parts.Length) return (null, at);

            if (DebugCpus.Find(cpus, parts[at]) is { } named) return (named, at + 1);

            // `cop` predates chips having names - see `man cpus`.
            if (string.Equals(parts[at], "cop", StringComparison.OrdinalIgnoreCase))
            {
                return (DebugCpus.Coprocessor(cpus), at + 1);
            }

            return (cpus[0], at);
        }

        public static string NoSuchCpu(IDebugTarget target, string word)
            => $"No debug CPU named '{word}' on this {target.CoreName}. Available: {DebugCpus.NameList(target.DebugCpus)}.";

        public static IDebugMemorySpace FindSpace(IDebugTarget? target, string name)
        {
            IDebugTarget t = RequireTarget(target);
            var spaces = t.GetMemorySpaces();
            var match = spaces.FirstOrDefault(s => string.Equals(s.Name, name, StringComparison.OrdinalIgnoreCase));
            if (match == null)
            {
                string available = string.Join(", ", spaces.Select(s => s.Name));
                throw new ArgumentException($"No memory space named '{name}'. Available: {available}");
            }
            return match;
        }

        // Little-endian, matching every other multi-byte value in this codebase.
        public static long ReadValue(IDebugMemorySpace space, int addr, int width)
        {
            long v = 0;
            for (int i = 0; i < width; i++) v |= (long)space.Read(addr + i) << (8 * i);
            return v;
        }

        // Absent an explicit range, default to the bank's upper 32KB - see §3.12.
        public static (int ScanStart, int ScanLen) DefaultScanRange(string[] parts, int argOffset, int targetAddr)
        {
            if (parts.Length >= argOffset + 2)
            {
                return (ParseHex(parts[argOffset]), ParseHex(parts[argOffset + 1]));
            }
            return ((targetAddr & 0xFF0000) | 0x8000, 0x8000);
        }

        // The walking and matching all three scan commands share; the ISA knowledge is the core's - see §3.1a.
        public static string ScanForStaticReferences(IDebugTarget target, int scanStart, int scanLen, int targetAddr, StaticReferenceKind kind)
        {
            var instrs = target.Disassemble("CpuBus", scanStart, scanLen);

            var matches = new List<string>();
            foreach (var instr in instrs)
            {
                if (instr.Address >= scanStart + scanLen) break;
                var reference = target.ClassifyStaticReference(instr);
                if (reference is { } r && r.Kind == kind && r.Target == targetAddr)
                {
                    matches.Add($"  ${instr.Address:X6}: {instr.Mnemonic} {instr.OperandText}");
                }
            }

            if (matches.Count == 0)
            {
                string verb = kind switch
                {
                    StaticReferenceKind.Call => "calls of",
                    StaticReferenceKind.Write => "writes to",
                    StaticReferenceKind.Read => "reads of",
                    _ => "references to",
                };
                return $"No statically-resolvable {verb} ${targetAddr:X6} found in ${scanStart:X6}-${scanStart + scanLen - 1:X6}.";
            }
            return string.Join('\n', matches);
        }
    }
}
