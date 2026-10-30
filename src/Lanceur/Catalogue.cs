using System;
using System.Collections.Generic;
using System.IO;
using System.Reflection;
using System.Web.Script.Serialization;

namespace Lanceur
{
    /// <summary>A version of the game the launcher knows, and its card.</summary>
    public class GameVersion
    {
        public string id { get; set; }
        public string nom { get; set; }
        public string depot { get; set; }          // GitHub "owner/name"
        public string plateforme { get; set; }
        public string disque { get; set; }         // which disc image it plays: a key of Settings.discs
        public string fichier { get; set; }        // regex of the release asset to install; empty: none portable
        public string exe { get; set; }            // the program in the version's folder
        public List<string> apports { get; set; }
        public string remarque { get; set; }

        public bool Installable { get { return !string.IsNullOrEmpty(fichier) && !string.IsNullOrEmpty(exe); } }
        public string Page { get { return "https://github.com/" + depot; } }
    }

    public class Catalogue
    {
        public List<GameVersion> versions { get; set; }

        /// <summary>A catalogue.json beside the launcher wins over the one built in.</summary>
        public static Catalogue Load(string folder)
        {
            string text;
            string beside = Path.Combine(folder, "catalogue.json");
            if (File.Exists(beside))
                text = File.ReadAllText(beside);
            else
                using (var stream = Assembly.GetExecutingAssembly().GetManifestResourceStream("catalogue.json"))
                using (var reader = new StreamReader(stream))
                    text = reader.ReadToEnd();
            var catalogue = new JavaScriptSerializer().Deserialize<Catalogue>(text);
            if (catalogue.versions == null) catalogue.versions = new List<GameVersion>();
            foreach (var v in catalogue.versions)
                if (v.apports == null) v.apports = new List<string>();
            return catalogue;
        }
    }
}
