using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.IO;
using System.IO.Compression;
using System.Linq;
using System.Text;
using System.Text.RegularExpressions;
using System.Threading.Tasks;

namespace Lanceur
{
    /// <summary>A version's folder (versions/&lt;id&gt;/ beside the launcher): what is
    /// installed there, putting a release in, starting it.</summary>
    public class Installation
    {
        const string TagFile = ".lanceur-version";
        readonly GameVersion version;
        public readonly string Folder;

        public Installation(string launcherFolder, GameVersion version)
        {
            this.version = version;
            Folder = Path.Combine(launcherFolder, "versions", version.id);
        }

        /// <summary>The release tag installed, or null.</summary>
        public string Tag
        {
            get
            {
                string file = Path.Combine(Folder, TagFile);
                return File.Exists(file) && File.Exists(Path.Combine(Folder, version.exe ?? ""))
                    ? File.ReadAllText(file).Trim() : null;
            }
        }

        public Asset AssetOf(Release release)
        {
            if (!version.Installable) return null;
            var pattern = new Regex(version.fichier, RegexOptions.IgnoreCase);
            return release.Assets.FirstOrDefault(a => pattern.IsMatch(a.Name));
        }

        /// <summary>Downloads the release's zip and writes its files over the folder's:
        /// only the files it carries (saves, settings, caches, texture packs stay).
        /// The tag is written last, so a broken update is offered again.</summary>
        public async Task Install(Release release, IProgress<int> progress)
        {
            Asset asset = AssetOf(release);
            if (asset == null) throw new InvalidOperationException("Cette version n'a pas de fichier à installer.");
            Directory.CreateDirectory(Folder);
            string zip = Path.Combine(Folder, ".telechargement.zip");
            try
            {
                await GitHub.Download(asset.Url, zip, progress);
                await Task.Run(() => Extract(zip));
                File.WriteAllText(Path.Combine(Folder, TagFile), release.Tag, new UTF8Encoding(false));
            }
            finally
            {
                if (File.Exists(zip)) File.Delete(zip);
            }
        }

        void Extract(string zip)
        {
            using (var archive = ZipFile.OpenRead(zip))
            {
                // The release's zip holds one folder (Burnout3-PC-<version>/): its content goes in.
                var names = archive.Entries.Select(e => e.FullName).ToList();
                string top = names.Count > 0 && names[0].Contains("/") ? names[0].Substring(0, names[0].IndexOf('/') + 1) : "";
                bool strip = top.Length > 0 && names.All(n => n.StartsWith(top, StringComparison.Ordinal));
                string root = Path.GetFullPath(Folder) + Path.DirectorySeparatorChar;
                foreach (var entry in archive.Entries)
                {
                    string name = strip ? entry.FullName.Substring(top.Length) : entry.FullName;
                    if (name.Length == 0) continue;
                    string target = Path.GetFullPath(Path.Combine(Folder, name));
                    if (!target.StartsWith(root, StringComparison.OrdinalIgnoreCase)) continue;   // no path out of the folder
                    if (name.EndsWith("/")) { Directory.CreateDirectory(target); continue; }
                    // A README the player may have read is replaced too; anything not in the zip is kept.
                    Directory.CreateDirectory(Path.GetDirectoryName(target));
                    string temporary = target + ".nouveau";
                    entry.ExtractToFile(temporary, true);
                    if (File.Exists(target)) File.Delete(target);
                    File.Move(temporary, target);
                }
            }
        }

        /// <summary>Starts the version, its disc given in its pc_settings.ini (disc=),
        /// the file's other lines kept.</summary>
        public void Play(string disc)
        {
            if (!string.IsNullOrEmpty(disc)) SetDisc(disc);
            Process.Start(new ProcessStartInfo(Path.Combine(Folder, version.exe)) { WorkingDirectory = Folder, UseShellExecute = false });
        }

        void SetDisc(string disc)
        {
            string ini = Path.Combine(Folder, "pc_settings.ini");
            var lines = File.Exists(ini) ? File.ReadAllLines(ini).ToList() : new List<string> { "# The PC port's settings (PC SETTINGS in the main menu)." };
            int at = lines.FindIndex(l => l.StartsWith("disc=", StringComparison.Ordinal));
            if (at >= 0) lines[at] = "disc=" + disc;
            else lines.Add("disc=" + disc);
            File.WriteAllLines(ini, lines, new UTF8Encoding(false));
        }
    }
}
