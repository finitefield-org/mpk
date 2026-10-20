package main

import (
	"encoding/json"
	"fmt"
)

const (
	virSchema         = "mpk.vir.v1"
	virHashDomain     = "MPK-VIR-1.0"
	contractDomain    = "MPK-CONTRACT-1.0"
	sourceMapSchema   = "mpk.source_map.v1"
	sourceMapDomain   = "MPK-SOURCE-MAP-1.0"
	maximumVIRBytes   = 201_326_592
	maximumMapBytes   = 33_554_432
	maximumMapEntries = 323_728
)

type virModule struct {
	Schema             string             `json:"schema"`
	SourceLanguage     string             `json:"source_language"`
	SemanticProfile    string             `json:"semantic_profile"`
	SemanticParameters semanticParameters `json:"semantic_parameters"`
	Units              []virUnit          `json:"units"`
	VIRHash            string             `json:"vir_hash"`
}

type virUnit struct {
	ID         string         `json:"id"`
	Name       string         `json:"name"`
	TypeDecls  []virTypeDecl  `json:"type_decls"`
	ConstDecls []virConstDecl `json:"const_decls"`
	Functions  []virFunction  `json:"functions"`
}

type virTypeDecl struct {
	ID     string     `json:"id"`
	Name   string     `json:"name"`
	Fields []virField `json:"fields"`
}

type virField struct {
	Name string  `json:"name"`
	Type virType `json:"type"`
}

type virConstDecl struct {
	ID    string     `json:"id"`
	Name  string     `json:"name"`
	Type  virType    `json:"type"`
	Value virLiteral `json:"value"`
}

type virType struct {
	Kind    string   `json:"kind"`
	Width   int64    `json:"width,omitempty"`
	Signed  *bool    `json:"signed,omitempty"`
	Length  int64    `json:"length,omitempty"`
	Element *virType `json:"element,omitempty"`
	ID      string   `json:"id,omitempty"`
}

type virBinding struct {
	ID   string  `json:"id"`
	Type virType `json:"type"`
}

type virFunction struct {
	ID             string                       `json:"id"`
	UnitID         string                       `json:"unit_id"`
	Name           string                       `json:"name"`
	Params         []virBinding                 `json:"params"`
	Results        []virBinding                 `json:"results"`
	Locals         []virBinding                 `json:"locals"`
	Blocks         []virBlock                   `json:"blocks"`
	Contracts      virContract                  `json:"contracts"`
	FeaturesUsed   []string                     `json:"features_used"`
	origin         sourceOrigin                 `json:"-"`
	loopHeaders    []string                     `json:"-"`
	loopParameters map[string]map[string]string `json:"-"`
}

type virBlock struct {
	Label        string           `json:"label"`
	Parameters   []virBinding     `json:"parameters"`
	Instructions []virInstruction `json:"instructions"`
	Terminator   virTerminator    `json:"terminator"`
}

type virInstruction struct {
	ID           string           `json:"id"`
	Kind         string           `json:"kind"`
	Op           string           `json:"op,omitempty"`
	Type         virType          `json:"type"`
	Target       string           `json:"target,omitempty"`
	Value        *virValue        `json:"value,omitempty"`
	Base         *virValue        `json:"base,omitempty"`
	Index        *virValue        `json:"index,omitempty"`
	Field        string           `json:"field,omitempty"`
	Fields       []virNamedValue  `json:"fields,omitempty"`
	Elements     []virValue       `json:"elements,omitempty"`
	LHS          *virValue        `json:"lhs,omitempty"`
	RHS          *virValue        `json:"rhs,omitempty"`
	Function     string           `json:"function,omitempty"`
	ContractHash string           `json:"contract_hash,omitempty"`
	Args         []virValue       `json:"args,omitempty"`
	SafetyChecks []virSafetyCheck `json:"safety_checks"`
	origin       sourceOrigin     `json:"-"`
}

type virNamedValue struct {
	Name  string   `json:"name"`
	Value virValue `json:"value"`
}

type virSafetyCheck struct {
	Kind string `json:"kind"`
}

type virTerminator struct {
	Kind      string       `json:"kind"`
	Values    []virValue   `json:"values,omitempty"`
	Cond      *virValue    `json:"cond,omitempty"`
	Label     string       `json:"label,omitempty"`
	Args      []virValue   `json:"args,omitempty"`
	ThenLabel string       `json:"then_label,omitempty"`
	ThenArgs  []virValue   `json:"then_args,omitempty"`
	ElseLabel string       `json:"else_label,omitempty"`
	ElseArgs  []virValue   `json:"else_args,omitempty"`
	origin    sourceOrigin `json:"-"`
}

