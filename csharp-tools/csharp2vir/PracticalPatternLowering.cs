using System;
using System.Collections.Generic;
using System.Linq;
using Microsoft.CodeAnalysis;
using Microsoft.CodeAnalysis.CSharp;
using Microsoft.CodeAnalysis.CSharp.Syntax;
using Microsoft.CodeAnalysis.Operations;

namespace Mpk.CSharp2Vir;

// W03 extends the private register CFG. Total getter claims are retained inputs,
// not proofs: T06-W04 must discharge them before decision-graph acceptance.
internal static partial class CSharpPracticalLoopLowering
{
    internal static void ValidatePatternCandidate(PracticalSourceSelection selection,
        IEnumerable<PracticalCapturedInput> inputs,System.Collections.Immutable.ImmutableArray<MetadataReference> references,
        IReadOnlyList<string> totalGetters,ReadOnlySpan<byte> candidate) {
        if(!candidate.SequenceEqual(Capture(selection,inputs,references,allowPatterns:true,totalGetters:totalGetters)))throw Fail("pattern_candidate_mismatch");
    }
    private static string GetterId(IPropertySymbol p,CSharpCompilation c) => PracticalIdentity.CallableId(
        "method",p.ContainingNamespace.ToDisplayString(),PracticalIdentity.SourceTypeId(p.ContainingNamespace.ToDisplayString(),p.ContainingType.Name),
        "get_"+p.Name,Array.Empty<string>(),PracticalExactTypeNormalizer.Normalize(p.Type,c).Id);
    private static void ValidatePatterns(CSharpCompilation compilation,IReadOnlyList<string> totalGetters)
    {
        var used=new HashSet<string>(StringComparer.Ordinal);
        if(totalGetters.Distinct(StringComparer.Ordinal).Count()!=totalGetters.Count)throw Fail("pattern_getter_claim");
        foreach(var tree in compilation.SyntaxTrees) {
            var model=compilation.GetSemanticModel(tree);
            foreach(var syntax in tree.GetRoot().DescendantNodes()) {
                if(syntax is GotoStatementSyntax or PositionalPatternClauseSyntax or SlicePatternSyntax)throw Fail("pattern_source_form");
                if(model.GetOperation(syntax) is not IPatternOperation pattern)continue;
                if(pattern is not (IConstantPatternOperation or IDiscardPatternOperation or IDeclarationPatternOperation
                    or ITypePatternOperation or IRelationalPatternOperation or IBinaryPatternOperation or INegatedPatternOperation
                    or IRecursivePatternOperation or IListPatternOperation))throw Fail("pattern_kind");
                // Normalization resolves exact closed scalar/enum/string/nullable
                // and immutable structural families; object/dynamic/open reject.
                PracticalExactTypeNormalizer.Normalize(pattern.InputType,compilation, topLevelNullability:pattern.InputType.IsReferenceType?NullableAnnotation.Annotated:NullableAnnotation.None);
                PracticalExactTypeNormalizer.Normalize(pattern.NarrowedType,compilation, topLevelNullability:pattern.NarrowedType.IsReferenceType?NullableAnnotation.NotAnnotated:NullableAnnotation.None);
                if(pattern is IListPatternOperation list && (list.InputType is not IArrayTypeSymbol {Rank:1,IsSZArray:true}
                    || list.Patterns.Any(p=>p is ISlicePatternOperation)))throw Fail("pattern_list");
                if(pattern is IRecursivePatternOperation recursive) {
                    if(recursive.DeconstructionSubpatterns.Length!=0 || recursive.DeconstructSymbol is not null)throw Fail("pattern_deconstruction");
                    foreach(var property in recursive.PropertySubpatterns) {
                        var pending=new Stack<IOperation>();pending.Push(property.Member);
                        while(pending.Count!=0) {
                            var member=pending.Pop();
                            if(member is IPropertyReferenceOperation access) {
                                var p=access.Property;
                                if(p.Name=="Length" && (access.Instance?.Type is IArrayTypeSymbol || access.Instance?.Type?.SpecialType==SpecialType.System_String))continue;
                                if(p.IsStatic||p.IsIndexer||p.IsVirtual||p.IsOverride||p.GetMethod is null
                                    || !SymbolEqualityComparer.Default.Equals(p.ContainingAssembly,compilation.Assembly))throw Fail("pattern_property");
                                string id=GetterId(p,compilation);used.Add(id);
                                if(!totalGetters.Contains(id,StringComparer.Ordinal))throw Fail("pattern_getter_totality");
                            } else if(member is IFieldReferenceOperation field && (!field.Field.IsReadOnly && !field.Field.IsConst || field.Field.IsStatic))throw Fail("pattern_field");
                            foreach(var child in member.ChildOperations)pending.Push(child);
                        }
                    }
                }
            }
        }
        if(!used.SetEquals(totalGetters))throw Fail("pattern_getter_claim");
    }
    private sealed partial class Builder
    {
        private readonly bool allowPatterns;
        private readonly Dictionary<ILabelSymbol,Node> switchExits=new(SymbolEqualityComparer.Default);
        private Node Decision(Op op,string value) {
            var node=Append("pattern_decision",op);node.inputs=new[]{value};return node;
        }
        private void Test(string value,Node yes,Node no,Op op) {
            var branch=Append("branch",op);branch.inputs=new[]{value};branch.successors=new[]{yes.id,no.id};current=null;
        }
        private void Bind(IPatternOperation pattern,string value) {
            var op=Get(pattern);
            if(op.Symbol.Length!=0)Eval("pattern_bind",op,new[]{value},slot:op.Symbol);
        }
        private void Match(IPatternOperation pattern,string value,Node yes,Node no) {
            var op=Get(pattern);
            switch(pattern) {
                case IDiscardPatternOperation:Link(current!,yes);current=null;return;
                case IDeclarationPatternOperation declaration:
                    if(!declaration.MatchesNull) {
                        var matched=New("jump");Test(Eval("pattern_type",op,new[]{value}),matched,no,op);current=matched;
                    }
                    Bind(pattern,value);Link(current!,yes);current=null;return;
                case ITypePatternOperation:Test(Eval("pattern_type",op,new[]{value}),yes,no,op);return;
                case IConstantPatternOperation constant:
                    Test(Eval("pattern_equal",op,new[]{value,Expression(Get(constant.Value))}),yes,no,op);return;
                case IRelationalPatternOperation relational:
                    Test(Eval("pattern_relational",op,new[]{value,Expression(Get(relational.Value))}),yes,no,op);return;
                case INegatedPatternOperation negated:Match(negated.Pattern,value,no,yes);return;
                case IBinaryPatternOperation binary:
                    var next=New("jump");bool and=binary.OperatorKind==BinaryOperatorKind.And;
                    Match(binary.LeftPattern,value,and?next:yes,and?no:next);current=next;Match(binary.RightPattern,value,yes,no);return;
                case IListPatternOperation list: {
                    var nonnull=New("jump");Test(Eval("pattern_not_null",op,new[]{value}),nonnull,no,op);current=nonnull;
                    var sized=New("jump");Test(Eval("pattern_length",op,new[]{value},slot:list.Patterns.Length.ToString(System.Globalization.CultureInfo.InvariantCulture)),sized,no,op);current=sized;
                    for(int i=0;i<list.Patterns.Length;i++) {
                        string element=Eval("pattern_element",op,new[]{value},slot:i.ToString(System.Globalization.CultureInfo.InvariantCulture));
                        var nextElement=New("jump");Match(list.Patterns[i],element,nextElement,no);current=nextElement;
                    }
                    Bind(pattern,value);Link(current!,yes);current=null;return;
                }
                case IRecursivePatternOperation recursive: {
                    var matched=New("jump");Test(Eval("pattern_type",op,new[]{value}),matched,no,op);current=matched;
                    foreach(var property in recursive.PropertySubpatterns) {
                        string member=PatternMember(property.Member,value,no);
                        var nextProperty=New("jump");Match(property.Pattern,member,nextProperty,no);current=nextProperty;
                    }
                    Bind(pattern,value);Link(current!,yes);current=null;return;
                }
                default:throw Fail("pattern_kind");
            }
        }
        private string PatternMember(IOperation member,string value,Node no) {
            if(member is IInstanceReferenceOperation)return value;
            IOperation? instance=member switch {IPropertyReferenceOperation p=>p.Instance,IFieldReferenceOperation f=>f.Instance,_=>throw Fail("pattern_member")};
            if(instance is null)throw Fail("pattern_static_member");
            string receiver=PatternMember(instance,value,no);
            if(instance is not IInstanceReferenceOperation) {
                var present=New("jump");Test(Eval("pattern_not_null",Get(instance),new[]{receiver}),present,no,Get(instance));current=present;
            }
            return Eval("pattern_member",Get(member),new[]{receiver});
        }
        private string IsPattern(Op op) {
            var expression=(IIsPatternOperation)op.Source!;string value=Expression(Get(expression.Value));Decision(op,value);
            var yes=New("jump");var no=New("jump");var join=New("jump");string result=Temp();
            Match(expression.Pattern,value,yes,no);
            current=yes;Store(result,Eval("pattern_true",op,Array.Empty<string>()),op);Link(current!,join);
            current=no;Store(result,Eval("pattern_false",op,Array.Empty<string>()),op);Link(current!,join);
            current=join;return Load(result,op);
        }
        private string SwitchExpression(Op op) {
            var expression=(ISwitchExpressionOperation)op.Source!;string value=Expression(Get(expression.Value));Decision(op,value);
            var join=New("jump");string result=Temp();
            foreach(var arm in expression.Arms) {
                var matched=New("jump");var next=New("jump");Match(arm.Pattern,value,matched,next);current=matched;
                if(arm.Guard is not null){var guarded=New("jump");Test(Expression(Get(arm.Guard)),guarded,next,Get(arm));current=guarded;}
                Store(result,Expression(Get(arm.Value)),op);Link(current!,join);current=next;
            }
            // The same exact closed exception is retained even when Roslyn
            // reports exhaustive. An unreachable fallback is not a proof.
            var unmatched=Append("throw",op);unmatched.slot="System.Runtime.CompilerServices.SwitchExpressionException";current=join;
            return Load(result,op);
        }
        private void SwitchStatement(Op op) {
            var statement=(ISwitchOperation)op.Source!;string value=Expression(Get(statement.Value));Decision(op,value);
            var exit=New("jump");switchExits.Add(statement.ExitLabel,exit);
            var sections=statement.Cases.Select(c=>(Case:c,Entry:New("jump"))).ToArray();Node? defaultTarget=null;
            foreach(var section in sections)foreach(var clause in section.Case.Clauses) {
                if(clause is IDefaultCaseClauseOperation){defaultTarget=section.Entry;continue;}
                var next=New("jump");
                if(clause is ISingleValueCaseClauseOperation single)Test(Eval("pattern_equal",Get(clause),new[]{value,Expression(Get(single.Value))}),section.Entry,next,Get(clause));
                else if(clause is IPatternCaseClauseOperation pattern) {
                    var matched=New("jump");Match(pattern.Pattern,value,matched,next);current=matched;
                    if(pattern.Guard is null){Link(current!,section.Entry);current=null;}
                    else Test(Expression(Get(pattern.Guard)),section.Entry,next,Get(clause));
                } else throw Fail("pattern_case");
                current=next;
            }
            Link(current!,defaultTarget??exit);
            foreach(var section in sections){current=section.Entry;foreach(var child in section.Case.Body)Statement(Get(child));if(current is not null)Link(current,exit);}
            switchExits.Remove(statement.ExitLabel);current=exit;
        }
    }
}
