using System;
using System.Collections.Generic;
using System.Linq;
using System.Text.Json;
using EmuSen.Crystal.Debugger;
using EmuSen.Crystal.Telemetry;

namespace EmuSen.Crystal.Systems.N64
{
    // The N64's debugger views, emusen-systems' n64 module unless EMUSEN_SYSTEMS_NATIVE=0, each …Managed the C# moved from the MarsRT shim; read by the shim now and by the generic target later - see EmuSen_Backend.md §18.7.
    public static class N64Views
    {
        // The space every N64 engine names the VI's fourteen registers, big-endian.
        public const string Vi = "VI";

        public static readonly IReadOnlyList<string> VideoRegisterNames = new[]
        {
            "CONTROL", "ORIGIN", "WIDTH", "V_INTR", "V_CURRENT", "BURST", "V_SYNC", "H_SYNC", "LEAP", "H_START", "V_START", "V_BURST", "X_SCALE", "Y_SCALE",
        };

        private static readonly IReadOnlyList<IDebugMemorySpace> NoSpaces = Array.Empty<IDebugMemorySpace>();

        private static NativeDebugViews.Answer? Ask(string view, IReadOnlyList<IDebugMemorySpace> spaces, Action<Utf8JsonWriter>? fields = null) =>
            SystemCode.Active && spaces.All(s => SystemCode.Takes(s.Name)) ? NativeDebugViews.Request(N64System.Id, view, spaces, fields) : null;

        private static MarsInstruction Instruction(JsonElement r)
        {
            JsonElement reference = r.GetProperty("reference");
            return reference.ValueKind == JsonValueKind.Null
                ? new MarsInstruction(r.GetProperty("mnemonic").GetString()!, r.GetProperty("operands").GetString()!, null, 0)
                : new MarsInstruction(r.GetProperty("mnemonic").GetString()!, r.GetProperty("operands").GetString()!,
                    Enum.Parse<StaticReferenceKind>(reference.GetProperty("kind").GetString()!), unchecked((uint)reference.GetProperty("target").GetInt64()));
        }

        // One word of a space decoded by the processor that runs code there, at the address it sees it at.
        public static MarsInstruction DecodeIn(string space, uint word, uint at) =>
            SystemCode.Takes(space) && Ask("instruction", NoSpaces, w => { w.WriteString("space", space); w.WriteNumber("address", unchecked((int)at)); w.WriteNumber("word", word); }) is { } a
                ? Instruction(a.Root)
                : DecodeInManaged(space, word, at);

        public static MarsInstruction DecodeInManaged(string space, uint word, uint at)
        {
            bool Is(string name) => string.Equals(space, name, StringComparison.OrdinalIgnoreCase);
            if (Is("IMEM") || Is("DMEM")) return RspDisassembler.DecodeManaged(word, at & 0xFFC);
            if (Is("RDRAM")) return Vr4300Disassembler.DecodeManaged(word, 0x8000_0000u | at);
            if (Is("ROM")) return Vr4300Disassembler.DecodeManaged(word, unchecked(0xB000_0000u + at));
            if (Is("PIFRAM")) return Vr4300Disassembler.DecodeManaged(word, unchecked(0xBFC0_07C0u + at));
            return Vr4300Disassembler.DecodeManaged(word, at);
        }

        // Which memory a physical address names given the RDRAM installed, or the register block it falls in, not addressable - see Mars_Memory.md §2.
        public static PhysicalAddress Physical(uint physical, long rdramLength)
        {
            var rdram = new IDebugMemorySpace[] { new DelegateDebugMemorySpace("RDRAM", (int)Math.Min(int.MaxValue, rdramLength), _ => 0) };
            return Ask("physical", rdram, w => w.WriteNumber("address", unchecked((int)physical))) is { } a
                ? new PhysicalAddress(a.Root.GetProperty("space").GetString()!, a.Root.GetProperty("offset").GetInt32(), a.Root.GetProperty("addressable").GetBoolean())
                : PhysicalManaged(physical, rdramLength);
        }