type virValue struct {
	Var   string      `json:"var,omitempty"`
	Const string      `json:"const,omitempty"`
	Bool  *bool       `json:"bool,omitempty"`
	Int   *virInteger `json:"int,omitempty"`
}

type virLiteral struct {
	Bool *bool       `json:"bool,omitempty"`
	Int  *virInteger `json:"int,omitempty"`
}

type virInteger struct {
	Value  string `json:"value"`
	Width  int64  `json:"width"`
	Signed bool   `json:"signed"`
}

type virContract struct {
	UnitID             string             `json:"unit_id"`
	FunctionID         string             `json:"function_id"`
	SemanticProfile    string             `json:"semantic_profile"`
	SemanticParameters semanticParameters `json:"semantic_parameters"`
	Requires           []virContractExpr  `json:"requires"`
	Ensures            []virContractExpr  `json:"ensures"`
	Modifies           []string           `json:"modifies"`
	Panic              string             `json:"panic"`
	Termination        string             `json:"termination"`
	Loops              []virLoopContract  `json:"loops"`
	ContractHash       string             `json:"contract_hash"`
}

type virContractExpr struct {
	Op     string            `json:"op,omitempty"`
	Args   []virContractExpr `json:"args,omitempty"`
	LHS    *virContractExpr  `json:"lhs,omitempty"`
	RHS    *virContractExpr  `json:"rhs,omitempty"`
	Value  *virContractExpr  `json:"value,omitempty"`
	Type   *virType          `json:"type,omitempty"`
	Var    string            `json:"var,omitempty"`
	Result *int64            `json:"result,omitempty"`
	Bool   *bool             `json:"bool,omitempty"`
	Int    *virInteger       `json:"int,omitempty"`
}

type virLoopContract struct {
	Header     string            `json:"header"`
	Invariants []virContractExpr `json:"invariants"`
	Decreases  []virContractExpr `json:"decreases"`
}

type sourceOrigin struct {
	Kind           string `json:"kind"`
	InputKind      string `json:"input_kind,omitempty"`
	NormalizedPath string `json:"normalized_path,omitempty"`
	Start          int64  `json:"start,omitempty"`
	End            int64  `json:"end,omitempty"`
	Reason         string `json:"reason,omitempty"`
}

type sourceMapReference struct {
	Kind        string `json:"kind"`
	UnitID      string `json:"unit_id"`
	FunctionID  string `json:"function_id"`
	Block       string `json:"block,omitempty"`
	Instruction string `json:"instruction,omitempty"`
}

type sourceMapEntry struct {
	Reference sourceMapReference `json:"reference"`
	Origin    sourceOrigin       `json:"origin"`
}

type sourceMap struct {
	Schema         string           `json:"schema"`
	SourceIRSchema string           `json:"source_ir_schema"`
	SourceIRHash   string           `json:"source_ir_hash"`
	Entries        []sourceMapEntry `json:"entries"`
	SourceMapHash  string           `json:"source_map_hash"`
}

func boolPointer(value bool) *bool { return &value }

func (value virType) MarshalJSON() ([]byte, error) {
	switch value.Kind {
	case "bool":
		return json.Marshal(struct {
			Kind string `json:"kind"`
		}{Kind: value.Kind})
	case "bv":
		if value.Signed == nil {
			return nil, fmt.Errorf("BV type lacks signedness")
		}
		return json.Marshal(struct {
			Kind   string `json:"kind"`
			Width  int64  `json:"width"`
			Signed bool   `json:"signed"`
		}{value.Kind, value.Width, *value.Signed})
	case "array":
		if value.Element == nil {
			return nil, fmt.Errorf("array type lacks element")
		}
		return json.Marshal(struct {
			Kind    string  `json:"kind"`
			Length  int64   `json:"length"`
			Element virType `json:"element"`
		}{value.Kind, value.Length, *value.Element})
	case "struct":
		return json.Marshal(struct {
			Kind string `json:"kind"`
			ID   string `json:"id"`
		}{value.Kind, value.ID})
	default:
		return nil, fmt.Errorf("unknown VIR type kind %q", value.Kind)
	}
}

