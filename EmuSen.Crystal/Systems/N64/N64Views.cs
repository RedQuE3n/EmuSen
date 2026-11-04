using System;
using System.Collections.Generic;
using System.Linq;
using System.Text.Json;
using EmuSen.Crystal.Debugger;
using EmuSen.Crystal.Telemetry;
using EmuSen.Galaxia.Native;

namespace EmuSen.Crystal.Systems.N64
{
    // The N64's debugger views, emusen-systems' n64 module through the platform library - see EmuSen_Backend.md §18.7, §18.10, §40.1.
    public static class N64Views
    {
        private static readonly IReadOnlyList<IDebugMemorySpace> NoSpaces = Array.Empty<IDebugMemorySpace>();

        private static NativeDebugViews.Answer Ask(string view, IReadOnlyList<IDebugMemorySpace> spaces, Action<Utf8JsonWriter>? fields = null) =>
            NativeDebugViews.Request(N64System.Id, view, spaces, fields) ?? throw new InvalidOperationException($"{PlatformLibrary.FileName} has no N64 view {view}");

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
            Instruction(Ask("instruction", NoSpaces, w => { w.WriteString("space", PlatformText.Mend(space)); w.WriteNumber("address", unchecked((int)at)); w.WriteNumber("word", word); }).Root);

        // Which memory a physical address names given the RDRAM installed, or the register block it falls in, not addressable - see Mars_Memory.md §2.
        public static PhysicalAddress Physical(uint physical, long rdramLength)
        {
            var rdram = new IDebugMemorySpace[] { new DelegateDebugMemorySpace("RDRAM", (int)Math.Min(int.MaxValue, rdramLength), _ => 0) };
            NativeDebugViews.Answer a = Ask("physical", rdram, w => w.WriteNumber("address", unchecked((int)physical)));
            return new PhysicalAddress(a.Root.GetProperty("space").GetString()!, a.Root.GetProperty("offset").GetInt32(), a.Root.GetProperty("addressable").GetBoolean());
        }

        // Count words of a space from address rounded down to a word, each read big-endian and decoded where it lies.
        public static IReadOnlyList<(DisassembledInstruction Instruction, MarsInstruction Decoded)> Disassemble(IDebugMemorySpace space, int address, int count)
        {
            var listing = new List<(DisassembledInstruction, MarsInstruction)>(Math.Max(count, 0));
            for (int i = 0, at = address & ~3; i < count; i++, at += 4)
            {
                var bytes = new byte[4];
                for (int b = 0; b < 4; b++) bytes[b] = space.Read(at + b);
                MarsInstruction decoded = DecodeIn(space.Name, (uint)((bytes[0] << 24) | (bytes[1] << 16) | (bytes[2] << 8) | bytes[3]), (uint)at);
                listing.Add((new DisassembledInstruction(at, bytes, decoded.Mnemonic, decoded.Operands), decoded));
            }
            return listing;
        }

        // The summary's console lines: the VR4300's PC, SP and RA, and the VI's origin, width and sync from the engine's video unit named VI.
        public static string? Summary(IReadOnlyList<DebugRegisterValue> registers, IReadOnlyList<DebugRegisterValue>? vi) =>
            Ask("summary", NoSpaces, w =>
            {
                WriteRegisters(w, "registers", registers);
                if (vi is not null) WriteRegisters(w, "vi", vi);
            }).Root.GetProperty("text").GetString();

        private static void WriteRegisters(Utf8JsonWriter w, string key, IReadOnlyList<DebugRegisterValue> registers)
        {
            w.WriteStartArray(key);
            foreach (DebugRegisterValue r in registers)
            {
                w.WriteStartObject();
                w.WriteString("name", PlatformText.Mend(r.Name));
                w.WriteNumber("value", r.Value);
                w.WriteEndObject();
            }
            w.WriteEndArray();
        }
    }
}
