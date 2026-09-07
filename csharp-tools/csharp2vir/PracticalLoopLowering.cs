using System;
using System.Collections.Generic;
using System.Collections.Immutable;
using System.Linq;
using System.Text.Json;
using Microsoft.CodeAnalysis;
using Microsoft.CodeAnalysis.CSharp;
using Microsoft.CodeAnalysis.Operations;

namespace Mpk.CSharp2Vir;

// W02's private, typed register/slot CFG. W06 owns SSA/VIR publication and
// independent whole-control import. Every expression operand is evaluated in
// source order; slots are explicit local state, never hidden source execution.
internal static class CSharpPracticalLoopLowering
{
    internal sealed class Node
    {
        public string id { get; set; } = "";
        public string kind { get; set; } = "";
        public int? source_ordinal { get; set; }
        public string operation { get; set; } = "";
        public string[] inputs { get; set; } = Array.Empty<string>();
        public string result { get; set; } = "";
        public string slot { get; set; } = "";
        public string[] successors { get; set; } = Array.Empty<string>();
        public string[] exceptional_successors { get; set; } = Array.Empty<string>();
    }
    internal sealed record Region(string loop_id, string? parent, string header, string body,
        string continue_target, string exit, string[] backedges);
    internal sealed record Function(string callable_id, JsonElement[] operations, Node[] nodes, Region[] loops);
    internal sealed record Op(int Ordinal, JsonElement Fact, IOperation? Source, Op[] Children)
    {
        internal string Kind => Fact.GetProperty("kind").GetString()!;
        internal string Symbol => Fact.GetProperty("symbol").GetString()!;
    }
    internal static byte[] Capture(PracticalSourceSelection selection, IEnumerable<PracticalCapturedInput> supplied,
        ImmutableArray<MetadataReference> references)
    {
        try {
            var inputs=supplied.ToArray();
            var facts=JsonSerializer.Deserialize<JsonElement>(CSharpPracticalLoopContracts.Capture(selection,inputs,references));
            var sequence=CSharpPracticalSequences.Validate(selection,inputs,references,allowLoopControl:true);
            var syntax=sequence.Arrays.Construction.Data.Syntax;
            var functions=new List<Function>();
            int closureBlocks=0;
            void CountBlock() { if(checked(++closureBlocks)>8192)throw PracticalFailures.Limit("cfg_blocks_per_closure"); }
            foreach(var callable in syntax.Callables) {
                var method=facts.GetProperty("methods").EnumerateArray().Single(m=>m.GetProperty("callable_id").GetString()==callable.Id);
                // Loop-free bodies retain their T03 normalized representation;
                // W06 composes them with these control bodies at VIR emission.
                if(method.GetProperty("loops").GetArrayLength()==0)continue;
                var source=syntax.SourceClosure.Sources.Single(s=>s.Path==method.GetProperty("source_path").GetString());
                functions.Add(new Builder(callable,method,source,CountBlock).Build());
            }
            return JsonSerializer.SerializeToUtf8Bytes(new {
                schema="mpk.csharp_practical.t04_w02.loop_lowering.v1", facts,
                normalized_syntax_sha256=syntax.SemanticSha256,
                normalized_syntax_utf8=System.Text.Encoding.UTF8.GetString(syntax.CopyCanonicalBytes()),
                sequence_handoff=JsonSerializer.Deserialize<JsonElement>(sequence.CopyCanonicalBytes()),
                functions,
            });
        } catch(PracticalCaptureFailure) { throw; }
        catch(Exception) { throw PracticalFailures.Protocol("loop_lowering_capture"); }
    }
    internal static void ValidateCandidate(PracticalSourceSelection selection,
        IEnumerable<PracticalCapturedInput> inputs,ImmutableArray<MetadataReference> references,
        ReadOnlySpan<byte> candidate)
    {
        if(!candidate.SequenceEqual(Capture(selection,inputs,references)))throw Fail("candidate_mismatch");
    }
    private static PracticalCaptureFailure Fail(string code) =>
        new(8,PracticalDiagnosticFamily.CSHARP_PRACTICAL_LOWERING,"loop_lowering_"+code);
    private sealed class Builder
    {
        private readonly PracticalNormalizedCallable callable;
        private readonly JsonElement method;
        private readonly PracticalSourceFile source;
        private readonly JsonElement[] operations;
        private readonly Action countBlock;
        private readonly Dictionary<IOperation,Op> original=new();
        private readonly List<Node> nodes=new();
        private readonly List<Region> regions=new();
        private readonly Stack<(ILoopOperation Loop,string Id,Node Continue,Node Exit)> loops=new();
        private readonly List<Op> roots=new();
        private Node? current;
        private int nextValue,nextSlot;
        private Node? exception;
        internal Builder(PracticalNormalizedCallable callable,JsonElement method,PracticalSourceFile source,Action countBlock)
        {
            this.callable=callable;this.method=method;this.source=source;this.countBlock=countBlock;
            operations=JsonSerializer.Deserialize<JsonElement[]>(callable.CopyBodyBytes())!;
            int offset=0;
            Op Read(int depth) {
                if(depth>512||offset>=operations.Length)throw Fail("operation_tree");
                int ordinal=offset++;var fact=operations[ordinal];
                var children=Enumerable.Range(0,fact.GetProperty("child_count").GetInt32()).Select(_=>Read(depth+1)).ToArray();
                var node=new Op(ordinal,fact,callable.OperationNodes[ordinal],children);
                if(node.Source is not null)original.Add(node.Source,node);
                return node;
            }
            while(offset<operations.Length)roots.Add(Read(0));
        }
        private Node New(string kind,Op? op=null) {
            if(nodes.Count>=1024)throw PracticalFailures.Limit("cfg_blocks_per_method");
            countBlock();
            var node=new Node{id=callable.Id+".node."+nodes.Count.ToString("D6",System.Globalization.CultureInfo.InvariantCulture),kind=kind,source_ordinal=op?.Ordinal};nodes.Add(node);return node;
        }
        private void Link(Node from,Node to) { if(from.successors.Length!=0)throw Fail("duplicate_edge");from.successors=new[]{to.id}; }
        private Node Append(string kind,Op? op=null) {
            if(current is null)throw Fail("unreachable_emission");
            var node=New(kind,op);Link(current,node);current=node;return node;
        }
        private string Temp()=>"temporary:"+(nextSlot++).ToString(System.Globalization.CultureInfo.InvariantCulture);
        private string Eval(string operation,Op? op,string[] inputs,bool mayThrow=false,string slot="") {
            var node=Append("evaluate",op);node.operation=operation;node.inputs=inputs;node.slot=slot;
            node.result=callable.Id+".value."+(nextValue++).ToString("D6",System.Globalization.CultureInfo.InvariantCulture);
            if(mayThrow) { exception??=New("throw");node.exceptional_successors=new[]{exception.id}; }
            return node.result;
        }
        private string Load(string slot,Op? op=null)=>Eval("load",op,Array.Empty<string>(),slot:slot);
        private string Store(string slot,string value,Op? op=null)=>Eval("store",op,new[]{value},slot:slot);
        private Op Get(IOperation operation)=>original.TryGetValue(operation,out var op)?op:throw Fail("source_operand");
        internal Function Build() {
            current=New("entry");
            foreach(var root in roots)Statement(root);
            if(current is not null) { var end=Append("return");current=null; }
            // Empty joins after two abrupt arms have no incoming edge. Remove
            // them and regenerate IDs without changing any executable edge.
            var retained=nodes.Where(n=>n.kind!="unreachable").ToArray();
            var ids=retained.Select((n,i)=>(n.id,NewId:callable.Id+".node."+i.ToString("D6",System.Globalization.CultureInfo.InvariantCulture))).ToDictionary(p=>p.id,p=>p.NewId,StringComparer.Ordinal);
            foreach(var node in retained) {node.id=ids[node.id];node.successors=node.successors.Select(id=>ids[id]).ToArray();node.exceptional_successors=node.exceptional_successors.Select(id=>ids[id]).ToArray();}
            var mapped=regions.Select(r=>new Region(r.loop_id,r.parent,ids[r.header],ids[r.body],ids[r.continue_target],ids[r.exit],r.backedges.Select(id=>ids[id]).ToArray())).OrderBy(r=>r.loop_id,StringComparer.Ordinal).ToArray();
            return new(callable.Id,operations,retained,mapped);
        }
        private void Statement(Op op) {
            if(current is null)return; // only a structurally unreachable suffix
            switch(op.Kind) {
                case "Block":case "VariableDeclarationGroup":case "VariableDeclaration":
                    foreach(var child in op.Children)Statement(child);break;
                case "VariableDeclarator":
                    if(op.Children.Length!=0)Store(op.Symbol,Expression(op.Children[0]),op);break;
                case "ExpressionStatement":Expression(op.Children.Single());break;
                case "Return":
                    var result=op.Children.Select(Expression).ToArray();var returned=Append("return",op);returned.inputs=result;current=null;break;
                case "Conditional":
                    var test=Expression(op.Children[0]);var branch=Append("branch",op);branch.inputs=new[]{test};
                    Node yes=New("jump"),no=New("jump"),join=New("jump");branch.successors=new[]{yes.id,no.id};
                    current=yes;Statement(op.Children[1]);bool yesLive=current is not null;if(yesLive)Link(current!,join);
                    current=no;if(op.Children.Length>2)Statement(op.Children[2]);bool noLive=current is not null;if(noLive)Link(current!,join);
                    if(!yesLive&&!noLive)join.kind="unreachable";
                    current=yesLive||noLive?join:null;break;
                case "Loop":case "ForLoop":case "WhileLoop":case "ForEachLoop":Loop(op);break;
                case "Branch":
                    var abrupt=(IBranchOperation)op.Source!;
                    if(loops.Count==0||abrupt.BranchKind is not(BranchKind.Break or BranchKind.Continue))throw Fail("abrupt_target");
                    var target=loops.Peek();bool isBreak=abrupt.BranchKind==BranchKind.Break;
                    if(!SymbolEqualityComparer.Default.Equals(abrupt.Target,isBreak?target.Loop.ExitLabel:target.Loop.ContinueLabel))throw Fail("abrupt_target");
                    var edge=Append(isBreak?"break":"continue",op);edge.slot=target.Id;Link(edge,isBreak?target.Exit:target.Continue);current=null;break;
                case "Empty":break;
                // Switch/exception source remains with its serial owner.
                case "Switch":case "SwitchExpression":case "Try":case "Throw":case "Labeled":throw Fail("later_control_owner");
                default:Expression(op);break;
            }
        }
        private void Loop(Op op) {
            var loop=op.Source as ILoopOperation??throw Fail("loop_shape");
            var row=method.GetProperty("loops").EnumerateArray().Single(l=>l.GetProperty("start_byte").GetInt32()==source.ByteOffset(loop.Syntax.SpanStart));
            string id=row.GetProperty("loop_id").GetString()!;
            string? parent=loops.Count==0?null:loops.Peek().Id;
            if(loop is IForLoopOperation f)foreach(var before in f.Before)Statement(Get(before));
            string collection="",index="",length="";
            if(loop is IForEachLoopOperation each) {
                if(each.IsAsynchronous||each.Locals.Length!=1)throw Fail("foreach_shape");
                collection=Store(Temp(),Expression(Get(each.Collection)),op);
                length=Store(Temp(),Eval("length",op,new[]{collection},true),op);
                index=Temp();Store(index,Eval("zero",op,Array.Empty<string>()),op);
            }
            // A true structural header keeps invariant entry before all guard
            // effects. The source guard has its own explicit branch in the
            // region; this also represents do without testing it on entry.
            var entered=Eval("true",op,Array.Empty<string>());
            var header=Append("loop_header",op);header.slot=id;header.inputs=new[]{entered};
            Node body=New("jump"),tail=New("jump"),exit=New("jump");
            loops.Push((loop,id,tail,exit));
            bool bottom=loop is IWhileLoopOperation {ConditionIsTop:false};
            if(bottom)Link(header,body);
            else {
                current=header;
                string condition=loop switch {
                    IForEachLoopOperation=>Eval("less",op,new[]{Load(index),length}),
                    IForLoopOperation {Condition:not null} conditional=>Expression(Get(conditional.Condition)),
                    IWhileLoopOperation {Condition:not null} conditional=>Expression(Get(conditional.Condition)),
                    _=>Eval("true",op,Array.Empty<string>()),
                };
                var branch=Append("branch",op);branch.inputs=new[]{condition};branch.successors=new[]{body.id,exit.id};
            }
            current=body;
            if(loop is IForEachLoopOperation iteration) {
                var control=Get(iteration.LoopControlVariable);
                string value=Eval("element",op,new[]{collection,Load(index)},true);
                Store(control.Symbol,Eval("iteration_convert",control,new[]{value},true),control);
            }
            Statement(Get(loop.Body));if(current is not null)Link(current,tail);
            current=tail;
            if(loop is IForLoopOperation after)foreach(var increment in after.AtLoopBottom)Statement(Get(increment));
            if(loop is IForEachLoopOperation)Store(index,Eval("increment",op,new[]{Load(index)}),op);
            string backedge;
            if(bottom) {
                var condition=Expression(Get(((IWhileLoopOperation)loop).Condition!));
                var branch=Append("branch",op);branch.inputs=new[]{condition};branch.successors=new[]{header.id,exit.id};backedge=branch.id;
            } else { Link(current!,header);backedge=current!.id; }
            header.successors=header.successors.Concat(new[]{exit.id}).ToArray();
            loops.Pop();regions.Add(new(id,parent,header.id,header.successors[0],tail.id,exit.id,new[]{backedge}));current=exit;
        }
        private string Conditional(Op op,Op test,Op yes,Op? no,bool shortCircuit=false) {
            string condition=Expression(test),slot=Temp();var branch=Append("branch",op);branch.inputs=new[]{condition};
            Node left=New("jump"),right=New("jump"),join=New("jump");branch.successors=new[]{left.id,right.id};
            bool and=op.Source is IBinaryOperation {OperatorKind:BinaryOperatorKind.ConditionalAnd};
            current=left;Store(slot,shortCircuit&&!and?condition:Expression(yes),op);Link(current!,join);
            current=right;Store(slot,shortCircuit?(and?condition:Expression(yes)):Expression(no!),op);Link(current!,join);
            current=join;return Load(slot,op);
        }
        private (string Slot,string[] Address) Address(Op target) {
            if(target.Kind is "LocalReference" or "ParameterReference")return(target.Symbol,Array.Empty<string>());
            if(target.Kind=="ArrayElementReference")return("",target.Children.Select(Expression).ToArray());
            throw Fail("assignment_target");
        }
        private string Read((string Slot,string[] Address) address,Op op)=>address.Slot.Length!=0?Load(address.Slot,op):Eval("element",op,address.Address,true);
        private string Write((string Slot,string[] Address) address,string value,Op op)=>address.Slot.Length!=0?Store(address.Slot,value,op):Eval("update",op,address.Address.Concat(new[]{value}).ToArray(),true);
        private string Expression(Op op) {
            switch(op.Kind) {
                case "LocalReference":case "ParameterReference":return Load(op.Symbol,op);
                case "VariableInitializer":case "Argument":case "Parenthesized":return Expression(op.Children.Single());
                case "Literal":case "DefaultValue":return Eval("constant",op,Array.Empty<string>());
                case "SimpleAssignment": {var address=Address(op.Children[0]);var value=Expression(op.Children[1]);return Write(address,value,op);}
                case "CompoundAssignment": {var address=Address(op.Children[0]);var left=Read(address,op);var right=Expression(op.Children[1]);return Write(address,Eval("binary",op,new[]{left,right},true),op);}
                case "Increment":case "Decrement": {var address=Address(op.Children.Single());var prior=Read(address,op);var value=Eval("unary_update",op,new[]{prior},true);Write(address,value,op);return ((IIncrementOrDecrementOperation)op.Source!).IsPostfix?prior:value;}
                case "Binary":
                    if(op.Source is IBinaryOperation {OperatorKind:BinaryOperatorKind.ConditionalAnd or BinaryOperatorKind.ConditionalOr})return Conditional(op,op.Children[0],op.Children[1],null,true);
                    return Eval("binary",op,op.Children.Select(Expression).ToArray(),true);
                case "Unary":return Eval("unary",op,op.Children.Select(Expression).ToArray(),true);
                case "Conversion":return Eval("convert",op,op.Children.Select(Expression).ToArray(),true);
                case "Conditional":return Conditional(op,op.Children[0],op.Children[1],op.Children[2]);
                case "ArrayElementReference":return Eval("element",op,op.Children.Select(Expression).ToArray(),true);
                case "ArrayCreation": {
                    var creation=(IArrayCreationOperation)op.Source!;
                    if(creation.DimensionSizes.Length!=1)throw Fail("array_rank");
                    var size=Expression(Get(creation.DimensionSizes[0]));var value=Eval("allocate",op,new[]{size},true);
                    if(creation.Initializer is not null)foreach(var element in creation.Initializer.ElementValues) {
                        var index=Eval("initializer_index",Get(element),Array.Empty<string>(),slot:Array.IndexOf(creation.Initializer.ElementValues.ToArray(),element).ToString(System.Globalization.CultureInfo.InvariantCulture));
                        Eval("update",op,new[]{value,index,Expression(Get(element))},true);
                    }
                    return value;
                }
                case "PropertyReference":case "FieldReference":return Eval("member",op,op.Children.Select(Expression).ToArray(),true);
                case "InstanceReference":return Load("this",op);
                case "Invocation":case "ObjectCreation":return Eval(op.Kind=="Invocation"?"call":"construct",op,op.Children.Select(Expression).ToArray(),true);
                default:throw Fail("expression_shape");
            }
        }
    }
}
