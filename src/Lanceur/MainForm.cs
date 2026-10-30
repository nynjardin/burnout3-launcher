using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.Drawing;
using System.IO;
using System.Linq;
using System.Text;
using System.Threading.Tasks;
using System.Windows.Forms;

namespace Lanceur
{
    /// <summary>The launcher's one window: the versions on the left, the chosen
    /// one's card, releases and buttons on the right, the disc images below.</summary>
    public class MainForm : Form
    {
        readonly string folder;
        readonly Catalogue catalogue;
        readonly Settings settings;
        // Each version's releases, once GitHub answered (or why it did not).
        readonly Dictionary<string, List<Release>> releases = new Dictionary<string, List<Release>>();
        readonly Dictionary<string, string> errors = new Dictionary<string, string>();
        bool busy;

        readonly ListBox list = new ListBox { Dock = DockStyle.Left, Width = 230, IntegralHeight = false };
        readonly Label title = new Label { Dock = DockStyle.Top, Height = 34, Font = new Font("Segoe UI", 14f, FontStyle.Bold) };
        readonly Label status = new Label { Dock = DockStyle.Top, Height = 44 };
        readonly TextBox card = new TextBox { Dock = DockStyle.Fill, Multiline = true, ReadOnly = true, ScrollBars = ScrollBars.Vertical, BackColor = SystemColors.Window };
        readonly Button install = new Button { Width = 150, Height = 32 };
        readonly Button play = new Button { Text = "Jouer", Width = 110, Height = 32 };
        readonly Button openFolder = new Button { Text = "Dossier", Width = 90, Height = 32 };
        readonly Button openPage = new Button { Text = "Page du projet", Width = 120, Height = 32 };
        readonly ProgressBar progress = new ProgressBar { Dock = DockStyle.Bottom, Height = 14, Visible = false };
        readonly Label ps2Disc = new Label { AutoSize = false, Width = 520, Height = 24, TextAlign = ContentAlignment.MiddleLeft };
        readonly Label xboxDisc = new Label { AutoSize = false, Width = 520, Height = 24, TextAlign = ContentAlignment.MiddleLeft };

        public MainForm(string folder)
        {
            this.folder = folder;
            catalogue = Catalogue.Load(folder);
            settings = Settings.Load(folder);

            Text = "Lanceur Burnout 3";
            Width = 900;
            Height = 600;
            MinimumSize = new Size(760, 480);
            Font = new Font("Segoe UI", 9.5f);
            StartPosition = FormStartPosition.CenterScreen;

            var buttons = new FlowLayoutPanel { Dock = DockStyle.Bottom, Height = 44, Padding = new Padding(0, 6, 0, 0) };
            buttons.Controls.AddRange(new Control[] { install, play, openFolder, openPage });
            var right = new Panel { Dock = DockStyle.Fill, Padding = new Padding(12, 6, 8, 6) };
            right.Controls.Add(card);
            right.Controls.Add(buttons);
            right.Controls.Add(status);
            right.Controls.Add(title);

            var discs = new TableLayoutPanel { Dock = DockStyle.Bottom, Height = 64, ColumnCount = 2, Padding = new Padding(8, 4, 8, 4) };
            discs.Controls.Add(ps2Disc, 0, 0);
            discs.Controls.Add(DiscButton("ps2_us", "Disque PS2 US…"), 1, 0);
            discs.Controls.Add(xboxDisc, 0, 1);
            discs.Controls.Add(DiscButton("xbox_us", "Disque Xbox US…"), 1, 1);

            Controls.Add(right);
            Controls.Add(list);
            Controls.Add(discs);
            Controls.Add(progress);

            foreach (var v in catalogue.versions) list.Items.Add(v.nom);
            list.SelectedIndexChanged += (s, e) => { Show(); Remember(); };
            install.Click += async (s, e) => await InstallChosen();
            play.Click += (s, e) => PlayChosen();
            openFolder.Click += (s, e) => { var i = Chosen(); if (i != null) { Directory.CreateDirectory(i.Folder); Process.Start("explorer.exe", "\"" + i.Folder + "\""); } };
            openPage.Click += (s, e) => { var v = ChosenVersion(); if (v != null) Process.Start(v.Page); };
            Load += async (s, e) => await LoadReleases();

            int start = Math.Max(0, catalogue.versions.FindIndex(v => v.id == settings.selected));
            if (list.Items.Count > 0) list.SelectedIndex = start;
            ShowDiscs();
        }

        Button DiscButton(string kind, string text)
        {
            var button = new Button { Text = text, Width = 140, Height = 26 };
            button.Click += (s, e) =>
            {
                using (var dialog = new OpenFileDialog { Filter = "Images disque (*.iso;*.bin;*.cue)|*.iso;*.bin;*.cue|Tous les fichiers|*.*", Title = text.TrimEnd('…') })
                {
                    if (dialog.ShowDialog(this) != DialogResult.OK) return;
                    settings.discs[kind] = dialog.FileName;
                    settings.Save(folder);
                    ShowDiscs();
                    Show();
                }
            };
            return button;
        }