func (value virInstruction) MarshalJSON() ([]byte, error) {
	type base struct {
		ID     string           `json:"id"`
		Kind   string           `json:"kind"`
		Type   virType          `json:"type"`
		Safety []virSafetyCheck `json:"safety_checks"`
	}
	prefix := base{value.ID, value.Kind, value.Type, value.SafetyChecks}
	switch value.Kind {
	case "Const":
		return json.Marshal(struct {
			base
			Value *virValue `json:"value"`
		}{prefix, value.Value})
	case "Copy":
		return json.Marshal(struct {
			base
			Target string    `json:"target"`
			Value  *virValue `json:"value"`
		}{prefix, value.Target, value.Value})
	case "BinOp":
		return json.Marshal(struct {
			base
			Op  string    `json:"op"`
			LHS *virValue `json:"lhs"`
			RHS *virValue `json:"rhs"`
		}{prefix, value.Op, value.LHS, value.RHS})
	case "UnaryOp":
		return json.Marshal(struct {
			base
			Op    string    `json:"op"`
			Value *virValue `json:"value"`
		}{prefix, value.Op, value.Value})
	case "Convert":
		return json.Marshal(struct {
			base
			Value *virValue `json:"value"`
		}{prefix, value.Value})
	case "Field":
		return json.Marshal(struct {
			base
			Base  *virValue `json:"base"`
			Field string    `json:"field"`
		}{prefix, value.Base, value.Field})
	case "Index":
		return json.Marshal(struct {
			base
			Base  *virValue `json:"base"`
			Index *virValue `json:"index"`
		}{prefix, value.Base, value.Index})
	case "MakeStruct":
		return json.Marshal(struct {
			base
			Fields []virNamedValue `json:"fields"`
		}{prefix, nonNilNamedValues(value.Fields)})
	case "MakeArray":
		return json.Marshal(struct {
			base
			Elements []virValue `json:"elements"`
		}{prefix, nonNilValues(value.Elements)})
	case "CallStatic":
		return json.Marshal(struct {
			base
			Function     string     `json:"function"`
			ContractHash string     `json:"contract_hash"`
			Args         []virValue `json:"args"`
		}{prefix, value.Function, value.ContractHash, nonNilValues(value.Args)})
	default:
		return nil, fmt.Errorf("unknown VIR instruction kind %q", value.Kind)
	}
}

func (value virTerminator) MarshalJSON() ([]byte, error) {
	switch value.Kind {
	case "Return":
		return json.Marshal(struct {
			Kind   string     `json:"kind"`
			Values []virValue `json:"values"`
		}{value.Kind, nonNilValues(value.Values)})
	case "Jump":
		return json.Marshal(struct {
			Kind  string     `json:"kind"`
			Label string     `json:"label"`
			Args  []virValue `json:"args"`
		}{value.Kind, value.Label, nonNilValues(value.Args)})
	case "Branch":
		return json.Marshal(struct {
			Kind      string     `json:"kind"`
			Cond      *virValue  `json:"cond"`
			ThenLabel string     `json:"then_label"`
			ThenArgs  []virValue `json:"then_args"`
			ElseLabel string     `json:"else_label"`
			ElseArgs  []virValue `json:"else_args"`
		}{value.Kind, value.Cond, value.ThenLabel, nonNilValues(value.ThenArgs), value.ElseLabel, nonNilValues(value.ElseArgs)})
	default:
		return nil, fmt.Errorf("unknown VIR terminator kind %q", value.Kind)
	}
}

func (value sourceOrigin) MarshalJSON() ([]byte, error) {
	switch value.Kind {
	case "source":
		return json.Marshal(struct {
			Kind           string `json:"kind"`
			InputKind      string `json:"input_kind"`
			NormalizedPath string `json:"normalized_path"`
			Start          int64  `json:"start"`
			End            int64  `json:"end"`
		}{value.Kind, value.InputKind, value.NormalizedPath, value.Start, value.End})
	case "synthetic":
		return json.Marshal(struct {
			Kind   string `json:"kind"`
			Reason string `json:"reason"`
		}{value.Kind, value.Reason})
	default:
		return nil, fmt.Errorf("unknown source origin kind %q", value.Kind)
	}
}

func nonNilValues(values []virValue) []virValue {
	if values == nil {
		return []virValue{}
	}
	return values
}
func nonNilNamedValues(values []virNamedValue) []virNamedValue {
	if values == nil {
		return []virNamedValue{}
	}
	return values
}

func defaultVIRContract(unitID, functionID string) virContract {
	value := true
	return virContract{
		UnitID: unitID, FunctionID: functionID,
		SemanticProfile:    goSemanticProfile,
		SemanticParameters: semanticParameters{TargetID: goTarget, PointerWidth: goPointerWidth},
		Requires:           []virContractExpr{},
		Ensures:            []virContractExpr{{Bool: &value}},
		Modifies:           []string{}, Panic: "forbidden", Termination: "total",
		Loops: []virLoopContract{}, ContractHash: zeroSHA256(),
	}
}


