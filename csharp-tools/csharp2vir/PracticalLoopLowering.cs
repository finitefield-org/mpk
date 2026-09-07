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
internal static partial class CSharpPracticalLoopLowering
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
    internal sealed record Function(string callable_id, JsonElement[] operations, Node[] nodes, Region[] loops,
        [property:System.Text.Json.Serialization.JsonIgnore(Condition=System.Text.Json.Serialization.JsonIgnoreCondition.WhenWritingNull)] object[]? operation_locations=null);
    internal sealed record Op(int Ordinal, JsonElement Fact, IOperation? Source, Op[] Children)
    {
        internal string Kind => Fact.GetProperty("kind").GetString()!;
        internal string Symbol => Fact.GetProperty("symbol").GetString()!;
    }
    internal static byte[] Capture(PracticalSourceSelection selection, IEnumerable<PracticalCapturedInput> supplied,
        ImmutableArray<MetadataReference> references, bool allowPatterns = false, IReadOnlyList<string>? totalGetters = null, bool allowExceptions = false, bool allowHandlers = false)
    {
        try {
            var inputs=supplied.ToArray();
            CSharpCompilation? exceptionCompilation=null;
            var facts=JsonSerializer.Deserialize<JsonElement>(CSharpPracticalLoopContracts.Capture(selection,inputs,references,allowPatterns,allowExceptions,allowHandlers));
            var sequence=CSharpPracticalSequences.Validate(selection,inputs,references,allowLoopControl:true, allowPatternControl:allowPatterns, allowExceptionControl:allowExceptions, allowHandlers:allowHandlers,
                validatedCompilation:c=>{exceptionCompilation=c;if(allowPatterns)ValidatePatterns(c,totalGetters??Array.Empty<string>());});
            return CaptureValidated(sequence, facts, exceptionCompilation!, allowPatterns, totalGetters, allowExceptions, allowHandlers);
        } catch(PracticalCaptureFailure) { throw; }
        catch(Exception) { throw PracticalFailures.Protocol("loop_lowering_capture"); }
    }
    internal static byte[] CaptureValidated(PracticalSequences sequence, JsonElement facts,
        CSharpCompilation exceptionCompilation, bool allowPatterns,
        IReadOnlyList<string>? totalGetters, bool allowExceptions, bool allowHandlers, bool ssaConditions = false)
    {
        if(allowPatterns)ValidatePatterns(exceptionCompilation,totalGetters??Array.Empty<string>());
            var syntax=sequence.Arrays.Construction.Data.Syntax;
            var functions=new List<Function>();var handlers=new List<object>();
            int closureBlocks=0;
            void CountBlock() { if(checked(++closureBlocks)>8192)throw PracticalFailures.Limit("cfg_blocks_per_closure"); }
            foreach(var callable in syntax.Callables) {
                var method=facts.GetProperty("methods").EnumerateArray().Single(m=>m.GetProperty("callable_id").GetString()==callable.Id);
                // Loop-free bodies retain their T03 normalized representation;
                // W06 composes them with these control bodies at VIR emission.
                if(!allowHandlers && !(allowExceptions && callable.OperationNodes.Any(o=>o is IThrowOperation)) && method.GetProperty("loops").GetArrayLength()==0 && !(allowPatterns && callable.OperationNodes.Any(o=>o is ISwitchOperation or ISwitchExpressionOperation or IIsPatternOperation)))continue;
                var source=syntax.SourceClosure.Sources.Single(s=>s.Path==method.GetProperty("source_path").GetString());
                var builder=new Builder(callable,method,source,CountBlock,allowPatterns,allowExceptions,allowHandlers,ssaConditions,sequence.Arrays.Steps);
                functions.Add(builder.Build());if(allowHandlers)handlers.Add(builder.HandlerGraph());
            }
            if(allowHandlers)return JsonSerializer.SerializeToUtf8Bytes(new {
                schema=ssaConditions?"mpk.csharp_practical.t04_w06.control_lowering.v1":"mpk.csharp_practical.t04_w05.handler_lowering.v1",facts,
                normalized_syntax_sha256=syntax.SemanticSha256,
                normalized_syntax_utf8=System.Text.Encoding.UTF8.GetString(syntax.CopyCanonicalBytes()),
                sequence_handoff=JsonSerializer.Deserialize<JsonElement>(sequence.CopyCanonicalBytes()),functions,handlers,
                total_getters=(totalGetters??Array.Empty<string>()).OrderBy(x=>x,StringComparer.Ordinal).ToArray(),
                exception_definitions=ExceptionDefinitions(exceptionCompilation!,sequence.Arrays.Construction.Data.Types),
            });
            if(allowExceptions)return JsonSerializer.SerializeToUtf8Bytes(new {
                schema="mpk.csharp_practical.t04_w04.exception_lowering.v1", facts,
                normalized_syntax_sha256=syntax.SemanticSha256,
                normalized_syntax_utf8=System.Text.Encoding.UTF8.GetString(syntax.CopyCanonicalBytes()),
                sequence_handoff=JsonSerializer.Deserialize<JsonElement>(sequence.CopyCanonicalBytes()),functions,
                total_getters=(totalGetters??Array.Empty<string>()).OrderBy(x=>x,StringComparer.Ordinal).ToArray(),
                exception_definitions=ExceptionDefinitions(exceptionCompilation!,sequence.Arrays.Construction.Data.Types),
            });
            if(allowPatterns)return JsonSerializer.SerializeToUtf8Bytes(new {
                schema="mpk.csharp_practical.t04_w03.pattern_lowering.v1", facts,
                normalized_syntax_sha256=syntax.SemanticSha256,
                normalized_syntax_utf8=System.Text.Encoding.UTF8.GetString(syntax.CopyCanonicalBytes()),
                sequence_handoff=JsonSerializer.Deserialize<JsonElement>(sequence.CopyCanonicalBytes()),functions,
                total_getters=(totalGetters??Array.Empty<string>()).OrderBy(x=>x,StringComparer.Ordinal).ToArray(),
            });
            return JsonSerializer.SerializeToUtf8Bytes(new {
                schema="mpk.csharp_practical.t04_w02.loop_lowering.v1", facts,
                normalized_syntax_sha256=syntax.SemanticSha256,
                normalized_syntax_utf8=System.Text.Encoding.UTF8.GetString(syntax.CopyCanonicalBytes()),
                sequence_handoff=JsonSerializer.Deserialize<JsonElement>(sequence.CopyCanonicalBytes()),
                functions,
            });
    }
    internal static void ValidateCandidate(PracticalSourceSelection selection,
        IEnumerable<PracticalCapturedInput> inputs,ImmutableArray<MetadataReference> references,
        ReadOnlySpan<byte> candidate)
    {
        if(!candidate.SequenceEqual(Capture(selection,inputs,references)))throw Fail("candidate_mismatch");
    }

    internal static void ValidateExceptionCandidate(PracticalSourceSelection selection,
        IEnumerable<PracticalCapturedInput> inputs,ImmutableArray<MetadataReference> references,
        IReadOnlyList<string> totalGetters,ReadOnlySpan<byte> candidate)
    {
        if(!candidate.SequenceEqual(Capture(selection,inputs,references,true,totalGetters,true)))throw Fail("exception_candidate_mismatch");
    }
    internal static void ValidateHandlerCandidate(PracticalSourceSelection selection,
        IEnumerable<PracticalCapturedInput> inputs,ImmutableArray<MetadataReference> references,
        IReadOnlyList<string> totalGetters,ReadOnlySpan<byte> candidate)
    {
        if(!candidate.SequenceEqual(Capture(selection,inputs,references,true,totalGetters,true,true)))throw Fail("handler_candidate_mismatch");
    }
    private static object[] ExceptionDefinitions(CSharpCompilation compilation,IReadOnlyList<PracticalDataType> types)
    {
        var result=new List<(string Id,object Value)>();
        foreach(var tree in compilation.SyntaxTrees)foreach(var declaration in tree.GetRoot().DescendantNodes().OfType<Microsoft.CodeAnalysis.CSharp.Syntax.TypeDeclarationSyntax>()) {
            var symbol=compilation.GetSemanticModel(tree).GetDeclaredSymbol(declaration)!;
            if(!CSharpPracticalCapture.IsExceptionBase(symbol.BaseType))continue;
            string id=PracticalIdentity.SourceTypeId(symbol.ContainingNamespace.ToDisplayString(),symbol.Name);
            var type=types.Single(t=>t.Id==id);
            result.Add((id,new {type_id=id,sealed_type=symbol.IsSealed,direct_base_type_id="System.Exception",payload_member_names=type.Members.Where(m=>m.Stored).Select(m=>m.Name).ToArray()}));
        }
        return result.OrderBy(r=>r.Id,StringComparer.Ordinal).Select(r=>r.Value).ToArray();
    }
    private static PracticalCaptureFailure Fail(string code) =>
        new(8,PracticalDiagnosticFamily.CSHARP_PRACTICAL_LOWERING,"loop_lowering_"+code);
    private sealed partial class Builder
    {
        private readonly bool allowExceptions;
        private void ExplicitThrow(Op op)
        {
            var thrown=(IThrowOperation)op.Source!;
            if(allowHandlers && thrown.Exception is null) {
                var active=handlerStack.LastOrDefault(f=>f.zone=="catch")??throw Fail("inactive_rethrow");
                var rethrow=Append("rethrow",op);rethrow.slot=active.catch_id;
                rethrow.exceptional_successors=new[]{Search(op).id};current=null;return;
            }
            IOperation operand=thrown.Exception!;
            while(operand is IConversionOperation {IsImplicit:true} conversion)operand=conversion.Operand;
            var creation=operand as IObjectCreationOperation??throw Fail("exception_operand");
            var source=Get(creation);
            bool builtin=CSharpPracticalCapture.IsClosedBuiltinException(creation.Type);
            string type=builtin?creation.Type!.ToDisplayString(SymbolDisplayFormat.CSharpErrorMessageFormat):PracticalIdentity.SourceTypeId(creation.Type!.ContainingNamespace.ToDisplayString(),creation.Type.Name);
            // The source payload uses the T03 constructor transaction. The
            // exception wrapper is a tagged value, never a CLR allocation.
            var payload=builtin?Array.Empty<string>():new[]{Expression(source)};
            string value=Eval("closed_exception",source,payload,slot:type);
            var edge=Append("explicit_throw",op);edge.inputs=new[]{value};edge.slot=type;
            var exit=allowHandlers?Search(op):New("exception_exit",op);if(!allowHandlers)exit.slot=type;edge.exceptional_successors=new[]{exit.id};current=null;
        }
        private readonly PracticalNormalizedCallable callable;
        private readonly JsonElement method;
        private readonly PracticalSourceFile source;
        private readonly JsonElement[] operations;
        private readonly Action countBlock;
        private readonly bool ssaConditions;
        private readonly Dictionary<IOperation,Op> original=new();
        private readonly List<Node> nodes=new();
        private readonly List<Region> regions=new();
        private readonly Stack<(ILoopOperation Loop,string Id,Node Continue,Node Exit)> loops=new();
        private readonly List<Op> roots=new();
        private Node? current;
        private int nextValue,nextSlot;
        private Node? exception;
        internal Builder(PracticalNormalizedCallable callable,JsonElement method,PracticalSourceFile source,Action countBlock,bool allowPatterns,bool allowExceptions,bool allowHandlers,bool ssaConditions = false,IReadOnlyList<PracticalArrayStep>? arraySteps=null)
        {
            aliasPublications=(arraySteps??Array.Empty<PracticalArrayStep>()).Where(s=>s.Operation=="alias_freeze").Select(s=>s.Site).ToHashSet(StringComparer.Ordinal);
            this.ssaConditions=ssaConditions;this.allowHandlers=allowHandlers;this.allowExceptions=allowExceptions;this.callable=callable;this.method=method;this.source=source;this.countBlock=countBlock;this.allowPatterns=allowPatterns;
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
            var node=new Node{id=callable.Id+".node."+nodes.Count.ToString("D6",System.Globalization.CultureInfo.InvariantCulture),kind=kind,source_ordinal=op?.Ordinal};nodes.Add(node);if(allowHandlers)contexts.Add(node,handlerStack.ToArray());return node;
        }
        private void Link(Node from,Node to) { if(from.successors.Length!=0)throw Fail("duplicate_edge");from.successors=new[]{to.id}; }
        private Node Append(string kind,Op? op=null) {
            if(current is null)throw Fail("unreachable_emission");
            var node=New(kind,op);Link(current,node);current=node;return node;
        }
        private readonly HashSet<string> aliasPublications;
        private bool AliasPublication(Op op)=>ssaConditions && op.Source is not null && aliasPublications.Contains(op.Source.Syntax.SyntaxTree.FilePath+":"+op.Source.Syntax.SpanStart.ToString(System.Globalization.CultureInfo.InvariantCulture)+":"+op.Source.Kind);
        private string Temp()=>"temporary:"+(nextSlot++).ToString(System.Globalization.CultureInfo.InvariantCulture);
        private string Eval(string operation,Op? op,string[] inputs,bool mayThrow=false,string slot="") {
            var node=Append("evaluate",op);node.operation=operation;node.inputs=inputs;node.slot=slot;
            node.result=callable.Id+".value."+(nextValue++).ToString("D6",System.Globalization.CultureInfo.InvariantCulture);
            if(mayThrow) { if(allowHandlers)node.exceptional_successors=new[]{Search(op).id};else{exception??=New("throw");node.exceptional_successors=new[]{exception.id};} }
            return node.result;
        }
        private string Load(string slot,Op? op=null)=>Eval("load",op,Array.Empty<string>(),slot:slot);
        private string Store(string slot,string value,Op? op=null)=>Eval("store",op,new[]{value},slot:slot);
        private Op Get(IOperation operation)=>original.TryGetValue(operation,out var op)?op:throw Fail("source_operand");
        internal Function Build() {
            current=New("entry");
            foreach(var root in roots)Statement(root);
            if(current is not null) { var end=Append("return");if(allowHandlers)Complete(end,"return",null);current=null; }
            // Empty joins after two abrupt arms have no incoming edge. Remove
            // them and regenerate IDs without changing any executable edge.
            var retained=nodes.Where(n=>n.kind!="unreachable").ToArray();
            var ids=retained.Select((n,i)=>(n.id,NewId:callable.Id+".node."+i.ToString("D6",System.Globalization.CultureInfo.InvariantCulture))).ToDictionary(p=>p.id,p=>p.NewId,StringComparer.Ordinal);
            foreach(var node in retained) {node.id=ids[node.id];node.successors=node.successors.Select(id=>ids[id]).ToArray();node.exceptional_successors=node.exceptional_successors.Select(id=>ids[id]).ToArray();}
            var mapped=regions.Select(r=>new Region(r.loop_id,r.parent,ids[r.header],ids[r.body],ids[r.continue_target],ids[r.exit],r.backedges.Select(id=>ids[id]).ToArray())).OrderBy(r=>r.loop_id,StringComparer.Ordinal).ToArray();
            if(allowHandlers)FinishHandlers(retained);
            object[]? locations=ssaConditions?callable.OperationNodes.Select(operation=>(object)new {
                start_byte=operation is null?method.GetProperty("start_byte").GetInt32():source.ByteOffset(operation.Syntax.SpanStart),
                end_byte=operation is null?method.GetProperty("end_byte").GetInt32():source.ByteOffset(operation.Syntax.Span.End),
            }).ToArray():null;
            return new(callable.Id,operations,retained,mapped,locations);
        }
        private void Statement(Op op) {
            if(current is null)return; // only a structurally unreachable suffix
            switch(op.Kind) {
                case "Block":case "VariableDeclarationGroup":case "VariableDeclaration":
                    foreach(var child in op.Children)Statement(child);break;
                case "VariableDeclarator":
                    if(op.Children.Length!=0){var initial=op.Children[0];while(initial.Kind=="VariableInitializer")initial=initial.Children.Single();string value=Expression(initial);if(AliasPublication(op))value=Eval("publish",initial,new[]{value});Store(op.Symbol,value,op);}break;
                case "ExpressionStatement":Expression(op.Children.Single());break;
                case "Return":
                    var result=op.Children.Select(Expression).ToArray();var returned=Append("return",op);returned.inputs=result;if(allowHandlers)Complete(returned,"return",null);current=null;break;
                case "Conditional":
                    Node yes,no,join;
                    if(ssaConditions) {
                        yes=New("jump");no=New("jump");join=New("jump");Condition(op.Children[0],yes,no);
                    } else {
                        var test=Expression(op.Children[0]);var branch=Append("branch",op);branch.inputs=new[]{test};
                        yes=New("jump");no=New("jump");join=New("jump");branch.successors=new[]{yes.id,no.id};
                    }
                    current=yes;Statement(op.Children[1]);bool yesLive=current is not null;if(yesLive)Link(current!,join);
                    current=no;if(op.Children.Length>2)Statement(op.Children[2]);bool noLive=current is not null;if(noLive)Link(current!,join);
                    if(!yesLive&&!noLive)join.kind="unreachable";
                    current=yesLive||noLive?join:null;break;
                case "Loop":case "ForLoop":case "WhileLoop":case "ForEachLoop":Loop(op);break;
                case "Branch":
                    var abrupt=(IBranchOperation)op.Source!;
                    if(allowPatterns && abrupt.BranchKind==BranchKind.Break && switchExits.TryGetValue(abrupt.Target,out var switchExit)) {
                        var switchBreak=Append("jump",op);Link(switchBreak,switchExit);if(allowHandlers)Complete(switchBreak,"normal",switchExit);current=null;break;
                    }
                    if(loops.Count==0||abrupt.BranchKind is not(BranchKind.Break or BranchKind.Continue))throw Fail("abrupt_target");
                    var target=loops.Peek();bool isBreak=abrupt.BranchKind==BranchKind.Break;
                    if(!SymbolEqualityComparer.Default.Equals(abrupt.Target,isBreak?target.Loop.ExitLabel:target.Loop.ContinueLabel))throw Fail("abrupt_target");
                    var edge=Append(isBreak?"break":"continue",op);edge.slot=target.Id;Link(edge,isBreak?target.Exit:target.Continue);if(allowHandlers)Complete(edge,isBreak?"break":"continue",isBreak?target.Exit:target.Continue);current=null;break;
                case "Try" when allowHandlers:TryStatement(op);break;
                case "ConstructorInitializer" when allowExceptions && op.Children.Length==1
                    && op.Children[0].Kind=="Invocation"
                    && op.Children[0].Symbol=="System.Runtime|System.Exception.Exception()"
                    && op.Children[0].Children.Length==1
                    && op.Children[0].Children[0].Kind=="InstanceReference":
                    // The capture owner has checked the sealed direct Exception
                    // declaration. Its parameterless base has no observable state.
                    break;
                case "Empty":break;
                // Switch/exception source remains with its serial owner.
                case "Throw" when allowExceptions:ExplicitThrow(op);break;
                case "Switch" when allowPatterns:SwitchStatement(op);break;
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
                var sourceCondition=loop switch {
                    IForLoopOperation conditional=>conditional.Condition,
                    IWhileLoopOperation conditional=>conditional.Condition,
                    _=>null,
                };
                if(ssaConditions && sourceCondition is not null)Condition(Get(sourceCondition),body,exit);
                else {
                string condition=loop switch {
                    IForEachLoopOperation=>Eval("less",op,new[]{Load(index),length}),
                    IForLoopOperation {Condition:not null} conditional=>Expression(Get(conditional.Condition)),
                    IWhileLoopOperation {Condition:not null} conditional=>Expression(Get(conditional.Condition)),
                    _=>Eval("true",op,Array.Empty<string>()),
                };
                var branch=Append("branch",op);branch.inputs=new[]{condition};branch.successors=new[]{body.id,exit.id};
                }
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
                if(ssaConditions) {
                    // A single join is the declared backedge even when the
                    // condition itself has several short-circuit exits.
                    var back=New("jump");Condition(Get(((IWhileLoopOperation)loop).Condition!),back,exit);
                    Link(back,header);backedge=back.id;
                } else {
                var condition=Expression(Get(((IWhileLoopOperation)loop).Condition!));
                var branch=Append("branch",op);branch.inputs=new[]{condition};branch.successors=new[]{header.id,exit.id};backedge=branch.id;
                }
            } else { Link(current!,header);backedge=current!.id; }
            header.successors=header.successors.Concat(new[]{exit.id}).ToArray();
            loops.Pop();regions.Add(new(id,parent,header.id,header.successors[0],tail.id,exit.id,new[]{backedge}));current=exit;
        }
        private string Conditional(Op op,Op test,Op yes,Op? no,bool shortCircuit=false) {
            if(ssaConditions && shortCircuit) {
                string result=Temp();var yesNode=New("jump");var noNode=New("jump");var joinNode=New("jump");
                Condition(op,yesNode,noNode);
                current=yesNode;Store(result,Eval("condition_true",op,Array.Empty<string>()),op);Link(current!,joinNode);
                current=noNode;Store(result,Eval("condition_false",op,Array.Empty<string>()),op);Link(current!,joinNode);
                current=joinNode;return Load(result,op);
            }
            if(ssaConditions && !shortCircuit) {
                string result=Temp();Node yesNode=New("jump"),noNode=New("jump"),resultJoin=New("jump");
                Condition(test,yesNode,noNode);
                current=yesNode;Store(result,Eval("join_value",op,new[]{Expression(yes)}),op);Link(current!,resultJoin);
                current=noNode;Store(result,Eval("join_value",op,new[]{Expression(no!)}),op);Link(current!,resultJoin);
                current=resultJoin;return Load(result,op);
            }
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
        private string InitializerTransaction(Op op) {
            var creation=(IObjectCreationOperation)op.Source!;
            var arguments=creation.Arguments.Select(argument=>{
                string value=Expression(Get(argument));
                return argument.Value.Type is IArrayTypeSymbol?Eval("publish",Get(argument.Value),new[]{value}):value;
            }).ToArray();
            string state=Eval("construction_begin",op,Array.Empty<string>());
            state=Eval("construction_invoke",op,new[]{state}.Concat(arguments).ToArray(),true);
            foreach(var item in creation.Initializer!.Initializers) {
                if(item is not ISimpleAssignmentOperation assignment)throw Fail("initializer_assignment");
                string value=Expression(Get(assignment.Value));
                state=Eval("construction_write",Get(assignment),new[]{state,value},slot:op.Ordinal.ToString(System.Globalization.CultureInfo.InvariantCulture));
            }
            return Eval("construction_finalize",op,new[]{state},true);
        }
        private string Expression(Op op) {
            if(allowPatterns && op.Kind=="FieldReference" && op.Source?.ConstantValue.HasValue==true)return Eval("pattern_constant",op,Array.Empty<string>());
            switch(op.Kind) {
                case "SwitchExpression" when allowPatterns:return SwitchExpression(op);
                case "IsPattern" when allowPatterns:return IsPattern(op);
                case "LocalReference":case "ParameterReference":return Load(op.Symbol,op);
                case "VariableInitializer":case "Argument":case "Parenthesized":return Expression(op.Children.Single());
                case "Literal":case "DefaultValue":return Eval("constant",op,Array.Empty<string>());
                case "SimpleAssignment": {
                    var target=op.Children[0];
                    if(allowExceptions && target.Kind is "FieldReference" or "PropertyReference"
                        && target.Children.Length==1 && target.Children[0].Source is IInstanceReferenceOperation
                        && op.Source!.Syntax.Ancestors().Any(n=>n is Microsoft.CodeAnalysis.CSharp.Syntax.ConstructorDeclarationSyntax)) {
                        var receiver=Expression(target.Children[0]);var assigned=Expression(op.Children[1]);
                        return Eval("construction_assign",op,new[]{receiver,assigned},slot:target.Symbol);
                    }
                    var address=Address(target);var value=Expression(op.Children[1]);if(AliasPublication(op))value=Eval("publish",op.Children[1],new[]{value});return Write(address,value,op);
                }
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
                case "PropertyReference":case "FieldReference":
                    if(allowHandlers && op.Children.Length==1 && op.Children[0].Source is ILocalReferenceOperation caught
                        && caught.Local.DeclaringSyntaxReferences.Any(r=>r.GetSyntax() is Microsoft.CodeAnalysis.CSharp.Syntax.CatchDeclarationSyntax))
                        return Eval("exception_payload",op,op.Children.Select(Expression).ToArray(),slot:op.Symbol);
                    return Eval("member",op,op.Children.Select(Expression).ToArray(),true);
                case "InstanceReference":return Load("this",op);
                case "ObjectCreation" when ssaConditions && op.Source is IObjectCreationOperation {Initializer:not null}:return InitializerTransaction(op);
                case "Invocation":case "ObjectCreation":
                    var arguments=op.Children.Select(child=>{
                        string value=Expression(child);
                        return ssaConditions && child.Source is IArgumentOperation argument && argument.Value.Type is IArrayTypeSymbol
                            ?Eval("publish",Get(argument.Value),new[]{value}):value;
                    }).ToArray();
                    return Eval(op.Kind=="Invocation"?"call":"construct",op,arguments,true);
                default:throw Fail("expression_shape");
            }
        }
    }
}

