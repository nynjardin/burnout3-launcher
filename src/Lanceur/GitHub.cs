using System;
using System.Collections.Generic;
using System.IO;
using System.Net;
using System.Net.Http;
using System.Threading.Tasks;
using System.Web.Script.Serialization;

namespace Lanceur
{
    public class Asset
    {
        public string Name;
        public string Url;
        public long Size;
    }

    public class Release
    {
        public string Tag;
        public string Title;
        public DateTime Published;
        public bool Prerelease;
        public string Notes;
        public List<Asset> Assets = new List<Asset>();
    }

    /// <summary>GitHub's public API, without an account (60 requests an hour).</summary>
    public static class GitHub
    {
        static readonly HttpClient Http = MakeClient();

        static HttpClient MakeClient()
        {
            ServicePointManager.SecurityProtocol |= SecurityProtocolType.Tls12;
            var client = new HttpClient();
            client.DefaultRequestHeaders.UserAgent.ParseAdd("Lanceur-Burnout3/0.1");
            client.DefaultRequestHeaders.Accept.ParseAdd("application/vnd.github+json");
            return client;
        }

        /// <summary>The repository's releases, newest first, prereleases included
        /// (releases/latest leaves them out, and upstream publishes nothing else).</summary>
        public static async Task<List<Release>> Releases(string repository)
        {
            string json = await Http.GetStringAsync("https://api.github.com/repos/" + repository + "/releases?per_page=10");
            var list = new List<Release>();
            var items = new JavaScriptSerializer { MaxJsonLength = int.MaxValue }.Deserialize<object[]>(json);
            foreach (Dictionary<string, object> item in items)
            {
                if (item["draft"] is bool && (bool)item["draft"]) continue;
                var release = new Release
                {
                    Tag = item["tag_name"] as string,
                    Title = item["name"] as string,
                    Prerelease = item["prerelease"] is bool && (bool)item["prerelease"],
                    Notes = item["body"] as string ?? "",
                };
                DateTime published;
                if (DateTime.TryParse(item["published_at"] as string, out published)) release.Published = published;
                foreach (Dictionary<string, object> a in (object[])item["assets"])
                    release.Assets.Add(new Asset
                    {
                        Name = a["name"] as string,
                        Url = a["browser_download_url"] as string,
                        Size = Convert.ToInt64(a["size"]),
                    });
                list.Add(release);
            }
            list.Sort((x, y) => y.Published.CompareTo(x.Published));
            return list;
        }

        /// <summary>Downloads to a file, telling how far it got (0-100).</summary>
        public static async Task Download(string url, string file, IProgress<int> progress)
        {
            using (var response = await Http.GetAsync(url, HttpCompletionOption.ResponseHeadersRead))
            {
                response.EnsureSuccessStatusCode();
                long total = response.Content.Headers.ContentLength ?? -1, done = 0;
                using (var input = await response.Content.ReadAsStreamAsync())
                using (var output = File.Create(file))
                {
                    var buffer = new byte[1 << 16];
                    int read, last = -1;
                    while ((read = await input.ReadAsync(buffer, 0, buffer.Length)) > 0)
                    {
                        await output.WriteAsync(buffer, 0, read);
                        done += read;
                        int percent = total > 0 ? (int)(done * 100 / total) : 0;
                        if (percent != last) progress.Report(last = percent);
                    }
                }
            }
        }
    }
}
