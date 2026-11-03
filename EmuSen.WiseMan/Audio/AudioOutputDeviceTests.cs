using System;
using System.IO;
using System.Linq;
using EmuSen.Endymion;
using EmuSen.Endymion.Native;
using EmuSen.Galaxia.Library;
using EmuSen.Galaxia.Models;
using EmuSen.WiseMan.Fixtures;
using SDL3;

namespace EmuSen.WiseMan.Audio
{
    // The library's playback devices and the player's choice of one against SDL's dummy driver, and the fallback a missing one takes - see EmuSen_Audio_Sync.md §7.4.
    [Collection(TestCollections.ProcessGlobals)]
    public sealed class AudioOutputDeviceTests : IDisposable
    {
        private readonly string _logs = Path.Combine(Path.GetTempPath(), "EmuSenAudioOutput_" + Guid.NewGuid().ToString("N"));
        private readonly string? _logsWere = ErrorLog.DirectoryOverride;

        public AudioOutputDeviceTests()
        {
            SDL.SetHint(SDL.Hints.AudioDriver, "dummy");
            ErrorLog.DirectoryOverride = _logs;
            ErrorLog.ResetForTests();
        }

        public void Dispose()
        {
            ErrorLog.DirectoryOverride = _logsWere;
            ErrorLog.ResetForTests();
            try { Directory.Delete(_logs, recursive: true); } catch (IOException) { }
        }

        private static AudioPlayer Library(string? device = null)
        {
            Assert.True(DeviceNative.Ready, DeviceNative.Report);
            var player = new AudioPlayer(32000, 256, 0.005, 4096, native: true);
            if (device is not null) player.SetOutputDevice(device);
            return player;
        }

        private string[] Logged() => Directory.Exists(_logs)
            ? Directory.GetFiles(_logs, "*.log", SearchOption.AllDirectories).SelectMany(File.ReadAllLines).Where(l => l.Contains("sound output")).ToArray()
            : Array.Empty<string>();

        [Fact]
        public void The_list_is_the_system_default_then_each_playback_device_by_name()
        {
            using AudioPlayer player = Library();
            var devices = AudioPlayer.OutputDevices();
            Assert.Equal(AppSettings.SystemDefaultOutput, devices[0]);
            Assert.True(devices.Count >= 2, "the dummy driver has a device of its own");
            Assert.Equal(devices.Count, devices.Distinct().Count());
            Assert.Equal(AppSettings.SystemDefaultOutput, new AppSettings().SoundOutputDevice);
        }

        [Fact]
        public void A_listed_device_is_played_on_and_the_system_default_chosen_again()
        {
            using AudioPlayer player = Library();
            string named = AudioPlayer.OutputDevices()[1];
            Assert.True(player.SetOutputDevice(named));
            Assert.Equal(named, player.OutputDevice);
            Assert.True(player.IsAvailable);
            Assert.Equal(32000, player.SampleRate);
            Assert.True(player.SetOutputDevice(AppSettings.SystemDefaultOutput));
            Assert.Equal(AppSettings.SystemDefaultOutput, player.OutputDevice);
            Assert.True(player.IsAvailable);
            Assert.Empty(Logged());
        }

        [Fact]
        public void A_saved_device_missing_at_start_falls_back_to_the_system_default_with_one_log_line()
        {
            using AudioPlayer player = Library("Headphones that are not plugged in");
            Assert.Equal(AppSettings.SystemDefaultOutput, player.OutputDevice);
            Assert.True(player.IsAvailable);
            player.Submit(new short[64], 44100);
            Assert.Equal(44100, player.SampleRate);
            string line = Assert.Single(Logged());
            Assert.Contains("\"Headphones that are not plugged in\" is not connected; the system default plays instead.", line);
        }
    }
}
