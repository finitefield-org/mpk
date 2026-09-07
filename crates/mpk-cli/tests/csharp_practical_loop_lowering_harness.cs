using System;
using System.Collections.Generic;
using System.Collections.Immutable;
using System.IO;
using System.Linq;
using System.Reflection;
using System.Text;
using System.Text.Json;
using Microsoft.CodeAnalysis;
using Microsoft.CodeAnalysis.CSharp;
namespace Mpk.CSharp2Vir;
internal static class PracticalLoopLoweringHarness
{
    public static int Main(string[] args)
    {
        string stage="setup";
        try {
            var references=Directory.EnumerateFiles(Path.Combine(args[0],"ref","net10.0"),"*.dll").OrderBy(p=>p,StringComparer.Ordinal).Select(p=>MetadataReference.CreateFromFile(p)).ToImmutableArray<MetadataReference>();
            string integer=PracticalIdentity.PrimitiveId("i32");
            string root=PracticalIdentity.CallableId("method","Business",PracticalIdentity.SourceTypeId("Business","Entry"),"Run",new[]{integer,PracticalIdentity.ClosedInstanceId("bounded_sequence",integer),PracticalIdentity.PrimitiveId("string")},integer);
            var cases=new List<(string Id,string Body,bool Accept)> {
                ("while","int i=0;while(i<n){i++;}return i;",true),
                ("do","int i=0;do{i++;}while(i<n);return i;",true),
                ("for","int sum=0;for(int i=0;i<n;i++){if(i==2)continue;if(i==5)break;sum+=i;}return sum;",true),
                ("nested","int sum=0;for(int i=0;i<n;i++){for(int j=0;j<n;j++){if(j==2)break;if(j==1)continue;sum++;}}return sum;",true),
                ("foreach_array","int sum=0;foreach(int item in a){sum+=item;}return sum;",true),
                ("foreach_array_var","int sum=0;foreach(var item in a){sum+=item;}return sum;",true),
                ("foreach_string","int sum=0;foreach(char item in s){sum+=item;}return sum;",true),
                ("foreach_string_var","int sum=0;foreach(var item in s){sum+=item;}return sum;",true),
                ("return","while(n>0){if(n==2)return n;n--;}return 0;",true),
                ("short_circuit","int i=0;int sum=0;while(i<n && ++sum<4){i++;}return sum*10+i;",true),
                ("do_continue","int i=0;int sum=0;do{i++;if(i<2)continue;sum+=i;}while(i<n);return sum;",true),
                ("both_abrupt","int sum=0;for(int i=0;i<n;i++){if(i==2)break;else continue;}return sum;",true),
                ("foreach_once","int sum=0;foreach(var item in (n++>=0?a:a)){sum+=item;}return sum*10+n;",true),
                ("index_update","int[] x=new int[3];int i=0;for(int j=0;j<2;j++){x[i++]++;}return x[0]*10+x[1];",true),
                ("membership","foreach(var item in a){if(item==n)return 1;}return 0;",true),
                ("lookup","for(int i=0;i<a.Length;i++){if(a[i]==n)return i;}return -1;",true),
                ("count_fill","int count=0;foreach(var item in a){if(item>0)count++;}int[] output=new int[count];int i=0;foreach(var item in a){if(item>0){output[i]=item;i++;}}int sum=0;foreach(var item in output){sum+=item;}return sum;",true),
                ("count_fill_disagreement","int count=0;foreach(var item in a){if(item>0)count++;}int[] output=new int[count];int i=0;foreach(var item in a){if(item>=0){output[i]=item;i++;}}return i;",true),
                ("sort","int[] output=new int[a.Length];for(int i=0;i<a.Length;i++){output[i]=a[i];}for(int i=0;i<output.Length;i++){for(int j=i+1;j<output.Length;j++){if(output[j]<output[i]){int tmp=output[i];output[i]=output[j];output[j]=tmp;}}}int hash=0;foreach(var item in output){hash=hash*3+item;}return hash;",true),
                ("dedup","int count=0;for(int i=0;i<a.Length;i++){if(i==0||a[i]!=a[i-1])count++;}int[] output=new int[count];int j=0;for(int i=0;i<a.Length;i++){if(i==0||a[i]!=a[i-1]){output[j]=a[i];j++;}}int sum=0;foreach(var item in output){sum+=item;}return sum;",true),
                ("duplicate_reject","for(int i=0;i<a.Length;i++){for(int j=i+1;j<a.Length;j++){if(a[i]==a[j])return -1;}}return a.Length;",true),
                ("duplicate_replace","int[] output=new int[a.Length];for(int i=0;i<a.Length;i++){output[i]=a[i];}for(int i=0;i<output.Length;i++){if(output[i]==n)output[i]=0;}int sum=0;foreach(var item in output){sum+=item;}return sum;",true),
                ("borrow_write","int[] x=new int[n];foreach(var item in x){x[0]=item;}return n;",false),
                ("borrow_alias","int[] x=new int[n];int[] alias=x;foreach(var item in alias){x[0]=item;}return n;",false),
                ("frozen_backedge","int[] x=new int[n];for(int i=0;i<n;i++){x[0]=i;int[] alias=x;}return n;",false),
                ("readonly_write","for(int i=0;i<a.Length;i++){a[i]=i;}return n;",false),
                ("goto","int i=0;label:while(i<n){i++;goto label;}return i;",false),
                ("switch","while(n>0){switch(n){case 1:break;default:n--;break;}break;}return n;",false),
            };
            var rows=new List<object>();
            foreach(var item in cases) {
                stage=item.Id;
                string source="namespace Business;public static class Entry{public static int Run(int n,int[] a,string s){"+item.Body+"}}\n";
                var input=new PracticalCapturedInput(PracticalCapturedInputKind.Source,"src/Entry.cs",Encoding.UTF8.GetBytes(source));
                var selection=new PracticalSourceSelection(CSharpPracticalCapture.SelectionSchema,"data",new[]{"src/Entry.cs"},new[]{root},Array.Empty<string>());
                try {
                    byte[] bytes=CSharpPracticalLoopLowering.Capture(selection,new[]{input},references);
                    if(!item.Accept)throw new Exception("unexpected_accept");
                    if(!bytes.SequenceEqual(CSharpPracticalLoopLowering.Capture(selection,new[]{input},references)))throw new Exception("determinism");
                    if(item.Id=="while") {
                        var changed=(byte[])bytes.Clone();changed[changed.Length-2]=(byte)' ';
                        bool rejected=false;try{CSharpPracticalLoopLowering.ValidateCandidate(selection,new[]{input},references,changed);}catch(PracticalCaptureFailure){rejected=true;}
                        if(!rejected)throw new Exception("candidate_mismatch");
                    }
                    var compilation=CSharpCompilation.Create("Runtime"+item.Id,new[]{CSharpSyntaxTree.ParseText(source,new CSharpParseOptions((LanguageVersion)1400))},references,new CSharpCompilationOptions(OutputKind.DynamicallyLinkedLibrary,checkOverflow:true));
                    using var stream=new MemoryStream();if(!compilation.Emit(stream).Success)throw new Exception("runtime_compile");
                    var method=Assembly.Load(stream.ToArray()).GetType("Business.Entry")!.GetMethod("Run")!;
                    var runs=new List<object>();
                    foreach(int n in new[]{0,1,2,6})foreach(int[] a in new[]{Array.Empty<int>(),new[]{-1,0,2,2,3},new[]{4,1,3}})foreach(string s in new[]{"","A😀日本"}) {
                        int? value=null;string error="";
                        try { value=(int)method.Invoke(null,new object[]{n,(int[])a.Clone(),s})!; }
                        catch(TargetInvocationException ex) {error=ex.InnerException!.GetType().Name;}
                        runs.Add(new {n,a,s,value,error});
                    }
                    rows.Add(new{id=item.Id,source,root,accepted=true,lowering=JsonSerializer.Deserialize<JsonElement>(bytes),runs,diagnostic="",code=""});
                } catch(PracticalCaptureFailure e) {
                    if(item.Accept)throw new Exception(e.Family+":"+e.Code);
                    if(e.ArtifactCount!=0)throw new Exception("artifact");
                    rows.Add(new{id=item.Id,source,root,accepted=false,lowering=(object?)null,runs=Array.Empty<object>(),diagnostic=e.Family.ToString(),code=e.Code});
                }
            }
            foreach(var variant in new[]{"set_add","set_count","map_add","map_replace"}) {
                stage=variant;bool map=variant.StartsWith("map",StringComparison.Ordinal),count=variant=="set_count";
                string wrapper=map?"Map":"Set",wrapperId=PracticalIdentity.SourceTypeId("Business",map?"Map":"Set");
                string prefix=map?"public readonly struct Pair{public readonly int Key;public readonly int Value;public Pair(int key,int value){Key=key;Value=value;}}":"";
                prefix+="public sealed class "+wrapper+"{public readonly "+(map?"Pair":"int")+"[] Items;public "+wrapper+"("+(map?"Pair":"int")+"[] items){Items=items;}}";
                string name=count?"Count":variant=="map_replace"?"Replace":"Add";
                string source="namespace Business;"+prefix+"public static class Entry{public static "+(count?"uint":wrapper)+" "+name+"("+wrapper+" collection"+(count?"":",int key"+(map?",int value":""))+"){"+(map?"new Pair(key,value);":"")+"new "+wrapper+"(collection.Items);int i=0;while(i<collection.Items.Length){i++;}return "+(count?"(uint)i":"collection")+";}}\n";
                var parameters=new List<string>{wrapperId};if(!count){parameters.Add(integer);if(map)parameters.Add(integer);}
                string selected=PracticalIdentity.CallableId("method","Business",PracticalIdentity.SourceTypeId("Business","Entry"),name,parameters,count?PracticalIdentity.PrimitiveId("u32"):wrapperId);
                var input=new PracticalCapturedInput(PracticalCapturedInputKind.Source,"src/Entry.cs",Encoding.UTF8.GetBytes(source));
                var selection=new PracticalSourceSelection(CSharpPracticalCapture.SelectionSchema,"data",new[]{"src/Entry.cs"},new[]{selected},Array.Empty<string>());
                var bytes=CSharpPracticalLoopLowering.Capture(selection,new[]{input},references);
                if(!bytes.SequenceEqual(CSharpPracticalLoopLowering.Capture(selection,new[]{input},references)))throw new Exception("collection_determinism");
                rows.Add(new{id=variant,source,root=selected,accepted=true,lowering=JsonSerializer.Deserialize<JsonElement>(bytes),runs=Array.Empty<object>(),diagnostic="",code=""});
            }
            File.WriteAllBytes("loop-source-cases.json",JsonSerializer.SerializeToUtf8Bytes(rows));return 0;
        } catch(Exception error) {Console.Error.WriteLine(stage+":"+error.Message);return 1;}
    }
}
