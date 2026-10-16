using System;
using System.Collections.Generic;
using System.Globalization;
using System.IO;
using System.Text;
using EmuSen.Cores;
using EmuSen.Cores.Nintendo.MoonRT;
using EmuSen.Crystal.Systems.Nes;
using EmuSen.Galaxia.Input;

namespace EmuSen.Pharaoh.Reference
{
    // EmuSen driving itself through the reference probe's own file protocol - see §3.48.
    public static class PeerProbe
    {
        public const string Prefix = "emusen";

        // The probe's names, not this project's, because the whole point is that a
        // consumer cannot tell which emulator produced a dump set - see §3.48.
        private static IEnumerable<(string Name, byte[] Data)> Spaces(ICore core)
        {
            if (core is MoonRtCore moon && moon.IsRomLoaded)
            {
                foreach (var (name, space) in new[] { ("ram", "RAM"), ("work", "PRGRAM"), ("nametable", "CIRAM"), ("oam", "OAM"), ("palette", "PALETTE"), ("chr", "CHR") })
                {
                    var data = new byte[moon.SpaceSize(space)];
                    for (int i = 0; i < data.Length; i++) data[i] = moon.ReadSpace(space, i);
                    if (data.Length > 0) yield return (name, data);
                }
            }
        }

        private static string Identity(ICore core, out string headerTrust)
        {
            headerTrust = "";
            if (core is MoonRtCore { Header: { } header })
            {
                headerTrust = header.Trust switch
                {
                    NesHeaderTrust.Archaic => "archaic",
                    NesHeaderTrust.Unverifiable => "unverifiable",
                    _ => "clean",
                };
                return header.Mapper.ToString(CultureInfo.InvariantCulture);
            }
            return "";
        }

        private static long PrgBytes(ICore core) => core is MoonRtCore { Header: { } h } ? (long)h.PrgBanks * NesHeader.PrgBankSize : 0;
        private static long ChrBytes(ICore core) => core is MoonRtCore { Header: { } h } ? (h.ChrIsRam ? NesHeader.ChrBankSize : (long)h.ChrBanks * NesHeader.ChrBankSize) : 0;
        private static bool SaveLoaded(ICore core) => core is MoonRtCore { Header: { } h } && h.HasBattery;

        public static int Run(ICore core, string romPath, string dumpDir, long startFrame, long endFrame,
            long stride, bool wantSignature, IReadOnlyList<(long Frame, long End, PadButton Button)> taps,
            Action<string> emit)
        {
            Directory.CreateDirectory(dumpDir);

            string system = core is MoonRtCore ? "nes" : "snes";
            string board = Identity(core, out string headerTrust);
            emit($"[INFO] backend {Prefix}, system {system}");
            emit($"[INFO] identity board={(board.Length == 0 ? "?" : board)} region=ntsc " +
                 $"headerTrust={(headerTrust.Length == 0 ? "?" : headerTrust)} " +
                 $"prg={PrgBytes(core)} chr={ChrBytes(core)} save={(SaveLoaded(core) ? 1 : 0)}");

            using SignatureWriter? signature = wantSignature
                ? new SignatureWriter(Path.Combine(dumpDir, $"{Prefix}_sig.csv"))
                : null;

            if (signature is not null)
            {
                signature.WriteHeader(Prefix, system, romPath, board, "ntsc", headerTrust,
                    PrgBytes(core), ChrBytes(core), SaveLoaded(core), "Rgba8888", Spaces(core));
                signature.WriteRow(0, Spaces(core), core.GetFrameBufferRgba());
                emit($"[INFO] signature stream -> {Path.Combine(dumpDir, $"{Prefix}_sig.csv")}");
            }

            long nextReport = startFrame;
            for (long frame = 0; frame <= endFrame; frame++)
            {
                foreach ((long start, long end, PadButton button) in taps)
                {
                    core.SetButton(1, button, frame >= start && frame < end);
                }

                core.RunFrame();
                signature?.WriteRow(frame + 1, Spaces(core), core.GetFrameBufferRgba());

                if (frame + 1 < nextReport) continue;

                long reported = frame + 1;
                WriteReport(core, romPath, dumpDir, system, board, headerTrust, reported);
                emit($"frame {reported,5} | dumped");
                nextReport = reported + Math.Max(1, stride);
            }

            return 0;
        }

        private static void WriteReport(ICore core, string romPath, string dumpDir, string system,
            string board, string headerTrust, long frame)
        {
            var manifest = new StringBuilder();
            manifest.Append("{\n");
            manifest.Append($"  \"backend\": \"{Prefix}\",\n");
            manifest.Append($"  \"system\": \"{system}\",\n");
            manifest.Append($"  \"rom\": \"{Escape(romPath)}\",\n");
            manifest.Append($"  \"frame\": {frame},\n");
            manifest.Append($"  \"identity\": {{ \"board\": \"{board}\", \"region\": \"ntsc\", " +
                            $"\"headerTrust\": \"{headerTrust}\", \"prg\": {PrgBytes(core)}, " +
                            $"\"chr\": {ChrBytes(core)}, \"saveLoaded\": {(SaveLoaded(core) ? "true" : "false")} }},\n");
            manifest.Append("  \"spaces\": [");

            bool first = true;
            foreach ((string name, byte[] data) in Spaces(core))
            {
                if (data.Length == 0) continue;
                string file = $"{Prefix}_{name}_f{frame:D5}.bin";
                File.WriteAllBytes(Path.Combine(dumpDir, file), data);
                manifest.Append(first ? "\n" : ",\n");
                first = false;
                manifest.Append($"    {{ \"name\": \"{name}\", \"size\": {data.Length}, \"file\": \"{file}\" }}");
            }

            manifest.Append(first ? "" : "\n  ").Append("],\n");

            byte[] screen = core.GetFrameBufferRgba();
            string screenFile = $"{Prefix}_screen_f{frame:D5}.bin";
            File.WriteAllBytes(Path.Combine(dumpDir, screenFile), screen);
            manifest.Append($"  \"screen\": {{ \"width\": {core.ScreenWidth}, \"height\": {core.ScreenHeight}, " +
                            $"\"bytes\": {screen.Length}, \"format\": \"Rgba8888\", \"file\": \"{screenFile}\" }}\n");
            manifest.Append("}\n");

            File.WriteAllText(Path.Combine(dumpDir, $"{Prefix}_manifest_f{frame:D5}.json"), manifest.ToString());
        }

        private static string Escape(string text) => text.Replace("\\", "\\\\").Replace("\"", "\\\"");
    }
}
