using System;
using System.IO;
using EmuSen.DianaSh.DianaSh.Bin;
using EmuSen.Galaxia.Library;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.DianaSh
{
    // A root spelled through a link is the directory the system resolves it to, as macOS's /var is /private/var - see EmuSen_DianaSh.md §13.11.
    [Collection(TestCollections.ProcessGlobals)]
    public class LinkedRootTests
    {
        [Fact]
        public void A_working_directory_entered_through_a_link_to_the_root_is_inside_it()
        {
            string temp = Path.Combine(Path.GetTempPath(), "EmuSenLinkedRoot", Guid.NewGuid().ToString("N"));
            string real = Path.Combine(temp, "real");
            string link = Path.Combine(temp, "link");
            Directory.CreateDirectory(Path.Combine(real, "home"));
            File.WriteAllText(Path.Combine(real, "home", "a.txt"), "inside");
            try { Directory.CreateSymbolicLink(link, real); }
            catch (Exception ex) when (ex is IOException or UnauthorizedAccessException)
            {
                // Windows makes links only with developer mode or elevation; Linux and macOS always can.
                Directory.Delete(temp, recursive: true);
                return;
            }
            string? data = DataStore.OverrideDirectory;
            string cwd = Environment.CurrentDirectory;
            try
            {
                string root = Path.Combine(link, "home");
                DataStore.OverrideDirectory = root;
                Environment.CurrentDirectory = root;
                // The precondition the defect needs: Linux and macOS name the working directory by its resolved path; Windows keeps the link's.
                if (!OperatingSystem.IsWindows()) Assert.NotEqual(root, Environment.CurrentDirectory);

                var shell = DianaShInterpreter.CreateDefault(null);
                Assert.Equal("inside", shell.Submit("cat a.txt").Output);
                Assert.Equal("0", shell.LastExitStatus);
                shell.Submit("echo written > b.txt");
                Assert.Equal("0", shell.LastExitStatus);
                Assert.True(File.Exists(Path.Combine(real, "home", "b.txt")));
                Assert.Equal("inside", shell.Submit($"cat \"{Path.Combine(Environment.CurrentDirectory, "a.txt")}\"").Output);
                // The resolved root's siblings stay outside it.
                File.WriteAllText(Path.Combine(real, "secret.txt"), "SECRET");
                Assert.DoesNotContain("SECRET", shell.Submit($"cat \"{Path.Combine(Path.GetDirectoryName(Environment.CurrentDirectory)!, "secret.txt")}\"").Output);
            }
            finally
            {
                Environment.CurrentDirectory = cwd;
                DataStore.OverrideDirectory = data;
                Directory.Delete(temp, recursive: true);
            }
        }
    }
}