        public static PhysicalAddress PhysicalManaged(uint physical, long rdramLength)
        {
            const uint RdramRegistersBase = 0x03F0_0000, SpDmemBase = 0x0400_0000, SpRegistersBase = 0x0404_0000, SpMemSize = 0x1000;
            const uint CartDomain2Address1 = 0x0500_0000, CartDomain2Address2 = 0x0800_0000, CartDomain1Address2 = 0x1000_0000;
            const uint PifRomBase = 0x1FC0_0000, PifRamBase = 0x1FC0_07C0, PifRamSize = 64;
            if (physical < rdramLength) return new PhysicalAddress("RDRAM", (int)physical);
            if (physical < RdramRegistersBase) return new PhysicalAddress("RDRAM (not installed)", (int)physical, false);
            if (physical < SpDmemBase) return new PhysicalAddress("RDRAM registers", (int)(physical - RdramRegistersBase), false);
            if (physical < SpRegistersBase)
            {
                uint local = (physical - SpDmemBase) % (2 * SpMemSize);
                return local < SpMemSize ? new PhysicalAddress("DMEM", (int)local) : new PhysicalAddress("IMEM", (int)(local - SpMemSize));
            }
            if (physical < CartDomain2Address1) return new PhysicalAddress("interface registers", (int)physical, false);
            if (physical < CartDomain2Address2) return new PhysicalAddress("64DD", (int)physical, false);
            if (physical < CartDomain1Address2) return new PhysicalAddress("save chip", (int)(physical - CartDomain2Address2), false);
            if (physical >= PifRamBase && physical < PifRamBase + PifRamSize) return new PhysicalAddress("PIFRAM", (int)(physical - PifRamBase));
            if (physical < PifRomBase) return new PhysicalAddress("ROM", (int)(physical - CartDomain1Address2));
            return new PhysicalAddress("PIF ROM", (int)(physical - PifRomBase), false);
        }

        // The VI's registers from the engine's VI space, none where it publishes no such space.
        public static IReadOnlyList<DebugRegisterValue> VideoRegisters(IReadOnlyList<IDebugMemorySpace> spaces) =>
            Ask("video_registers", spaces) is { } a
                ? a.Root.GetProperty("registers").EnumerateArray().Select(r => new DebugRegisterValue(r.GetProperty("name").GetString()!, r.GetProperty("value").GetUInt32(), 32)).ToArray()
                : VideoRegistersManaged(spaces);

        public static IReadOnlyList<DebugRegisterValue> VideoRegistersManaged(IReadOnlyList<IDebugMemorySpace> spaces)
        {
            if (SystemDebugSpaces.Find(spaces, Vi) is not { } vi) return Array.Empty<DebugRegisterValue>();
            int n = Math.Min(Math.Max(0, vi.Size) / 4, VideoRegisterNames.Count);
            var values = new DebugRegisterValue[n];
            for (int i = 0; i < n; i++)
            {
                uint v = 0;
                for (int b = 0; b < 4; b++) v = (v << 8) | vi.Read(4 * i + b);
                values[i] = new DebugRegisterValue(VideoRegisterNames[i], v, 32);
            }
            return values;
        }

        // The summary's console lines: the VR4300's PC, SP and RA, and the VI's origin, width and sync where the VI space is published.
        public static string? Summary(IReadOnlyList<DebugRegisterValue> registers, IReadOnlyList<IDebugMemorySpace> spaces) =>
            registers.All(r => SystemCode.Takes(r.Name)) && Ask("summary", spaces, w => WriteRegisters(w, registers)) is { } a
                ? a.Root.GetProperty("text").GetString()
                : SummaryManaged(registers, spaces);

        public static string? SummaryManaged(IReadOnlyList<DebugRegisterValue> registers, IReadOnlyList<IDebugMemorySpace> spaces)
        {
            ulong? R(string name) => registers.FirstOrDefault(r => string.Equals(r.Name, name, StringComparison.OrdinalIgnoreCase)) is { Name: not null } r ? r.Value : null;
            if (R("PC") is not { } pc || R("sp") is not { } sp || R("ra") is not { } ra) return null;
            string text = $"CPU  PC={pc:X16} SP={sp:X16} RA={ra:X16}";
            IReadOnlyList<DebugRegisterValue> vi = VideoRegistersManaged(spaces);
            return vi.Count >= 7 ? $"{text}\nVI   ORIGIN={vi[1].Value:X8} WIDTH={vi[2].Value} V_SYNC={vi[6].Value}" : text;
        }

        private static void WriteRegisters(Utf8JsonWriter w, IReadOnlyList<DebugRegisterValue> registers)
        {
            w.WriteStartArray("registers");
            foreach (DebugRegisterValue r in registers)
            {
                w.WriteStartObject();
                w.WriteString("name", r.Name);
                w.WriteNumber("value", r.Value);
                w.WriteEndObject();
            }
            w.WriteEndArray();
        }
    }
}
