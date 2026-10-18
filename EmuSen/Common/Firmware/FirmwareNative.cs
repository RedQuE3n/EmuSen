using System;
using System.Collections.Generic;
using System.Linq;
using System.Text.Json;
using EmuSen.Crystal.Services;
using EmuSen.Galaxia.Models;

namespace EmuSen.Common.Firmware
{
    // FirmwareLibrary's calls into emusen-firmware, under the file services' switch; a text that cannot cross stays on the C# - see EmuSen_Backend.md §15.1.
    public static class FirmwareNative
    {
        public static bool Active => ServiceLibrary.Active;

        private static bool Takes(FirmwareRequest r) => ServiceLibrary.Takes(r.FileName) && ServiceLibrary.TakesAll(r.AlternateNames) && r.Parts.All(ServiceLibrary.TakesAll);

        private static void Write(Utf8JsonWriter w, FirmwareRequest r)
        {
            w.WriteStartObject("request");
            w.WriteString("file", r.FileName);
            w.WriteNumber("size", r.Size);
            ServiceLibrary.WriteList(w, "alternates", r.AlternateNames);
            w.WriteStartArray("parts");
            foreach (var form in r.Parts) { w.WriteStartArray(); foreach (string name in form) w.WriteStringValue(name); w.WriteEndArray(); }
            w.WriteEndArray();
            w.WriteEndObject();
        }

        // The folders' order from the library, or null for a value the C# must read.
        public static IReadOnlyList<string>? Folders(string? setting, string home, string store)
        {
            if (!ServiceLibrary.Takes(setting) || !ServiceLibrary.Takes(home) || !ServiceLibrary.Takes(store)) return null;
            byte[] request = ServiceLibrary.Request(w => { ServiceLibrary.WriteText(w, "setting", setting); w.WriteString("home", home); w.WriteString("store", store); });
            using JsonDocument d = ServiceLibrary.CallJson("emusen_firmware_folders", request, 512);
            return ServiceLibrary.Words(d.RootElement, "folders");
        }

        public static bool Takes(FirmwareRequest r, IReadOnlyList<string> folders) => Takes(r) && ServiceLibrary.TakesAll(folders);

        public static FirmwareFound? Find(FirmwareRequest r, IReadOnlyList<string> folders)
        {
            byte[] request = ServiceLibrary.Request(w => { Write(w, r); ServiceLibrary.WriteList(w, "folders", folders); });
            byte[] envelope;
            try { envelope = ServiceLibrary.Call("emusen_firmware_find", request, Math.Max(0, r.Size) + 4096); }
            catch (ServiceFault fault) when (fault.Status == ServiceFault.Absent) { return null; }
            int length = BitConverter.ToInt32(envelope, 0);
            using JsonDocument d = JsonDocument.Parse(envelope.AsMemory(4, length));
            int at = 4 + length;
            var parts = new List<byte[]>();
            foreach (JsonElement n in d.RootElement.GetProperty("lengths").EnumerateArray())
            {
                int size = n.GetInt32();
                parts.Add(envelope[at..(at + size)]);
                at += size;
            }
            return new FirmwareFound(d.RootElement.GetProperty("folder").GetString()!, ServiceLibrary.Words(d.RootElement, "names"), parts);
        }

        public static (FirmwareLocated? Found, string? WrongSizeIn) Locate(FirmwareRequest r, IReadOnlyList<string> folders)
        {
            byte[] request = ServiceLibrary.Request(w => { Write(w, r); ServiceLibrary.WriteList(w, "folders", folders); });
            using JsonDocument d = ServiceLibrary.CallJson("emusen_firmware_locate", request, 512);
            JsonElement found = d.RootElement.GetProperty("found");
            FirmwareLocated? located = found.ValueKind == JsonValueKind.Null ? null
                : new FirmwareLocated(found.GetProperty("folder").GetString()!, ServiceLibrary.Words(found, "names"), found.GetProperty("whole").GetBoolean());
            return (located, ServiceLibrary.Text(d.RootElement, "wrong"));
        }

        // True when installed, false for a wrong size; the words of a write that failed come back as a fault.
        public static bool Install(FirmwareRequest r, string source, string store)
        {
            byte[] request = ServiceLibrary.Request(w => { Write(w, r); w.WriteString("source", source); w.WriteString("store", store); });
            return ServiceLibrary.CallStatus("emusen_firmware_install", request) == 1;
        }
    }
}
