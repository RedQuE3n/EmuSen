using System.IO;
using System.Text.RegularExpressions;
using System.Xml;
using System.Xml.Linq;

namespace EmuSen.Mistress.BigPicture.Theme
{
    // Reads a theme's XML strictly, else as ES-DE does: a bare & kept, text outside the root ignored - see EmuSen_BigPicture.md §31.2.
    public static class ThemeXml
    {
        private static readonly Regex Protected = new(@"<!\[CDATA\[.*?\]\]>|<!--.*?-->|&(?:amp|lt|gt|quot|apos|#[0-9]+|#x[0-9A-Fa-f]+);|&", RegexOptions.Singleline | RegexOptions.Compiled);
        private static readonly Regex Declaration = new(@"<\?xml[^?]*\?>", RegexOptions.Compiled);

        public static XElement Load(string file, out string? leniency) => Parse(File.ReadAllText(file), out leniency);

        // Throws XmlException for XML that ES-DE refuses too, such as mismatched tags.
        public static XElement Parse(string text, out string? leniency)
        {
            leniency = null;
            try
            {
                return XDocument.Parse(text, LoadOptions.SetLineInfo).Root!;
            }
            catch (XmlException strict)
            {
                int bare = 0;
                string escaped = Protected.Replace(text, m => m.Value == "&" ? Bare(ref bare) : m.Value);
                escaped = Declaration.Replace(escaped, m => new string(' ', m.Length));
                var settings = new XmlReaderSettings { ConformanceLevel = ConformanceLevel.Fragment, DtdProcessing = DtdProcessing.Ignore };
                XElement root;
                bool outside = false;
                using (XmlReader reader = XmlReader.Create(new StringReader(escaped), settings))
                {
                    try
                    {
                        while (reader.Read() && reader.NodeType != XmlNodeType.Element)
                            if (reader.NodeType == XmlNodeType.Text) outside = true;
                        if (reader.NodeType != XmlNodeType.Element) throw strict;
                        using XmlReader subtree = reader.ReadSubtree();
                        root = XElement.Load(subtree, LoadOptions.SetLineInfo);
                    }
                    catch (XmlException)
                    {
                        throw strict;
                    }
                    try
                    {
                        while (reader.Read())
                            if (reader.NodeType is XmlNodeType.Text or XmlNodeType.Element) outside = true;
                    }
                    catch (XmlException)
                    {
                        outside = true;
                    }
                }
                leniency = (bare > 0 ? $"{bare} bare & kept as text" : "") + (bare > 0 && outside ? "; " : "") + (outside ? "text outside the root element ignored" : "");
                if (leniency.Length == 0) leniency = strict.Message;
                return root;
            }
        }

        private static string Bare(ref int count)
        {
            count++;
            return "&amp;";
        }
    }
}
