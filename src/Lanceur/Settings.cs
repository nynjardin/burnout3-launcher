using System.Collections.Generic;
using System.IO;
using System.Text;
using System.Web.Script.Serialization;

namespace Lanceur
{
    /// <summary>The launcher's own settings, in lanceur.json beside it.</summary>
    public class Settings
    {
        public Dictionary<string, string> discs { get; set; }   // "ps2_us", "xbox_us" -> image path
        public string selected { get; set; }                    // the version shown last

        public Settings() { discs = new Dictionary<string, string>(); }

        static string PathIn(string folder) { return Path.Combine(folder, "lanceur.json"); }

        public static Settings Load(string folder)
        {
            try
            {
                var settings = new JavaScriptSerializer().Deserialize<Settings>(File.ReadAllText(PathIn(folder)));
                if (settings.discs == null) settings.discs = new Dictionary<string, string>();
                return settings;
            }
            catch
            {
                return new Settings();
            }
        }

        public void Save(string folder)
        {
            File.WriteAllText(PathIn(folder), new JavaScriptSerializer().Serialize(this), new UTF8Encoding(false));
        }

        public string Disc(string kind)
        {
            string path;
            return kind != null && discs.TryGetValue(kind, out path) ? path : null;
        }
    }
}
