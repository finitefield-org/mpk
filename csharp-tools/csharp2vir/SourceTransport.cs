using System;
using System.Text;

namespace Mpk.CSharp2Vir;

internal sealed class CapturedSourceText
{
    internal CapturedSourceText(string normalizedPath, string text)
    {
        NormalizedPath = normalizedPath;
        Text = text;
    }

    internal string NormalizedPath { get; }

    internal string Text { get; }
}

internal sealed class CapturedSourceSet
{
    private readonly CapturedSourceText[] sources;

    internal CapturedSourceSet(CapturedSourceText[] sources)
    {
        this.sources = sources;
    }

    internal int Count => sources.Length;

    internal CapturedSourceText SourceAt(int index) => sources[index];
}

internal static class SourceTransport
{
    private static readonly UTF8Encoding StrictUtf8 = new UTF8Encoding(false, true);

    internal static CapturedSourceSet Validate(CapturedSnapshot snapshot)
    {
        System.Collections.Generic.IReadOnlyList<string> selectedPaths = snapshot.Selection.Raw.Sources;
        var sources = new CapturedSourceText[selectedPaths.Count];
        for (int index = 0; index < selectedPaths.Count; index++)
        {
            CapturedFile captured = snapshot.Find(CapturedInputKind.Source, selectedPaths[index]);
            sources[index] = new CapturedSourceText(captured.NormalizedPath, Decode(captured.Bytes));
        }

        return new CapturedSourceSet(sources);
    }

    internal static string Decode(ReadOnlySpan<byte> bytes)
    {
        if (bytes.Length == 0
            || (bytes.Length >= 3 && bytes[0] == 0xef && bytes[1] == 0xbb && bytes[2] == 0xbf)
            || bytes[^1] != (byte)'\n')
        {
            throw FrontendFailure.SourceError("CSHARP_SOURCE_ENCODING");
        }

        foreach (byte value in bytes)
        {
            if (value == 0 || value == (byte)'\r')
            {
                throw FrontendFailure.SourceError("CSHARP_SOURCE_ENCODING");
            }
        }

        string text;
        try
        {
            text = StrictUtf8.GetString(bytes);
        }
        catch (DecoderFallbackException)
        {
            throw FrontendFailure.SourceError("CSHARP_SOURCE_ENCODING");
        }

        foreach (Rune rune in text.EnumerateRunes())
        {
            int scalar = rune.Value;
            if ((scalar >= 0xfdd0 && scalar <= 0xfdef)
                || (scalar & 0xffff) == 0xfffe
                || (scalar & 0xffff) == 0xffff
                || scalar == 0x85
                || scalar == 0x2028
                || scalar == 0x2029)
            {
                throw FrontendFailure.SourceError("CSHARP_SOURCE_ENCODING");
            }
        }

        return text;
    }
}
