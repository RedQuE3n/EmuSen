using System;
using System.Collections.Generic;
using System.Globalization;
using System.IO;
using System.Linq;
using System.Text;
using System.Text.Json;
using EmuSen.Crystal.Client;
using EmuSen.Galaxia.Input;

namespace EmuSen.Pharaoh.Reference
{
    // EmuSen driving itself through the reference probe's own file protocol, over a session of the frontend API - see §3.48.
    public static class PeerProbe
    {
        public const string Prefix = "emusen";

        // The probe's names for the NES's spaces, not this project's, so a consumer cannot tell which emulator wrote a dump - see §3.48.
        private static readonly (string Name, string Space)[] NesSpaces = { ("ram", "RAM"), ("work", "PRGRAM"), ("nametable", "CIRAM"), ("oam", "OAM"), ("palette", "PALETTE"), ("chr", "CHR") };

        // What the route says of the file: its system, and the NES's header as its pack reads it.
        private sealed record Identity(string System, string Board, string HeaderTrust, long Prg, long Chr, bool Save);

        private static Identity IdentityOf(CrystalRuntime runtime, string romPath)
        {
            JsonElement route = JsonDocument.Parse(runtime.Route(romPath)).RootElement;
            string system = route.GetProperty("system").GetString() == "nes" ? "nes" : "snes";
            if (route.GetProperty("cartridge") is not { ValueKind: JsonValueKind.Object } c) return new Identity(system, "", "", 0, 0, false);
            return new Identity(system, c.GetProperty("board").GetInt32().ToString(CultureInfo.InvariantCulture), c.GetProperty("header_trust").GetString() ?? "",
                c.GetProperty("prg_bytes").GetInt64(), c.GetProperty("chr_bytes").GetInt64(), c.GetProperty("battery").GetBoolean());
        }

        private static IEnumerable<(string Name, byte[] Data)> Spaces(CrystalSession session, Identity id)
        {
            if (id.System != "nes") yield break;
            Dictionary<string, int> sizes = JsonDocument.Parse(session.Spaces()).RootElement.EnumerateArray()
                .ToDictionary(s => s.GetProperty("name").GetString()!, s => s.GetProperty("size").GetInt32());
            foreach (var (name, space) in NesSpaces)
            {
                var data = new byte[sizes.GetValueOrDefault(space)];
                if (data.Length == 0) continue;
                session.ReadSpace(space, 0, data);
                yield return (name, data);
            }
        }

        private static (byte[] Rgba, int Width, int Height) Screen(CrystalSession session)
        {
            JsonElement shape = JsonDocument.Parse(session.Info()).RootElement.GetProperty("picture");
            var rgba = new byte[session.PictureBytes];
            session.CopyPicture(rgba);
            return (rgba, shape.GetProperty("width").GetInt32(), shape.GetProperty("height").GetInt32());
        }

        public static int Run(CrystalRuntime runtime, CrystalSession session, string romPath, string dumpDir, long startFrame, long endFrame,
            long stride, bool wantSignature, IReadOnlyList<(long Frame, long End, PadButton Button)> taps, Action<string> emit)
        {
            Directory.CreateDirectory(dumpDir);

            Identity id = IdentityOf(runtime, romPath);
            emit($"[INFO] backend {Prefix}, system {id.System}");
            emit($"[INFO] identity board={(id.Board.Length == 0 ? "?" : id.Board)} region=ntsc " +
                 $"headerTrust={(id.HeaderTrust.Length == 0 ? "?" : id.HeaderTrust)} " +
                 $"prg={id.Prg} chr={id.Chr} save={(id.Save ? 1 : 0)}");

            using SignatureWriter? signature = wantSignature
                ? new SignatureWriter(Path.Combine(dumpDir, $"{Prefix}_sig.csv"))
                : null;

            if (signature is not null)
            {
                signature.WriteHeader(Prefix, id.System, romPath, id.Board, "ntsc", id.HeaderTrust, id.Prg, id.Chr, id.Save, "Rgba8888", Spaces(session, id));
                signature.WriteRow(0, Spaces(session, id), Screen(session).Rgba);
                emit($"[INFO] signature stream -> {Path.Combine(dumpDir, $"{Prefix}_sig.csv")}");
            }

            var info = new Crystal.Client.Contract.CrystalFrameInfo();
            long nextReport = startFrame;
            for (long frame = 0; frame <= endFrame; frame++)
            {
                foreach ((long start, long end, PadButton button) in taps)
                {
                    session.Button(1, button, frame >= start && frame < end);
                }

                session.Frame(ref info);
                signature?.WriteRow(frame + 1, Spaces(session, id), Screen(session).Rgba);

                if (frame + 1 < nextReport) continue;

                long reported = frame + 1;
                WriteReport(session, id, romPath, dumpDir, reported);
                emit($"frame {reported,5} | dumped");
                nextReport = reported + Math.Max(1, stride);
            }

            return 0;
        }

        private static void WriteReport(CrystalSession session, Identity id, string romPath, string dumpDir, long frame)
        {
            var manifest = new StringBuilder();
            manifest.Append("{\n");
            manifest.Append($"  \"backend\": \"{Prefix}\",\n");
            manifest.Append($"  \"system\": \"{id.System}\",\n");
            manifest.Append($"  \"rom\": \"{Escape(romPath)}\",\n");
            manifest.Append($"  \"frame\": {frame},\n");
            manifest.Append($"  \"identity\": {{ \"board\": \"{id.Board}\", \"region\": \"ntsc\", " +
                            $"\"headerTrust\": \"{id.HeaderTrust}\", \"prg\": {id.Prg}, " +
                            $"\"chr\": {id.Chr}, \"saveLoaded\": {(id.Save ? "true" : "false")} }},\n");
            manifest.Append("  \"spaces\": [");

            bool first = true;
            foreach ((string name, byte[] data) in Spaces(session, id))
            {
                if (data.Length == 0) continue;
                string file = $"{Prefix}_{name}_f{frame:D5}.bin";
                File.WriteAllBytes(Path.Combine(dumpDir, file), data);
                manifest.Append(first ? "\n" : ",\n");
                first = false;
                manifest.Append($"    {{ \"name\": \"{name}\", \"size\": {data.Length}, \"file\": \"{file}\" }}");
            }

            manifest.Append(first ? "" : "\n  ").Append("],\n");

            var (screen, width, height) = Screen(session);
            string screenFile = $"{Prefix}_screen_f{frame:D5}.bin";
            File.WriteAllBytes(Path.Combine(dumpDir, screenFile), screen);
            manifest.Append($"  \"screen\": {{ \"width\": {width}, \"height\": {height}, " +
                            $"\"bytes\": {screen.Length}, \"format\": \"Rgba8888\", \"file\": \"{screenFile}\" }}\n");
            manifest.Append("}\n");

            File.WriteAllText(Path.Combine(dumpDir, $"{Prefix}_manifest_f{frame:D5}.json"), manifest.ToString());
        }

        private static string Escape(string text) => text.Replace("\\", "\\\\").Replace("\"", "\\\"");
    }
}