internal static partial class CSharpPracticalLoopLowering
{
    private sealed record HandlerFrame(string region,string zone,string catch_id);
    private sealed class HandlerCatch
    {
        internal string Id="",Type="",Local="";
        internal Node Entry=null!;
        internal Node? Filter;
    }
    private sealed class HandlerRegion
    {
        internal string Id="";
        internal int Ordinal;
        internal Node Try=null!,Exit=null!;
        internal Node? Finally;
        internal readonly List<HandlerCatch> Catches=new();
    }
    private sealed partial class Builder
    {
        private readonly bool allowHandlers;
        private readonly List<HandlerFrame> handlerStack=new();
        private readonly Dictionary<Node,HandlerFrame[]> contexts=new();
        private readonly List<HandlerRegion> handlerRegions=new();
        private readonly Dictionary<Node,(string Kind,Node? Target)> completions=new();
        private readonly List<object> transfers=new();
        private Node Search(Op? op)=>New("handler_search",op);
        private void Complete(Node node,string kind,Node? target) {
            node.kind="handler_completion";node.operation=kind;node.successors=Array.Empty<string>();
            completions.Add(node,(kind,target));
        }
        private void TryStatement(Op op) {
            var tried=(ITryOperation)op.Source!;
            var region=new HandlerRegion{Id=callable.Id+".handler."+handlerRegions.Count,Ordinal=op.Ordinal};handlerRegions.Add(region);
            region.Exit=New("jump");
            var before=current!;
            handlerStack.Add(new(region.Id,"try",""));region.Try=New("jump",op);Link(before,region.Try);current=region.Try;
            Statement(Get(tried.Body));
            if(current is not null){var leave=Append("handler_completion",op);Complete(leave,"normal",region.Exit);}
            handlerStack.RemoveAt(handlerStack.Count-1);
            foreach(var clause in tried.Catches) {
                var caught=new HandlerCatch{Id=region.Id+".catch."+region.Catches.Count,
                    Type=CSharpPracticalCapture.IsClosedBuiltinException(clause.ExceptionType)?clause.ExceptionType!.ToDisplayString(SymbolDisplayFormat.CSharpErrorMessageFormat):PracticalIdentity.SourceTypeId(clause.ExceptionType!.ContainingNamespace.ToDisplayString(),clause.ExceptionType.Name)};
                region.Catches.Add(caught);
                if(clause.ExceptionDeclarationOrExpression is not null)caught.Local=Get(clause.ExceptionDeclarationOrExpression).Symbol;
                if(clause.Filter is not null) {
                    handlerStack.Add(new(region.Id,"filter",caught.Id));caught.Filter=New("handler_filter_entry",Get(clause));current=caught.Filter;
                    var condition=Expression(Get(clause.Filter));var result=Append("handler_filter_result",Get(clause));result.inputs=new[]{condition};result.slot=caught.Id;
                    handlerStack.RemoveAt(handlerStack.Count-1);
                }
                handlerStack.Add(new(region.Id,"catch",caught.Id));caught.Entry=New("handler_entry",Get(clause));caught.Entry.slot=caught.Local;current=caught.Entry;
                Statement(Get(clause.Handler));
                if(current is not null){var leave=Append("handler_completion",op);Complete(leave,"normal",region.Exit);}
                handlerStack.RemoveAt(handlerStack.Count-1);
            }
            if(tried.Finally is not null) {
                handlerStack.Add(new(region.Id,"finally",""));region.Finally=New("handler_finally_entry",Get(tried.Finally));current=region.Finally;
                Statement(Get(tried.Finally));if(current is not null)Append("handler_resume",op);
                handlerStack.RemoveAt(handlerStack.Count-1);
            }
            current=region.Exit;
        }
        private void FinishHandlers(Node[] retained) {
            var byRegion=handlerRegions.ToDictionary(r=>r.Id,StringComparer.Ordinal);
            string[] Finalies(IEnumerable<HandlerFrame> frames)=>frames.Reverse().Where(f=>f.zone!="finally" && byRegion[f.region].Finally is not null).Select(f=>byRegion[f.region].Finally!.id).ToArray();
            // Each throwing operation has a dedicated search node. The table
            // lists every typed/filter alternative and its exact unwind suffix.
            // Filters are executed before any listed finally, never during unwind.
            foreach(var node in retained.Where(n=>n.kind=="handler_search")) {
                var frames=contexts[node];var candidates=new List<object>();var edges=new List<string>();
                for(int depth=frames.Length-1;depth>=0;depth--) {
                    var frame=frames[depth];if(frame.zone=="filter")break; // thrown filter => false at this boundary
                    if(frame.zone!="try")continue;
                    foreach(var caught in byRegion[frame.region].Catches) {
                        var unwind=Finalies(frames.Skip(depth+1));
                        candidates.Add(new{catch_id=caught.Id,type_id=caught.Type,filter=caught.Filter?.id,entry=caught.Entry.id,local=caught.Local,finally_entries=unwind});
                        edges.Add(caught.Filter?.id??unwind.FirstOrDefault()??caught.Entry.id);
                    }
                }
                string? filterCatch=frames.LastOrDefault(f=>f.zone=="filter")?.catch_id;
                var escaping=filterCatch is null?Finalies(frames):Array.Empty<string>();edges.AddRange(escaping);
                node.successors=edges.Distinct(StringComparer.Ordinal).ToArray();
                transfers.Add(new{node=node.id,kind="throw",target=(string?)null,candidates,finally_entries=escaping,filter_catch=filterCatch});
            }
            foreach(var pair in completions) {
                var frames=contexts[pair.Key];var destination=pair.Value.Target is null?Array.Empty<HandlerFrame>():contexts[pair.Value.Target];
                int common=0;while(common<frames.Length && common<destination.Length && frames[common]==destination[common])common++;
                if(frames.Skip(common).Any(f=>f.zone=="finally"))throw Fail("finally_abrupt");
                var unwind=Finalies(frames.Skip(common));var target=pair.Value.Target?.id;
                pair.Key.successors=unwind.Take(1).Concat(unwind.Length==0&&target is not null?new[]{target}:Array.Empty<string>()).ToArray();
                transfers.Add(new{node=pair.Key.id,kind=pair.Value.Kind,target,candidates=Array.Empty<object>(),finally_entries=unwind,filter_catch=(string?)null});
            }
            var continuations=handlerRegions.SelectMany(r=>r.Catches.SelectMany(c=>c.Filter is null?new[]{c.Entry.id}:new[]{c.Entry.id,c.Filter.id}).Concat(new[]{r.Exit.id}).Concat(r.Finally is null?Array.Empty<string>():new[]{r.Finally.id}))
                .Concat(completions.Values.Where(c=>c.Target is not null).Select(c=>c.Target!.id)).Distinct(StringComparer.Ordinal).OrderBy(x=>x,StringComparer.Ordinal).ToArray();
            foreach(var node in retained.Where(n=>n.kind is "handler_resume" or "handler_filter_result"))node.successors=continuations;
        }
        internal object HandlerGraph()=>new {
            callable_id=callable.Id,
            regions=handlerRegions.Select(r=>new{id=r.Id,source_ordinal=r.Ordinal,try_entry=r.Try.id,exit=r.Exit.id,finally_entry=r.Finally?.id,
                catches=r.Catches.Select(c=>new{id=c.Id,type_id=c.Type,local=c.Local,filter=c.Filter?.id,entry=c.Entry.id}).ToArray()}).ToArray(),
            contexts=contexts.Where(p=>p.Key.kind!="unreachable").Select(p=>new{node=p.Key.id,frames=p.Value}).ToArray(),transfers,
        };
    }
}
