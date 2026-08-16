using System;
using System.IO;
using System.Reflection;
using System.Threading.Tasks;
using Avalonia.Controls;
using Avalonia.Headless;
using Avalonia.Input;
using EmuSen.Common;
using EmuSen.Galaxia;
using EmuSen.Galaxia.Models;
using EmuSen.LunaP.Windowing;
using EmuSen.Mistress.Views;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Mistress
{
    // IdleCursor and FileDrop in both frontends - see EmuSen_Settings_Reference.md §4.20.
    // The risk worth a test is disposal: a cursor left hidden by an object nobody
    // unsubscribed is an application whose pointer is gone for good.
    [Collection(TestCollections.ProcessGlobals)]
    public class PointerAndDropTests : IDisposable
    {
        private static readonly HeadlessUnitTestSession Session =
            HeadlessUnitTestSession.GetOrStartForAssembly(typeof(PointerAndDropTests).GetTypeInfo().Assembly);

        private readonly string _root;

        public PointerAndDropTests()
        {
            _root = Path.Combine(Path.GetTempPath(), "EmuSenPointerTests", Guid.NewGuid().ToString("N"));
            string romDir = Path.Combine(_root, "Roms");
            Directory.CreateDirectory(romDir);
            ConfigStore.OverrideDirectory = Path.Combine(_root, "Config");
            new AppSettings { RomDirectory = romDir, LogDirectory = Path.Combine(_root, "Logs") }.Save();
        }

        public void Dispose()
        {
            ConfigStore.OverrideDirectory = null;
            try { if (Directory.Exists(_root)) Directory.Delete(_root, recursive: true); } catch { }
        }

        private static T Field<T>(object owner, string name) =>
            (T)owner.GetType().GetField(name, BindingFlags.Instance | BindingFlags.NonPublic)!.GetValue(owner)!;

        [Fact]
        public Task The_main_window_hides_the_pointer_over_the_frame_and_not_the_menu() => Session.Dispatch(() =>
        {
            var window = new MainWindow();
            window.Show();

            var cursor = Field<IdleCursor>(window, "_idleCursor");
            Assert.NotNull(cursor);

            // Attached to the frame, so the pointer stays over the menu bar - a
            // window-level switch could not say that.
            cursor.Hide();
            Assert.True(cursor.IsHidden);
            cursor.Show();
            Assert.False(cursor.IsHidden);

            window.Close();
        }, default);

        // The failure this exists for: close the window with the pointer hidden and,
        // without disposal, it never comes back.
        [Fact]
        public Task Closing_the_main_window_gives_the_pointer_back() => Session.Dispatch(() =>
        {
            var window = new MainWindow();
            window.Show();

            var cursor = Field<IdleCursor>(window, "_idleCursor");
            cursor.Hide();
            Assert.True(cursor.IsHidden);

            window.Close();

            Assert.False(cursor.IsHidden);
        }, default);

        // A drag carrying a whole folder of ROMs has no single answer, so it is refused
        // before the indicator promises anything.
        [Fact]
        public Task The_main_window_accepts_one_dropped_file_and_refuses_several() => Session.Dispatch(() =>
        {
            var window = new MainWindow();
            window.Show();

            var drop = Field<FileDrop>(window, "_fileDrop");
            Assert.NotNull(drop.Accept);
            Assert.True(drop.Accept!(new[] { "one.smc" }));
            Assert.False(drop.Accept!(new[] { "one.smc", "two.smc" }));

            window.Close();
        }, default);
    }
}
