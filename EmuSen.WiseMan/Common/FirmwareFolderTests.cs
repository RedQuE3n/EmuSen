using EmuSen.Common.Firmware;
using EmuSen.Cores;
using EmuSen.Galaxia;
using EmuSen.Galaxia.Library;
using EmuSen.Galaxia.Models;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Common
{
    // The player's firmware folder: the setting's rules, the order folders are searched in, and the folder only ever read - see EmuSen_Firmware.md §2.3 and EmuSen_Config_Reference.md §7.7.
    [Collection(TestCollections.ProcessGlobals)]
    public class FirmwareFolderTests : IDisposable
    {
        private readonly string _root = Path.Combine(Path.GetTempPath(), "EmuSenFirmwareFolder_" + Guid.NewGuid().ToString("N"));
        private readonly string? _config = ConfigStore.OverrideDirectory, _data = DataStore.OverrideDirectory;
        private readonly Action<string>? _sink = ConfigDiagnostics.Sink;
        private readonly List<string> _said = new();

        private static readonly FirmwareRequest Widget = new("TESTCORE", "WIDGET", "widget.rom", Size: 64, "a made-up chip's mask ROM");

        public FirmwareFolderTests()
        {
            ConfigStore.OverrideDirectory = Path.Combine(_root, "Config");
            DataStore.OverrideDirectory = Path.Combine(_root, "Home");
            FirmwareLibrary.ResetDirectory();
            Directory.CreateDirectory(DataStore.Firmware);
            Directory.CreateDirectory(Chosen);
            ConfigDiagnostics.Sink = _said.Add;
        }

        public void Dispose()
        {
            ConfigDiagnostics.Sink = _sink;
            ConfigStore.OverrideDirectory = _config;
            DataStore.OverrideDirectory = _data;
            try { Directory.Delete(_root, recursive: true); } catch (IOException) { }
        }

        private string Chosen => Path.Combine(_root, "BIOS");

        private static void Choose(string? value) => new AppSettings { FirmwareDirectory = value }.Save();

        // Made-up bytes of the right length, each file's own fill so the test can tell which was read.
        private static void Put(string folder, string name, int size, byte fill) => File.WriteAllBytes(Path.Combine(folder, name), Enumerable.Repeat(fill, size).ToArray());

        public static TheoryData<string?> Blanks => new() { null, "", "   ", "\t", " " };

        [Theory]
        [MemberData(nameof(Blanks))]
        public void A_blank_setting_is_unset_and_firmware_comes_from_home_firmware_alone(string? blank)
        {
            Choose(blank);
            Assert.Equal(FolderState.Unset, SettingFolders.Checked(blank).State);
            Assert.Equal([DataStore.Firmware], FirmwareLibrary.Folders);
            Put(DataStore.Firmware, "widget.rom", 64, 0xA1);
            Assert.Equal((DataStore.Firmware, (byte)0xA1), (FirmwareLibrary.Find(Widget)!.Folder, FirmwareLibrary.TryLoad(Widget)![0]));
            Assert.Empty(_said);
        }

        [Fact]
        public void A_file_in_the_chosen_folder_is_found_and_the_chosen_folder_is_searched_first()
        {
            Choose(Chosen);
            Assert.Equal([Chosen, DataStore.Firmware], FirmwareLibrary.Folders);
            Put(Chosen, "widget.rom", 64, 0xB2);
            FirmwareFound found = FirmwareLibrary.Find(Widget)!;
            Assert.Equal((Chosen, true, (byte)0xB2), (found.Folder, found.Whole, found.Parts[0][0]));
            Assert.True(FirmwareLibrary.IsInstalled(Widget));
            Assert.Empty(FirmwareLibrary.MissingFrom([Widget]));
        }

        // The same name and size in both folders: the chosen folder's file is the one read.
        [Fact]
        public void A_tie_of_name_and_size_is_broken_by_the_chosen_folder()
        {
            Choose(Chosen);
            Put(DataStore.Firmware, "widget.rom", 64, 0xA1);
            Put(Chosen, "widget.rom", 64, 0xB2);
            Assert.Equal((Chosen, (byte)0xB2), (FirmwareLibrary.Find(Widget)!.Folder, FirmwareLibrary.TryLoad(Widget)![0]));

            // A file of the wrong size is no tie: it is not the image, so home/Firmware's is read.
            Put(Chosen, "widget.rom", 63, 0xB2);
            Assert.Equal((DataStore.Firmware, (byte)0xA1), (FirmwareLibrary.Find(Widget)!.Folder, FirmwareLibrary.TryLoad(Widget)![0]));
        }

        [Fact]
        public void A_split_form_is_taken_from_one_folder_and_never_from_two()
        {
            Choose(Chosen);
            var split = Widget with { Parts = [["widget.program.rom", "widget.data.rom"]] };
            Put(Chosen, "widget.program.rom", 48, 1);
            Put(DataStore.Firmware, "widget.data.rom", 16, 2);
            Assert.Null(FirmwareLibrary.Find(split));
            Put(Chosen, "widget.data.rom", 16, 3);
            FirmwareFound found = FirmwareLibrary.Find(split)!;
            Assert.Equal((Chosen, false, (byte)1, (byte)3), (found.Folder, found.Whole, found.Parts[0][0], found.Parts[1][0]));
            Assert.Equal(2, FirmwareLibrary.TryLoadParts(split)!.Count);
        }

        [Theory]
        [InlineData("genesis")]
        [InlineData("BIOS/PS1")]
        [InlineData("./BIOS")]
        [InlineData("~someone/BIOS")]
        public void A_relative_path_is_refused_and_said_once(string value)
        {
            FolderValue checkedValue = SettingFolders.Checked(value);
            Assert.Equal((FolderState.Relative, null, false), (checkedValue.State, checkedValue.Path, checkedValue.Searched));
            Assert.Contains("is not a full path", checkedValue.Problem);
            Choose(value);
            Assert.Equal([DataStore.Firmware], FirmwareLibrary.Folders);
            Assert.Equal([DataStore.Firmware], FirmwareLibrary.Folders);
            Assert.Equal($"appsettings.json: FirmwareDirectory: {checkedValue.Problem}", Assert.Single(_said));
        }

        [Theory]
        [InlineData("~", "")]
        [InlineData("~/", "")]
        [InlineData("~/EmuSenNoSuchBios", "EmuSenNoSuchBios")]
        [InlineData("  ~/EmuSenNoSuchBios/PS1  ", "EmuSenNoSuchBios/PS1")]
        public void A_tilde_is_the_home(string value, string under)
        {
            string home = Environment.GetFolderPath(Environment.SpecialFolder.UserProfile);
            FolderValue checkedValue = SettingFolders.Checked(value);
            Assert.Equal(Path.GetFullPath(Path.Join(home, under)), checkedValue.Path);
            Assert.True(checkedValue.Searched);
            Assert.Equal(value.Trim(), checkedValue.Written);
        }

        [Fact]
        public void A_file_is_refused()
        {
            string file = Path.Combine(_root, "bios.zip");
            File.WriteAllBytes(file, [1, 2, 3]);
            FolderValue checkedValue = SettingFolders.Checked(file);
            Assert.Equal((FolderState.File, null), (checkedValue.State, checkedValue.Path));
            Assert.Equal($"\"{file}\" is a file, not a folder, so it is not used.", checkedValue.Problem);
            Choose(file);
            Assert.Equal([DataStore.Firmware], FirmwareLibrary.Folders);
            Assert.Single(_said);
        }

        // An absent folder is said, and still searched, since a drive mounted later makes it real; nothing makes it.
        [Fact]
        public void A_missing_folder_is_reported_and_not_made()
        {
            string missing = Path.Combine(_root, "Unplugged", "BIOS");
            FolderValue checkedValue = SettingFolders.Checked(missing);
            Assert.Equal((FolderState.Missing, missing), (checkedValue.State, checkedValue.Path));
            Assert.Equal($"The folder {missing} does not exist.", checkedValue.Problem);
            Choose(missing);
            Assert.Equal([missing, DataStore.Firmware], FirmwareLibrary.Folders);
            Assert.Null(FirmwareLibrary.Find(Widget));
            FirmwareOverview.Build();
            Assert.False(Directory.Exists(Path.Combine(_root, "Unplugged")));
            Assert.Equal($"appsettings.json: FirmwareDirectory: {checkedValue.Problem}", Assert.Single(_said));
        }

        [Fact]
        public void Home_firmware_chosen_as_the_folder_is_searched_once()
        {
            Choose(DataStore.Firmware + Path.DirectorySeparatorChar);
            Assert.Equal([DataStore.Firmware], FirmwareLibrary.Folders);
        }

        // A test's override is the one folder searched, whatever the settings say, so a machine's own folders never reach a test.
        [Fact]
        public void A_test_s_override_is_the_only_folder()
        {
            Choose(Chosen);
            string only = Path.Combine(_root, "Only");
            FirmwareLibrary.Directory = only;
            try { Assert.Equal([only], FirmwareLibrary.Folders); }
            finally { FirmwareLibrary.ResetDirectory(); }
        }

        // Install writes into home/Firmware; the chosen folder is never written.
        [Fact]
        public void Install_writes_into_home_firmware_and_never_into_the_chosen_folder()
        {
            Choose(Chosen);
            string picked = Path.Combine(_root, "picked.rom");
            File.WriteAllBytes(picked, new byte[64]);
            Assert.True(FirmwareLibrary.Install(Widget, picked));
            Assert.True(File.Exists(Path.Combine(DataStore.Firmware, "widget.rom")));
            Assert.Empty(Directory.GetFileSystemEntries(Chosen));
        }

        [Fact]
        public void The_overview_says_which_folder_each_file_was_found_in()
        {
            Choose(Chosen);
            var item = new EmuSen.Mistress.Views.FirmwareRow("A chip", "chip.rom", 8, true, false, "accuracy", null) { FoundIn = FirmwareItem.YourFolder };
            Assert.Equal("Found in your folder.", item.Where);
            Assert.Null((item with { Present = false, FoundIn = null }).Where);
            Assert.Equal("Not used: the file in EmuSen's folder is not 8 bytes.", (item with { Present = false, FoundIn = null, WrongSize = true, WrongSizeIn = FirmwareItem.EmuSensFolder }).Note);
        }
    }
}
