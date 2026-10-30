using System;
using System.IO;
using System.Windows.Forms;

namespace Lanceur
{
    static class Program
    {
        [STAThread]
        static void Main()
        {
            Application.EnableVisualStyles();
            Application.SetCompatibleTextRenderingDefault(false);
            // Portable: everything lives beside the launcher's exe.
            string folder = Path.GetDirectoryName(Application.ExecutablePath);
            Application.Run(new MainForm(folder));
        }
    }
}
