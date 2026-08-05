using EmuSen.Cauldron;
using System;
using System.Linq;
using EmuSen.DianaOS.DianaOS.Bin;
using EmuSen.DianaOS.DianaOS.Etc;
using EmuSen.DianaOS.DianaOS.Lib;
using EmuSen.DianaOS.DianaOS.Var;
using EmuSen.DianaOS.DianaOS.Dev;
using static EmuSen.DianaOS.DianaOS.Bin.Commands.EmuSen.DebugCommandHelpers;

namespace EmuSen.DianaOS.DianaOS.Bin.Commands.EmuSen
{
    // Writes a CPU register, so a halted machine can be pushed down the other branch - see `man setreg`.
    public class SetRegCommand : IDianaOSCommand
    {
        public string Name => "setreg";
        public bool IsReadOnly => false;
        public string Usage => string.Join('\n', new[]
        {
            "  setreg <reg> <value>          write one of the main CPU's registers",
            "  setreg <cpu> <reg> <value>    same, on another processor - `cpus` lists them",
        });

        public DianaOSResult Execute(IDebugTarget? target, string[] parts, string? stdin)
        {
            IDebugTarget resolved;
            try { resolved = RequireTarget(target); }
            catch (Exception ex) { return DianaOSResult.Fail(ex.Message); }

            if (parts.Length < 3) return "Usage: setreg [<cpu>] <register> <value>";

            // A lone register name is not a scope word, so only look for one when there is room.
            int at = 1;
            DebugCpu? cpu = resolved.DebugCpus.Count > 0 ? resolved.DebugCpus[0] : null;
            if (parts.Length > 3 && DebugCpus.Find(resolved.DebugCpus, parts[1]) is { } named)
            {
                cpu = named;
                at = 2;
            }

            if (cpu == null) return DianaOSResult.Fail($"{resolved.CoreName} publishes no debug CPUs, so it has no writable registers.");
            if (parts.Length < at + 2) return "Usage: setreg [<cpu>] <register> <value>";

            string name = parts[at];
            ulong value;
            try { value = (ulong)ParseHex(parts[at + 1]); }
            catch (Exception) { return DianaOSResult.Fail($"'{parts[at + 1]}' is not a value."); }

            var known = cpu.Registers?.Current ?? Array.Empty<DebugRegisterValue>();
            var register = known.FirstOrDefault(r => string.Equals(r.Name, name, StringComparison.OrdinalIgnoreCase));
            if (register.Name == null)
            {
                string available = known.Count == 0 ? "(none)" : string.Join(", ", known.Select(r => r.Name));
                return DianaOSResult.Fail($"{cpu.Name} has no register named '{name}'. Available: {available}.");
            }

            if (cpu.RegisterWriter == null)
                return DianaOSResult.Fail($"{cpu.Name} reports its registers but this core cannot write them - see `man cpus`.");

            // Refused rather than truncated: a silently masked value is a wrong answer, not a warning.
            ulong max = register.BitWidth >= 64 ? ulong.MaxValue : (1UL << register.BitWidth) - 1;
            if (value > max)
                return DianaOSResult.Fail($"{register.Name} is {register.BitWidth}-bit, so 0x{value:X} does not fit (max 0x{max:X}).");

            ulong previous = register.Value;
            if (!cpu.RegisterWriter(register.Name, value))
                return DianaOSResult.Fail($"{cpu.Name}'s {register.Name} is read-only on this core.");

            cpu.Registers?.Refresh();
            int digits = Math.Max(1, register.BitWidth / 4);
            return $"{cpu.Name} {register.Name} = 0x{value.ToString("X" + digits)} (was 0x{previous.ToString("X" + digits)}).";
        }
    }
}