const (
	successorProfileRegistrySchema = "mpk.semantic_profile.registry.v1"
	successorProfileRegistryID = "mpk.semantic_profile.registry.v1"
	successorProfileRegistryRevision = int64(3)
	successorProfileRegistryRevisionArgument = "3"
	successorProfileRegistrySHA256 = "fc102411ac266a38db27f904df2ca6f794bca1a216fff12377d88990e653c557"
	successorGoProfileEntrySHA256 = "b10ec338d1f2b3fefc015e4d46c27def43e92ff3d87341624b48c93db951ca96"
	successorSemanticParametersSchema = "mpk.semantic_parameters.go_fixed.v0"
	successorSelectionSchema = "mpk.selection.go_function.v0"
)

type successorProfileRegistryIdentity struct {
	Schema string `json:"schema"`
	ID string `json:"id"`
	Revision int64 `json:"revision"`
	RegistrySHA256 string `json:"registry_sha256"`
}

type successorSemanticParametersEnvelope struct {
	Schema string `json:"schema"`
	Value semanticParameters `json:"value"`
}

type successorSemanticContext struct {
	ProfileRegistry successorProfileRegistryIdentity `json:"profile_registry"`
	ProfileEntrySHA256 string `json:"profile_entry_sha256"`
	SourceLanguage string `json:"source_language"`
	SemanticProfile string `json:"semantic_profile"`
	SemanticParameters successorSemanticParametersEnvelope `json:"semantic_parameters"`
}

type successorSelectionEnvelope struct {
	Schema string `json:"schema"`
	Value goSelection `json:"value"`
}

func fixedSuccessorSemanticContext() successorSemanticContext {
	return successorSemanticContext{
		ProfileRegistry: successorProfileRegistryIdentity{
			Schema: successorProfileRegistrySchema,
			ID: successorProfileRegistryID,
			Revision: successorProfileRegistryRevision,
			RegistrySHA256: successorProfileRegistrySHA256,
		},
		ProfileEntrySHA256: successorGoProfileEntrySHA256,
		SourceLanguage: "go",
		SemanticProfile: goSemanticProfile,
		SemanticParameters: successorSemanticParametersEnvelope{
			Schema: successorSemanticParametersSchema,
			Value: semanticParameters{TargetID: goTarget, PointerWidth: goPointerWidth},
		},
	}
}

func successorSelection(value goSelection) successorSelectionEnvelope {
	return successorSelectionEnvelope{Schema: successorSelectionSchema, Value: value}
}

func marshalSuccessor(value any) ([]byte, error) {
	return json.Marshal(value)
}

func (value virModule) MarshalJSON() ([]byte, error) {
	type successorVIR struct {
		Schema string `json:"schema"`
		SemanticContext successorSemanticContext `json:"semantic_context"`
		Units []virUnit `json:"units"`
		VIRHash string `json:"vir_hash"`
	}
	return marshalSuccessor(successorVIR{
		Schema: value.Schema,
		SemanticContext: fixedSuccessorSemanticContext(),
		Units: value.Units,
		VIRHash: value.VIRHash,
	})
}

func (value virContract) MarshalJSON() ([]byte, error) {
	type successorContract struct {
		SemanticContext successorSemanticContext `json:"semantic_context"`
		UnitID string `json:"unit_id"`
		FunctionID string `json:"function_id"`
		Requires []virContractExpr `json:"requires"`
		Ensures []virContractExpr `json:"ensures"`
		Modifies []string `json:"modifies"`
		Panic string `json:"panic"`
		Termination string `json:"termination"`
		Loops []virLoopContract `json:"loops"`
		ContractHash string `json:"contract_hash"`
	}
	return marshalSuccessor(successorContract{
		SemanticContext: fixedSuccessorSemanticContext(),
		UnitID: value.UnitID,
		FunctionID: value.FunctionID,
		Requires: value.Requires,
		Ensures: value.Ensures,
		Modifies: value.Modifies,
		Panic: value.Panic,
		Termination: value.Termination,
		Loops: value.Loops,
		ContractHash: value.ContractHash,
	})
}

func (value sourceMap) MarshalJSON() ([]byte, error) {
	type successorMap struct {
		Schema string `json:"schema"`
		SemanticContext successorSemanticContext `json:"semantic_context"`
		SourceIRSchema string `json:"source_ir_schema"`
		SourceIRHash string `json:"source_ir_hash"`
		Entries []sourceMapEntry `json:"entries"`
		SourceMapHash string `json:"source_map_hash"`
	}
	return marshalSuccessor(successorMap{
		Schema: value.Schema,
		SemanticContext: fixedSuccessorSemanticContext(),
		SourceIRSchema: value.SourceIRSchema,
		SourceIRHash: value.SourceIRHash,
		Entries: value.Entries,
		SourceMapHash: value.SourceMapHash,
	})
}
