using System;
using System.Collections.Generic;
using System.IO;
using EmuSen.Galaxia;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Galaxia
{
    // The persistence every config file in the project now shares - see
    // EmuSen_Config_Reference.md §2.
    [Collection(TestCollections.ProcessGlobals)]
    public class ConfigFileTests : IDisposable
    {
        private sealed class Sample
        {
            public string Name { get; set; } = "";
            public int Count { get; set; }
        }

        private readonly string _dir;

        public ConfigFileTests()
        {
            _dir = Path.Combine(Path.GetTempPath(), "EmuSenGalaxia_" + Guid.NewGuid().ToString("N"));
            ConfigStore.OverrideDirectory = _dir;
        }

        public void Dispose()
        {
            ConfigStore.OverrideDirectory = null;
            ConfigStore.OverrideLegacyDirectory = null;
            try { Directory.Delete(_dir, recursive: true); } catch { }
        }

        [Fact]
        public void Round_trips_through_the_file()
        {
            var file = new ConfigFile<Sample>("sample.json");

            Assert.True(file.Save(new Sample { Name = "venus", Count = 7 }));
            Sample? loaded = file.Load();

            Assert.NotNull(loaded);
            Assert.Equal("venus", loaded!.Name);
            Assert.Equal(7, loaded.Count);
        }

        [Fact]
        public void Creates_the_directory_it_writes_into()
        {
            Assert.False(Directory.Exists(_dir));

            Assert.True(new ConfigFile<Sample>("sample.json").Save(new Sample()));

            Assert.True(Directory.Exists(_dir));
        }

        [Fact]
        public void Missing_file_loads_as_null_and_falls_back_to_the_supplied_default()
        {
            var file = new ConfigFile<Sample>("absent.json");

            Assert.Null(file.Load());
            Assert.Equal("fallback", file.Load(() => new Sample { Name = "fallback" }).Name);
        }

        // The whole point of the swallow-everything contract: a hand-edited
        // file with a typo in it must not take the program down.
        [Fact]
        public void Corrupt_file_loads_as_null_rather_than_throwing()
        {
            Directory.CreateDirectory(_dir);
            File.WriteAllText(Path.Combine(_dir, "broken.json"), "{ this is not json");

            Assert.Null(new ConfigFile<Sample>("broken.json").Load());
        }

        [Fact]
        public void Comments_and_trailing_commas_survive_a_hand_edit()
        {
            Directory.CreateDirectory(_dir);
            File.WriteAllText(Path.Combine(_dir, "edited.json"), """
                {
                  // bumped this by hand
                  "name": "venus",
                  "count": 3,
                }
                """);

            Sample? loaded = new ConfigFile<Sample>("edited.json").Load();

            Assert.NotNull(loaded);
            Assert.Equal("venus", loaded!.Name);
            Assert.Equal(3, loaded.Count);
        }

        // Save writes a temp file and renames it - if that temp were left
        // behind, `ls /etc/EmuSen` would fill up with .tmp files.
        [Fact]
        public void Save_leaves_no_temp_file_behind()
        {
            new ConfigFile<Sample>("sample.json").Save(new Sample());

            Assert.Empty(Directory.GetFiles(_dir, "*.tmp"));
            Assert.Single(Directory.GetFiles(_dir));
        }

        [Fact]
        public void Overwriting_replaces_the_previous_contents_entirely()
        {
            var file = new ConfigFile<Sample>("sample.json");
            file.Save(new Sample { Name = "a-much-longer-value-than-the-next-one", Count = 1 });

            file.Save(new Sample { Name = "b", Count = 2 });

            Assert.Equal("b", file.Load()!.Name);
        }

        [Fact]
        public void A_dictionary_is_a_config_file_too()
        {
            var file = new ConfigFile<Dictionary<string, int>>("map.json");

            file.Save(new Dictionary<string, int> { ["Start"] = 6, ["Select"] = 4 });

            Dictionary<string, int>? loaded = file.Load();
            Assert.NotNull(loaded);
            Assert.Equal(6, loaded!["Start"]);
            Assert.Equal(4, loaded["Select"]);
        }

        [Fact]
        public void Category_files_land_in_their_own_subdirectory()
        {
            var file = new ConfigFile<Sample>("zelda", "sample.json");

            Assert.Equal(Path.Combine(_dir, "zelda", "sample.json"), file.Path);
            Assert.True(file.Save(new Sample()));
            Assert.True(File.Exists(file.Path));
        }

        [Fact]
        public void Delete_removes_the_file_and_is_quiet_about_one_that_is_not_there()
        {
            var file = new ConfigFile<Sample>("sample.json");
            file.Save(new Sample());

            Assert.True(file.Delete());
            Assert.False(file.Exists);
            Assert.True(file.Delete());
        }

        // Path is resolved per access, not cached - the binding maps hold
        // their ConfigFile in a static field that outlives any one test.
        [Fact]
        public void Path_follows_the_override_when_it_moves()
        {
            var file = new ConfigFile<Sample>("sample.json");
            string before = file.Path;

            ConfigStore.OverrideDirectory = Path.Combine(_dir, "elsewhere");

            Assert.NotEqual(before, file.Path);
            Assert.Equal(Path.Combine(_dir, "elsewhere", "sample.json"), file.Path);
        }

        [Fact]
        public void First_load_migrates_the_pre_Galaxia_file_and_leaves_the_original_alone()
        {
            string legacy = Path.Combine(_dir, "legacy");
            Directory.CreateDirectory(legacy);
            File.WriteAllText(Path.Combine(legacy, "sample.json"), """{"name":"from-appdata","count":42}""");
            ConfigStore.OverrideLegacyDirectory = legacy;

            var file = new ConfigFile<Sample>("sample.json");
            Sample? loaded = file.Load();

            Assert.NotNull(loaded);
            Assert.Equal("from-appdata", loaded!.Name);
            Assert.True(file.Exists);
            Assert.True(File.Exists(Path.Combine(legacy, "sample.json")));
        }

        // Once migrated, the new file is authoritative - editing config must
        // not silently revert to whatever %AppData% still holds.
        [Fact]
        public void Migration_does_not_run_again_once_the_new_file_exists()
        {
            string legacy = Path.Combine(_dir, "legacy");
            Directory.CreateDirectory(legacy);
            File.WriteAllText(Path.Combine(legacy, "sample.json"), """{"name":"from-appdata","count":42}""");
            ConfigStore.OverrideLegacyDirectory = legacy;

            var file = new ConfigFile<Sample>("sample.json");
            file.Save(new Sample { Name = "current", Count = 1 });

            Assert.Equal("current", file.Load()!.Name);
        }

        // Guards the developer's own ~/.config/EmuSen against every test in
        // this suite that redirects the config directory but says nothing
        // about the legacy one.
        [Fact]
        public void An_override_with_no_legacy_override_never_reaches_the_real_config()
        {
            ConfigStore.OverrideLegacyDirectory = null;

            Assert.Null(new ConfigFile<Sample>("appsettings.json").Load());
        }
    }
}
