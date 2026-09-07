using System;
using System.Collections.Generic;
using System.Collections.Immutable;
using System.IO;
using System.Linq;
using System.Text;
using System.Text.Json;
using Microsoft.CodeAnalysis;
namespace Mpk.CSharp2Vir;
internal static class PracticalLoopContractsHarness
{
    public static int Main(string[] args)
    {
        string stage = "source_cases";
        try {
            var references = Directory.EnumerateFiles(Path.Combine(args[0], "ref", "net10.0"), "*.dll")
                .OrderBy(p => p, StringComparer.Ordinal).Select(p => MetadataReference.CreateFromFile(p)).ToImmutableArray<MetadataReference>();
            string integer = PracticalIdentity.PrimitiveId("i32");
            string root = PracticalIdentity.CallableId("method", "Business", PracticalIdentity.SourceTypeId("Business", "Entry"), "Run",
                new[] { integer, PracticalIdentity.ClosedInstanceId("bounded_sequence", integer), PracticalIdentity.PrimitiveId("string") }, integer);
            var cases = new List<(string Id, string Body, bool Accept)> {
                ("while", "int i=0;while(i<n){i++;}return i;", true),
                ("do", "int i=0;do{i++;}while(i<n);return i;", true),
                ("for", "for(int i=0;i<n;i++){if(i==2)continue;if(i==4)break;}return n;", true),
                ("foreach_array", "int sum=0;foreach(int item in a){sum+=item;}return sum;", true),
                ("foreach_array_var", "int sum=0;foreach(var item in a){sum+=item;}return sum;", true),
                ("foreach_string", "int sum=0;foreach(char item in s){sum+=item;}return sum;", true),
                ("foreach_string_var", "int sum=0;foreach(var item in s){sum+=item;}return sum;", true),
                ("nested", "int i=0;while(i<n){for(int j=0;j<n;j++){if(j==2)continue;if(j==3)break;}i++;}return i;", true),
                ("return", "while(n>0){if(n==2)return n;n--;}return 0;", true),
                ("switch_break", "while(n>0){switch(n){case 2:break;default:n--;break;}break;}return n;", true),
                ("scope", "int before=0;while(before<n){int body=before;before++;}int after=n;return after;", true),
                ("shadow", "{int i=0;while(i<n){i++;}}{int i=0;while(i<n){i++;}}return n;", true),
                ("array_fill", "int[] output=new int[n];for(int i=0;i<n;i++){output[i]=i;}return output.Length;", true),
                ("count_fill", "int count=0;foreach(int item in a){if(item>0)count++;}int[] output=new int[count];int i=0;foreach(int item in a){if(item>0){output[i]=item;i++;}}return output.Length;", true),
                ("borrow_write", "foreach(int item in a){a[0]=item;}return n;", true),
                ("partial", "while(true){}", true),
                ("conditional_directive", "\n#if true\nwhile(n>0){n--;}\n#endif\nreturn n;", false),
                ("nullable_directive", "\n#nullable disable\nwhile(n>0){n--;}return n;", false),
                ("none", "return n;", true),
                ("unicode", "// 日本語 😀\nint i=0;while(i<n){i++;}return i;", true),
                ("impure_source", "while(n>0){System.Console.WriteLine(n);n--;}return n;", false),
                ("generic", "var xs=new System.Collections.Generic.List<int>();while(n>0){n--;}return n;", false),
            };
            foreach (int count in new[] { 31, 32, 33 })
                cases.Add(("loops_"+count, string.Concat(Enumerable.Repeat("while(n>0){n--;}", count))+"return n;", count<=32));
            foreach (int depth in new[] { 7, 8, 9 })
                cases.Add(("nesting_"+depth, string.Concat(Enumerable.Repeat("while(n>0){",depth))+"n--;"+new string('}',depth)+"return n;",depth<=8));
            var rows = new List<object>();
            foreach(var item in cases) {
                string source="namespace Business;public static class Entry{public static int Run(int n,int[] a,string s){"+item.Body+"}}\n";
                var input = new PracticalCapturedInput(PracticalCapturedInputKind.Source,"src/Entry.cs",Encoding.UTF8.GetBytes(source));
                var selection = new PracticalSourceSelection(CSharpPracticalCapture.SelectionSchema,"data",new[]{"src/Entry.cs"},new[]{root},Array.Empty<string>());
                try {
                    byte[] bytes=CSharpPracticalLoopContracts.Capture(selection,new[]{input},references);
                    if(!item.Accept)throw new InvalidOperationException("unexpected_accept:"+item.Id);
                    if(!bytes.SequenceEqual(CSharpPracticalLoopContracts.Capture(selection,new[]{input},references)))throw new InvalidOperationException("determinism:"+item.Id);
                    rows.Add(new {id=item.Id,source,root,accepted=true,facts=JsonSerializer.Deserialize<JsonElement>(bytes),diagnostic="",code=""});
                } catch(PracticalCaptureFailure e) {
                    if(item.Accept)throw new InvalidOperationException(item.Id+":"+e.Family+":"+e.Code);
                    if(e.ArtifactCount!=0)throw new InvalidOperationException("artifacts");
                    rows.Add(new {id=item.Id,source,root,accepted=false,facts=(object?)null,diagnostic=e.Family.ToString(),code=e.Code});
                }
            }
            foreach (var variant in new[] { "set_add", "set_count", "map_add", "map_replace" }) {
                stage = variant;
                bool map = variant.StartsWith("map", StringComparison.Ordinal);
                string wrapper = map ? "Map" : "Set";
                string wrapperId = PracticalIdentity.SourceTypeId("Business", wrapper);
                string prefix = map ? "public readonly struct Pair{public readonly int Key;public readonly int Value;public Pair(int key,int value){Key=key;Value=value;}}" : "";
                prefix += "public sealed class " + wrapper + "{public readonly " + (map ? "Pair" : "int") + "[] Items;public " + wrapper + "(" + (map ? "Pair" : "int") + "[] items){Items=items;}}";
                bool count = variant == "set_count";
                string methodName = count ? "Count" : variant == "map_replace" ? "Replace" : "Add";
                string source = "namespace Business;" + prefix + "public static class Entry{public static " + (count ? "uint" : wrapper) + " " + methodName + "(" + wrapper + " collection" + (count ? "" : ",int key" + (map ? ",int value" : "")) + "){" + (map ? "new Pair(key,value);" : "") + "new " + wrapper + "(collection.Items);int i=0;while(i<collection.Items.Length){i++;}return " + (count ? "(uint)i" : "collection") + ";}}\n";
                var parameters = new List<string>{wrapperId}; if(!count){parameters.Add(integer);if(map)parameters.Add(integer);}
                string selected = PracticalIdentity.CallableId("method","Business",PracticalIdentity.SourceTypeId("Business","Entry"),methodName,parameters.ToArray(),count?PracticalIdentity.PrimitiveId("u32"):wrapperId);
                var input=new PracticalCapturedInput(PracticalCapturedInputKind.Source,"src/Entry.cs",Encoding.UTF8.GetBytes(source));
                var selection=new PracticalSourceSelection(CSharpPracticalCapture.SelectionSchema,"data",new[]{"src/Entry.cs"},new[]{selected},Array.Empty<string>());
                byte[] bytes=CSharpPracticalLoopContracts.Capture(selection,new[]{input},references);
                if(!bytes.SequenceEqual(CSharpPracticalLoopContracts.Capture(selection,new[]{input},references)))throw new InvalidOperationException("collection_determinism");
                rows.Add(new{id=variant,source,root=selected,accepted=true,facts=JsonSerializer.Deserialize<JsonElement>(bytes),diagnostic="",code=""});
            }
            File.WriteAllBytes("loop-source-cases.json",JsonSerializer.SerializeToUtf8Bytes(rows));
            return 0;
        } catch(Exception e){Console.Error.WriteLine(stage+":"+(e is PracticalCaptureFailure f ? f.Family+":"+f.Code : e.ToString()));return 1;}
    }
}
