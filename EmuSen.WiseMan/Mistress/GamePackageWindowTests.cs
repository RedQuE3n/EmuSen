using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Reflection;
using System.Security.Cryptography;
using System.Threading;
using System.Threading.Tasks;
using Avalonia.Controls;
using Avalonia.Headless;
using Avalonia.Interactivity;
using Avalonia.Threading;
using Avalonia.VisualTree;
using EmuSen.Galaxia;
using EmuSen.Galaxia.Library;
using EmuSen.Galaxia.Media;
using EmuSen.Galaxia.Models;
using EmuSen.Galaxia.Native;
using EmuSen.Mistress.Views;
using EmuSen.WiseMan.Crystal;
using EmuSen.WiseMan.Fixtures;
using EmuSen.WiseMan.Mistress.BigPicture;
using Xunit.Abstractions;

namespace EmuSen.WiseMan.Mistress
{
    // The Game Package window through real jobs of the library tool: Make Package, a patch beside the game added and taken out again, a patch for another image refused in words, and the library unchanged; and the window reached from the game options sheet by the pad - see EmuSen_GamePackages.md §15.
    [Collection(TestCollections.ProcessGlobals)]
    public class GamePackageWindowTests : IDisposable
    {
        private static readonly HeadlessUnitTestSession Session =
            HeadlessUnitTestSession.GetOrStartForAssembly(typeof(GamePackageWindowTests).GetTypeInfo().Assembly);

        private readonly ITestOutputHelper _out;
        private readonly string _root = Path.Combine(Environment.GetEnvironmentVariable("TMPDIR") is { Length: > 0 } t ? t : Path.GetTempPath(), "EmuSenGamePackage_" + Guid.NewGuid().ToString("N"));
        private readonly string _library;

        public GamePackageWindowTests(ITestOutputHelper output)
        {
            _out = output;
            _library = Path.Combine(_root, "Roms");
            Directory.CreateDirectory(_library);
            ConfigStore.OverrideDirectory = Path.Combine(_root, "Config");
            DataStore.OverrideDirectory = Path.Combine(_root, "Home");
        }

        public void Dispose()
        {
            ConfigStore.OverrideDirectory = null;
            DataStore.OverrideDirectory = null;
            try { Directory.Delete(_root, recursive: true); } catch (IOException) { }
        }

        private List<string> Listing() =>
            Directory.EnumerateFiles(_library, "*", SearchOption.AllDirectories).Order(StringComparer.Ordinal)
                .Select(f => $"{f}\t{new FileInfo(f).Length}\t{File.GetLastWriteTimeUtc(f).Ticks}\t{Convert.ToHexString(MD5.HashData(File.ReadAllBytes(f)))}").ToList();

        private static void WaitIdle(GamePackageWindow window)
        {
            var clock = System.Diagnostics.Stopwatch.StartNew();
            while (window.Busy)
            {
                window.Poll();
                Dispatcher.UIThread.RunJobs();
                if (clock.Elapsed.TotalSeconds > 60) Assert.Fail("the window's job did not end in time: " + window.Status);
                Thread.Sleep(5);
            }
            Dispatcher.UIThread.RunJobs();
        }

        private static void Press(GamePackageWindow window, string name)
        {
            Button button = window.GetVisualDescendants().OfType<Button>().First(c => c.Name == name);
            Assert.True(button.IsEnabled, $"{name} is not enabled");
            button.RaiseEvent(new RoutedEventArgs(Button.ClickEvent));
            Dispatcher.UIThread.RunJobs();
            WaitIdle(window);
        }