        void ShowDiscs()
        {
            ps2Disc.Text = "Disque PS2 US : " + (settings.Disc("ps2_us") ?? "(aucun)");
            xboxDisc.Text = "Disque Xbox US : " + (settings.Disc("xbox_us") ?? "(aucun)");
        }

        GameVersion ChosenVersion() { return list.SelectedIndex >= 0 ? catalogue.versions[list.SelectedIndex] : null; }
        Installation Chosen() { var v = ChosenVersion(); return v == null ? null : new Installation(folder, v); }

        void Remember()
        {
            var v = ChosenVersion();
            if (v == null || settings.selected == v.id) return;
            settings.selected = v.id;
            try { settings.Save(folder); } catch (IOException) { }
        }

        async Task LoadReleases()
        {
            foreach (var v in catalogue.versions)
            {
                try { releases[v.id] = await GitHub.Releases(v.depot); }
                catch (Exception ex) { errors[v.id] = ex.Message; }
                if (v == ChosenVersion()) Show();
            }
        }

        Release Latest(GameVersion v)
        {
            List<Release> list;
            return releases.TryGetValue(v.id, out list) ? list.FirstOrDefault() : null;
        }

        /// <summary>The chosen version: its card, what is installed, what is out, the buttons.</summary>
        new void Show()
        {
            var v = ChosenVersion();
            if (v == null) return;
            var installation = Chosen();
            string installed = installation.Tag;
            Release latest = Latest(v);
            string error;

            title.Text = v.nom;
            var line = new StringBuilder("Plateforme : " + v.plateforme + "   ·   github.com/" + v.depot + "\r\n");
            line.Append("Installée : " + (installed ?? "non") + "   ·   Dernière : ");
            if (latest != null) line.Append(latest.Tag + (latest.Prerelease ? " (préversion)" : "") + ", " + latest.Published.ToString("dd/MM/yyyy"));
            else if (errors.TryGetValue(v.id, out error)) line.Append("GitHub ne répond pas (" + error + ")");
            else if (releases.ContainsKey(v.id)) line.Append("aucune version publiée");
            else line.Append("…");
            status.Text = line.ToString();

            var text = new StringBuilder("Ce qu'elle apporte :\r\n");
            foreach (var a in v.apports) text.Append("  • " + a + "\r\n");
            if (!string.IsNullOrEmpty(v.remarque)) text.Append("\r\n" + v.remarque + "\r\n");
            List<Release> all;
            if (releases.TryGetValue(v.id, out all))
                foreach (var r in all.Take(3))
                {
                    text.Append("\r\n──── " + r.Tag + "  (" + r.Published.ToString("dd/MM/yyyy") + ")" + (r.Prerelease ? "  préversion" : "") + "\r\n");
                    text.Append(r.Notes.Replace("\r\n", "\n").Replace("\n", "\r\n").Trim() + "\r\n");
                }
            card.Text = text.ToString();

            Asset asset = latest != null ? installation.AssetOf(latest) : null;
            install.Text = installed == null ? "Installer" : installed == latest?.Tag ? "À jour" : "Mettre à jour";
            install.Enabled = !busy && asset != null && installed != latest.Tag;
            play.Enabled = !busy && installed != null;
            openPage.Text = v.Installable ? "Page du projet" : "Page du projet ↗";
        }

        async Task InstallChosen()
        {
            var v = ChosenVersion();
            Release latest = v != null ? Latest(v) : null;
            if (latest == null) return;
            busy = true;
            progress.Value = 0;
            progress.Visible = true;
            Show();
            try
            {
                await Chosen().Install(latest, new Progress<int>(p => progress.Value = Math.Min(100, p)));
            }
            catch (Exception ex)
            {
                MessageBox.Show(this, "L'installation a échoué :\r\n" + ex.Message, Text, MessageBoxButtons.OK, MessageBoxIcon.Error);
            }
            finally
            {
                busy = false;
                progress.Visible = false;
                Show();
            }
        }

        void PlayChosen()
        {
            var v = ChosenVersion();
            if (v == null) return;
            string disc = settings.Disc(v.disque);
            if (disc == null)
            {
                MessageBox.Show(this, "Choisis d'abord l'image du disque (en bas de la fenêtre).", Text, MessageBoxButtons.OK, MessageBoxIcon.Information);
                return;
            }
            try { Chosen().Play(disc); }
            catch (Exception ex) { MessageBox.Show(this, "Le jeu ne démarre pas :\r\n" + ex.Message, Text, MessageBoxButtons.OK, MessageBoxIcon.Error); }
        }
    }
}
