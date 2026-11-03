using System;
using System.Collections.Generic;
using System.Linq;
using EmuSen.Galaxia.Models;
using EmuSen.Mistress.Library;
using EmuSen.Mistress.Pages;

namespace EmuSen.WiseMan.Fixtures
{
    // A host for Settings' one plain description where a main window cannot give what a test needs, such as a running hosted core's own rows - see EmuSen_BigPicture.md §52.6.
    public sealed class SettingsHostFake : ISettingsHost
    {
        public Func<string, IReadOnlyList<SettingRow>> Rows { get; init; } = RuntimeCatalog.SettingsFor;
        public Dictionary<(string, string), string?> Values { get; } = new();
        public List<string> Doors { get; } = new();

        public AppSettings App { get; init; } = new();
        public GraphicsConfig Graphics { get; init; } = new();
        public bool Desktop { get; init; }
        public string Version => "test";
        public string? RunningTitle { get; init; }
        public string? RunningConsole { get; init; }
        public string? RunningCropNote(string console) => null;
        public ShaderShelf Shaders { get; init; } = new(System.IO.Path.Combine(System.IO.Path.GetTempPath(), "EmuSenNoPack", Guid.NewGuid().ToString("N")));
        public (bool Running, int Waiting) ScrapeRun => (false, 0);
        public void DownloadShaders() { }
        public void Ask(string title, string prompt, string current, Action<string> then) { }
        public IReadOnlyList<SystemFacts> Systems { get; init; } = RuntimeCatalog.Systems;
        public IReadOnlyList<SettingRow> Schema(string console) => Rows(console);
        public string? CoreValue(string console, string key) => Values.GetValueOrDefault((console, key));
        public void SetCoreValue(string console, string key, string? value) => Values[(console, key)] = value;
        public IReadOnlyList<FirmwareFile>? Firmware(string system) => [];
        public IReadOnlyDictionary<string, FirmwareWords> FirmwareOverview() => new Dictionary<string, FirmwareWords>();
        public IReadOnlyList<EmuSen.Mistress.Views.CardMedium> Cards(string console) => [];
        public string NewCard(string console) => "";
        public int Games(string console) => 0;
        public int AllGames => 0;
        public int Hidden => 0;
        public IReadOnlyList<EmuSen.Mistress.Library.GameCollection> CustomCollections => [];
        public IReadOnlyList<PlayerSeat> Players => [];
        public IReadOnlyList<string> Pads => [];
        public void Seat(int pad, int player) { }
        public void ForgetSeat(int player) { }
        public int PlayerOf(int pad) => 0;
        public bool PadPressed(int pad) => false;
        public IReadOnlyList<string> Looks => ["EmuSen (built in)"];
        public int LookAt => 0;
        public void ChooseLook(int at) { }
        public bool Logging { get; set; }
        public string? ScrapeUser => null;
        public string AccountLine => "";
        public long? OpenVgdbBytes => null;
        public string? EditingCollection => null;
        public void SaveApp(string? what = null) { }
        public void SaveGraphics(string? console) { }
        public void Open(string door) => Doors.Add(door);
        public EmuSen.Endymion.Input.GamepadBindings PadBindings { get; } = new(RuntimeCatalog.Systems.Select(s => s.Console).Distinct());
        public void SavePadBindings() { }
        public IReadOnlyList<string> SoundOutputs => ["System default"];
        public EmuSen.Mistress.Input.ControllerKeyBindings KeyBindings { get; } = new(RuntimeCatalog.Systems.Select(s => s.Console).Distinct());
        public EmuSen.Mistress.Input.HotkeyBindingMap Hotkeys { get; } = new();
        public void SaveKeys() { }
        public bool SystemReducesMotion => false;
        public PadFacts PadNow(string console) => RuntimeCatalog.Systems.First(s => s.Console == console).Pad;
    }
}