        [Fact]
        public Task A_package_is_made_and_a_patch_beside_the_game_added_and_taken_out_with_the_library_unchanged() => Session.Dispatch(() =>
        {
            if (!LibraryToolJob.Available) { _out.WriteLine("skipped: " + LibraryToolJob.Report); return; }
            string game = Path.Combine(_library, "snes", "Cobalt Harbor (Synthetic).sfc");
            Directory.CreateDirectory(Path.GetDirectoryName(game)!);
            byte[] rom = SyntheticRom.BuildBlank();
            File.WriteAllBytes(game, rom);
            byte[] hacked = rom.ToArray();
            hacked[0x100] = 0x5A;
            File.WriteAllBytes(Path.Combine(_library, "snes", "Bright Title.ips"), CrystalParityTests.Ips((0x100, new byte[] { 0x5A })));
            new AppSettings { RomDirectory = _library }.Save();
            List<string> before = Listing();
            var window = new GamePackageWindow("Cobalt Harbor (Synthetic)", game, _library, null, browse: false) { Width = 1280, Height = 800 };
            window.Show();
            try
            {
                Assert.Null(window.Package);
                Press(window, "GamePackageMake");
                Assert.True(window.Package is { } p && Directory.Exists(p), window.Status);
                Assert.Equal("The package was made, linked to the game where it is.", window.Status);
                Press(window, "GamePackageAdd.0");
                Assert.StartsWith("Added Cobalt Harbor (Synthetic).ips. It is named as RetroArch finds it.", window.Status);
                Assert.Equal(new[] { ("Cobalt Harbor (Synthetic).ips", "IPS") }, window.Patches());
                Assert.Equal(RomHash.Of(hacked).Md5, GamePackage.Read(Path.Combine(window.Package!, GamePackage.FileName)).PatchedIdentity?.Md5);
                Press(window, "GamePackagePatch.0.Remove");
                Assert.Equal("Took out Cobalt Harbor (Synthetic).ips.", window.Status);
                Assert.Empty(window.Patches());
                Assert.True(window.Changed);
                Assert.Equal(before, Listing());
            }
            finally { window.Close(); }
        }, default);

        // Decided 2026-10-25 (§14.8 Q34): a claimed file that is another dump now resolves as a plain game, and the window names its package changed.
        [Fact]
        public Task A_changed_game_resolves_as_a_plain_one_and_its_package_is_shown_changed() => Session.Dispatch(() =>
        {
            if (!LibraryToolJob.Available) { _out.WriteLine("skipped: " + LibraryToolJob.Report); return; }
            string game = Path.Combine(_library, "snes", "Opal Reach (Synthetic).sfc");
            Directory.CreateDirectory(Path.GetDirectoryName(game)!);
            byte[] rom = SyntheticRom.BuildBlank();
            File.WriteAllBytes(game, rom);
            new AppSettings { RomDirectory = _library }.Save();
            var made = new GamePackageWindow("Opal Reach (Synthetic)", game, _library, null, browse: false) { Width = 1280, Height = 800 };
            made.Show();
            string package;
            try
            {
                Press(made, "GamePackageMake");
                package = made.Package!;
                Assert.Equal(package, GamePackages.RootFor(game));
            }
            finally { made.Close(); }
            rom[0x200] ^= 0xFF;
            File.WriteAllBytes(game, rom);
            File.SetLastWriteTimeUtc(game, new DateTime(2031, 1, 1, 0, 0, 0, DateTimeKind.Utc));
            Assert.Null(GamePackages.RootFor(game));
            Assert.Equal(package, GamePackages.Index(PackagePlaces.Current()).Claimant(Path.GetFullPath(game))?.Folder);
            Assert.Throws<IOException>(() => GamePackages.Resolve(package, PackagePlaces.Current()));
            var window = new GamePackageWindow("Opal Reach (Synthetic)", game, _library, package, browse: false, refused: true) { Width = 1280, Height = 800 };
            window.Show();
            try
            {
                string shown = window.GetVisualDescendants().OfType<TextBlock>().First(t => t.Name == "GamePackagePlace").Text ?? "";
                Assert.Contains(", changed: the game's file is not as the package recorded it", shown);
            }
            finally { window.Close(); }
        }, default);

        [Fact]
        public Task A_patch_made_for_another_image_is_refused_in_words_and_nothing_is_added() => Session.Dispatch(() =>
        {
            if (!LibraryToolJob.Available) { _out.WriteLine("skipped: " + LibraryToolJob.Report); return; }
            string game = Path.Combine(_library, "snes", "Amber Saga (Synthetic).sfc");
            Directory.CreateDirectory(Path.GetDirectoryName(game)!);
            File.WriteAllBytes(game, SyntheticRom.BuildBlank());
            new AppSettings { RomDirectory = _library }.Save();
            var window = new GamePackageWindow("Amber Saga (Synthetic)", game, _library, null, browse: false) { Width = 1280, Height = 800 };
            window.Show();
            try
            {
                Press(window, "GamePackageMake");
                string patch = Path.Combine(_root, "not-a-patch.bps");
                File.WriteAllBytes(patch, "BPS1 is all this file holds"u8.ToArray());
                window.Add(patch);
                WaitIdle(window);
                Assert.StartsWith("Not done: not-a-patch.bps: the patch file is damaged", window.Status);
                Assert.Empty(window.Patches());
            }
            finally { window.Close(); }
        }, default);
    }
}
