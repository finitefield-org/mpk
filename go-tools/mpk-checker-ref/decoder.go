package mpkcheckerref

const (
	certMagic  = "MPKCERT"
	certFormat = "MPK-CERT-0.1"
	coreSpec   = "MPK-Core-0.1"
	hashLen    = 32
)

type HashBytes [hashLen]byte

type Certificate struct {
	Module             string
	Imports            []Import
	NameTable          []string
	LevelTable         []LevelNode
	TermTable          []TermNode
	ProofNodeTable     []ProofNode
	Declarations       []Declaration
	TheoryCertificates []TheoryCertificate
	ExportBlock        []ExportEntry
	AxiomReport        AxiomReport
	SourceManifest     *SourceManifest
	Hashes             CertificateHashes
}

type CertificateHashes struct {
	ExportHash      HashBytes
	AxiomReportHash HashBytes
	CertificateHash HashBytes
}

type Import struct {
	ModuleName      string
	ExportHash      HashBytes
	CertificateHash *HashBytes
}

type LevelTag uint8

const (
	LevelZero LevelTag = iota
	LevelSucc
	LevelMax
	LevelParam
)

type LevelNode struct {
	Tag LevelTag
	A   uint32
	B   uint32
}

type TermTag uint8

const (
	TermSort TermTag = iota
	TermVar
	TermConst
	TermApp
	TermLam
	TermPi
	TermLet
)

type TermNode struct {
	Tag       TermTag
	A         uint32
	B         uint32
	C         uint32
	Arguments []uint32
}

type ProofNodeTag uint8

const (
	ProofExact ProofNodeTag = iota
	ProofApply
	ProofIntro
	ProofLetProof
	ProofRefl
	ProofRewrite
	ProofEqRec
	ProofConstructor
	ProofRecursor
	ProofConv
	ProofTheory
)

type ProofNode struct {
	Tag               ProofNodeTag
	Term              uint32
	ExpectedType      uint32
	FunctionProof     uint32
	ArgumentProofs    []uint32
	DomainType        uint32
	BodyProof         uint32
	Value             uint32
	EqProof           uint32
	TargetProof       uint32
	Motive            uint32
	BaseProof         uint32
	Constructor       uint32
	Recursor          uint32
	MinorProofs       []uint32
	MajorProof        uint32
	Proof             uint32
	DefeqWitness      *uint32
	TheoryCertificate uint32
}

type DeclarationTag uint8

const (
	DeclAxiom DeclarationTag = iota
	DeclDef
	DeclTheorem
	DeclInductive
	DeclConstructor
	DeclRecursor
	DeclTheoryPrimitive
)

type Reducibility uint8

const (
	Reducible Reducibility = iota
	Opaque
)

type Declaration struct {
	Name         uint32
	Tag          DeclarationTag
	Type         uint32
	Value        uint32
	Proof        uint32
	Inductive    uint32
	Generated    bool
	Reducibility Reducibility
}

type TheoryCertificate struct {
	Format  string
	Payload []byte
}

type ExportEntry struct {
	Name            uint32
	Declaration     uint32
	DeclarationHash HashBytes
}

type AxiomCategory string

const (
	AxiomCategoryCore          AxiomCategory = "CoreAxiom"
	AxiomCategoryBuiltinTheory AxiomCategory = "BuiltinTheoryAxiom"
	AxiomCategoryGoSemantics   AxiomCategory = "GoSemanticsAxiom"
	AxiomCategoryExternal      AxiomCategory = "ExternalAxiom"
)

type AxiomReport struct {
	Entries                 []AxiomReportEntry
	DeclarationDependencies []DeclarationAxiomDependencies
	Summary                 AxiomReportSummary
}

type AxiomReportEntry struct {
	Category                        AxiomCategory
	Name                            string
	OriginModule                    string
	TypeHash                        HashBytes
	DeclarationHash                 HashBytes
	SourceCertificateHash           *HashBytes
	DirectDependentDeclarations     []uint32
	TransitiveDependentDeclarations []uint32
	ApprovalProfile                 *string
	ReviewerNote                    *string
}

type DeclarationAxiomDependencies struct {
	DeclarationName             string
	DeclarationHash             HashBytes
	DirectAxiomDependencies     []uint32
	TransitiveAxiomDependencies []uint32
}

type AxiomReportSummary struct {
	CoreAxiomCount          uint64
	BuiltinTheoryAxiomCount uint64
	GoSemanticsAxiomCount   uint64
	ExternalAxiomCount      uint64
	TotalAxiomCount         uint64
}

type SourceManifest struct {
	Payload []byte
}

type DecodeErrorKind string

const (
	DecodeUnexpectedEOF        DecodeErrorKind = "unexpected_eof"
	DecodeTrailingBytes        DecodeErrorKind = "trailing_bytes"
	DecodeInvalidMagic         DecodeErrorKind = "invalid_magic"
	DecodeInvalidFormat        DecodeErrorKind = "invalid_format"
	DecodeInvalidCoreSpec      DecodeErrorKind = "invalid_core_spec"
	DecodeNonMinimalVarint     DecodeErrorKind = "non_minimal_varint"
	DecodeVarintOverflow       DecodeErrorKind = "varint_overflow"
	DecodeLengthOverflow       DecodeErrorKind = "length_overflow"
	DecodeInvalidUTF8          DecodeErrorKind = "invalid_utf8"
	DecodeInvalidName          DecodeErrorKind = "invalid_name"
	DecodeInvalidBool          DecodeErrorKind = "invalid_bool"
	DecodeUnknownTag           DecodeErrorKind = "unknown_tag"
	DecodeUnknownAxiomCategory DecodeErrorKind = "unknown_axiom_category"
	DecodeUnknownReducibility  DecodeErrorKind = "unknown_reducibility"
	DecodeInvalidReference     DecodeErrorKind = "invalid_reference"
	DecodeFutureReference      DecodeErrorKind = "future_reference"
)

type DecodeError struct {
	Kind   DecodeErrorKind
	Offset int
	Detail string
}

func (e *DecodeError) Error() string {
	if e.Detail == "" {
		return string(e.Kind) + " at offset " + formatUint64(uint64(e.Offset))
	}
	return string(e.Kind) + " at offset " + formatUint64(uint64(e.Offset)) + ": " + e.Detail
}

func DecodeCertificate(data []byte) (*Certificate, error) {
	d := decoder{data: data}
	if err := d.readMagic(); err != nil {
		return nil, err
	}

	format, err := d.readString()
	if err != nil {
		return nil, err
	}
	if format != certFormat {
		return nil, d.errAt(0, DecodeInvalidFormat, format)
	}

	spec, err := d.readString()
	if err != nil {
		return nil, err
	}
	if spec != coreSpec {
		return nil, d.errAt(0, DecodeInvalidCoreSpec, spec)
	}

	module, err := d.readString()
	if err != nil {
		return nil, err
	}
	if err := validateName(module, "module"); err != nil {
		return nil, err
	}

	imports, err := readVec(&d, (*decoder).readImport)
	if err != nil {
		return nil, err
	}
	nameTable, err := readVec(&d, (*decoder).readName)
	if err != nil {
		return nil, err
	}
	levelTable, err := readVec(&d, (*decoder).readLevelNode)
	if err != nil {
		return nil, err
	}
	termTable, err := readVec(&d, (*decoder).readTermNode)
	if err != nil {
		return nil, err
	}
	proofNodeTable, err := readVec(&d, (*decoder).readProofNode)
	if err != nil {
		return nil, err
	}
	declarations, err := readVec(&d, (*decoder).readDeclaration)
	if err != nil {
		return nil, err
	}
	theoryCertificates, err := readVec(&d, (*decoder).readTheoryCertificate)
	if err != nil {
		return nil, err
	}
	exportBlock, err := readVec(&d, (*decoder).readExportEntry)
	if err != nil {
		return nil, err
	}
	axiomReport, err := d.readAxiomReport()
	if err != nil {
		return nil, err
	}
	sourceManifest, err := d.readSourceManifest()
	if err != nil {
		return nil, err
	}
	hashes, err := d.readHashes()
	if err != nil {
		return nil, err
	}

	if !d.finished() {
		return nil, d.err(DecodeTrailingBytes, "")
	}

	cert := &Certificate{
		Module:             module,
		Imports:            imports,
		NameTable:          nameTable,
		LevelTable:         levelTable,
		TermTable:          termTable,
		ProofNodeTable:     proofNodeTable,
		Declarations:       declarations,
		TheoryCertificates: theoryCertificates,
		ExportBlock:        exportBlock,
		AxiomReport:        axiomReport,
		SourceManifest:     sourceManifest,
		Hashes:             hashes,
	}
	if err := validateCertificateShape(cert); err != nil {
		return nil, err
	}
	return cert, nil
}

type decoder struct {
	data   []byte
	offset int
}

func (d *decoder) finished() bool {
	return d.offset == len(d.data)
}

func (d *decoder) err(kind DecodeErrorKind, detail string) *DecodeError {
	return d.errAt(d.offset, kind, detail)
}

func (d *decoder) errAt(offset int, kind DecodeErrorKind, detail string) *DecodeError {
	return &DecodeError{Kind: kind, Offset: offset, Detail: detail}
}

func (d *decoder) readMagic() error {
	start := d.offset
	got, err := d.readExact(len(certMagic))
	if err != nil {
		return err
	}
	if !equalBytes(got, []byte(certMagic)) {
		return d.errAt(start, DecodeInvalidMagic, "")
	}
	return nil
}

func (d *decoder) readExact(n int) ([]byte, error) {
	start := d.offset
	if n < 0 || start > len(d.data)-n {
		return nil, d.errAt(start, DecodeUnexpectedEOF, "")
	}
	d.offset += n
	return d.data[start:d.offset], nil
}

func (d *decoder) readU8() (uint8, error) {
	b, err := d.readExact(1)
	if err != nil {
		return 0, err
	}
	return b[0], nil
}

func (d *decoder) readBool() (bool, error) {
	start := d.offset
	value, err := d.readU8()
	if err != nil {
		return false, err
	}
	switch value {
	case 0:
		return false, nil
	case 1:
		return true, nil
	default:
		return false, d.errAt(start, DecodeInvalidBool, "")
	}
}

func (d *decoder) readU32() (uint32, error) {
	start := d.offset
	value, err := d.readU64()
	if err != nil {
		return 0, err
	}
	if value > uint64(maxUint32()) {
		return 0, d.errAt(start, DecodeLengthOverflow, "")
	}
	return uint32(value), nil
}

func (d *decoder) readU64() (uint64, error) {
	start := d.offset
	var result uint64
	for byteIndex := 0; byteIndex < 10; byteIndex++ {
		b, err := d.readU8()
		if err != nil {
			return 0, err
		}
		low := uint64(b & 0x7f)
		if byteIndex == 9 && low > 1 {
			return 0, d.errAt(start, DecodeVarintOverflow, "")
		}

		result |= low << (byteIndex * 7)
		if b&0x80 == 0 {
			used := byteIndex + 1
			if minimalVarintLen(result) != used {
				return 0, d.errAt(start, DecodeNonMinimalVarint, "")
			}
			return result, nil
		}
	}
	return 0, d.errAt(start, DecodeVarintOverflow, "")
}

func (d *decoder) readLen() (int, error) {
	start := d.offset
	value, err := d.readU64()
	if err != nil {
		return 0, err
	}
	if value > uint64(maxInt()) {
		return 0, d.errAt(start, DecodeLengthOverflow, "")
	}
	return int(value), nil
}

func (d *decoder) readBytesWithLen() ([]byte, error) {
	n, err := d.readLen()
	if err != nil {
		return nil, err
	}
	bytes, err := d.readExact(n)
	if err != nil {
		return nil, err
	}
	return append([]byte(nil), bytes...), nil
}

func (d *decoder) readString() (string, error) {
	start := d.offset
	bytes, err := d.readBytesWithLen()
	if err != nil {
		return "", err
	}
	if !validUTF8(bytes) {
		return "", d.errAt(start, DecodeInvalidUTF8, "")
	}
	return string(bytes), nil
}

func (d *decoder) readName() (string, error) {
	name, err := d.readString()
	if err != nil {
		return "", err
	}
	if err := validateName(name, "name_table"); err != nil {
		return "", err
	}
	return name, nil
}

func (d *decoder) readHash() (HashBytes, error) {
	var hash HashBytes
	bytes, err := d.readExact(hashLen)
	if err != nil {
		return hash, err
	}
	copy(hash[:], bytes)
	return hash, nil
}

func (d *decoder) readOptionalHash() (*HashBytes, error) {
	present, err := d.readBool()
	if err != nil {
		return nil, err
	}
	if !present {
		return nil, nil
	}
	hash, err := d.readHash()
	if err != nil {
		return nil, err
	}
	return &hash, nil
}

func (d *decoder) readOptionalString() (*string, error) {
	present, err := d.readBool()
	if err != nil {
		return nil, err
	}
	if !present {
		return nil, nil
	}
	value, err := d.readString()
	if err != nil {
		return nil, err
	}
	return &value, nil
}

func (d *decoder) readOptionalU32() (*uint32, error) {
	present, err := d.readBool()
	if err != nil {
		return nil, err
	}
	if !present {
		return nil, nil
	}
	value, err := d.readU32()
	if err != nil {
		return nil, err
	}
	return &value, nil
}

func readVec[T any](d *decoder, readItem func(*decoder) (T, error)) ([]T, error) {
	n, err := d.readLen()
	if err != nil {
		return nil, err
	}
	var values []T
	for i := 0; i < n; i++ {
		value, err := readItem(d)
		if err != nil {
			return nil, err
		}
		values = append(values, value)
	}
	return values, nil
}

func (d *decoder) readU32Vec() ([]uint32, error) {
	return readVec(d, (*decoder).readU32)
}

func (d *decoder) readImport() (Import, error) {
	moduleName, err := d.readString()
	if err != nil {
		return Import{}, err
	}
	if err := validateName(moduleName, "import.module_name"); err != nil {
		return Import{}, err
	}
	exportHash, err := d.readHash()
	if err != nil {
		return Import{}, err
	}
	certificateHash, err := d.readOptionalHash()
	if err != nil {
		return Import{}, err
	}
	return Import{
		ModuleName:      moduleName,
		ExportHash:      exportHash,
		CertificateHash: certificateHash,
	}, nil
}

func (d *decoder) readLevelNode() (LevelNode, error) {
	start := d.offset
	tag, err := d.readU8()
	if err != nil {
		return LevelNode{}, err
	}
	switch LevelTag(tag) {
	case LevelZero:
		return LevelNode{Tag: LevelZero}, nil
	case LevelSucc:
		inner, err := d.readU32()
		return LevelNode{Tag: LevelSucc, A: inner}, err
	case LevelMax:
		lhs, err := d.readU32()
		if err != nil {
			return LevelNode{}, err
		}
		rhs, err := d.readU32()
		return LevelNode{Tag: LevelMax, A: lhs, B: rhs}, err
	case LevelParam:
		name, err := d.readU32()
		return LevelNode{Tag: LevelParam, A: name}, err
	default:
		return LevelNode{}, d.errAt(start, DecodeUnknownTag, "")
	}
}

func (d *decoder) readTermNode() (TermNode, error) {
	start := d.offset
	tag, err := d.readU8()
	if err != nil {
		return TermNode{}, err
	}
	switch TermTag(tag) {
	case TermSort:
		level, err := d.readU32()
		return TermNode{Tag: TermSort, A: level}, err
	case TermVar:
		index, err := d.readU32()
		return TermNode{Tag: TermVar, A: index}, err
	case TermConst:
		global, err := d.readU32()
		if err != nil {
			return TermNode{}, err
		}
		levels, err := d.readU32Vec()
		return TermNode{Tag: TermConst, A: global, Arguments: levels}, err
	case TermApp:
		function, err := d.readU32()
		if err != nil {
			return TermNode{}, err
		}
		arguments, err := d.readU32Vec()
		return TermNode{Tag: TermApp, A: function, Arguments: arguments}, err
	case TermLam:
		return d.readBinaryTerm(TermLam)
	case TermPi:
		return d.readBinaryTerm(TermPi)
	case TermLet:
		ty, err := d.readU32()
		if err != nil {
			return TermNode{}, err
		}
		value, err := d.readU32()
		if err != nil {
			return TermNode{}, err
		}
		body, err := d.readU32()
		return TermNode{Tag: TermLet, A: ty, B: value, C: body}, err
	default:
		return TermNode{}, d.errAt(start, DecodeUnknownTag, "")
	}
}

func (d *decoder) readBinaryTerm(tag TermTag) (TermNode, error) {
	ty, err := d.readU32()
	if err != nil {
		return TermNode{}, err
	}
	body, err := d.readU32()
	return TermNode{Tag: tag, A: ty, B: body}, err
}

func (d *decoder) readProofNode() (ProofNode, error) {
	start := d.offset
	tag, err := d.readU8()
	if err != nil {
		return ProofNode{}, err
	}
	switch ProofNodeTag(tag) {
	case ProofExact:
		term, expectedType, err := d.readProofTermAndExpected()
		return ProofNode{Tag: ProofExact, Term: term, ExpectedType: expectedType}, err
	case ProofApply:
		functionProof, err := d.readU32()
		if err != nil {
			return ProofNode{}, err
		}
		argumentProofs, err := d.readU32Vec()
		if err != nil {
			return ProofNode{}, err
		}
		expectedType, err := d.readU32()
		return ProofNode{
			Tag:            ProofApply,
			FunctionProof:  functionProof,
			ArgumentProofs: argumentProofs,
			ExpectedType:   expectedType,
		}, err
	case ProofIntro:
		domainType, err := d.readU32()
		if err != nil {
			return ProofNode{}, err
		}
		bodyProof, err := d.readU32()
		if err != nil {
			return ProofNode{}, err
		}
		expectedType, err := d.readU32()
		return ProofNode{Tag: ProofIntro, DomainType: domainType, BodyProof: bodyProof, ExpectedType: expectedType}, err
	case ProofLetProof:
		value, err := d.readU32()
		if err != nil {
			return ProofNode{}, err
		}
		bodyProof, err := d.readU32()
		if err != nil {
			return ProofNode{}, err
		}
		expectedType, err := d.readU32()
		return ProofNode{Tag: ProofLetProof, Value: value, BodyProof: bodyProof, ExpectedType: expectedType}, err
	case ProofRefl:
		term, expectedType, err := d.readProofTermAndExpected()
		return ProofNode{Tag: ProofRefl, Term: term, ExpectedType: expectedType}, err
	case ProofRewrite:
		eqProof, err := d.readU32()
		if err != nil {
			return ProofNode{}, err
		}
		targetProof, err := d.readU32()
		if err != nil {
			return ProofNode{}, err
		}
		expectedType, err := d.readU32()
		return ProofNode{Tag: ProofRewrite, EqProof: eqProof, TargetProof: targetProof, ExpectedType: expectedType}, err
	case ProofEqRec:
		motive, err := d.readU32()
		if err != nil {
			return ProofNode{}, err
		}
		eqProof, err := d.readU32()
		if err != nil {
			return ProofNode{}, err
		}
		baseProof, err := d.readU32()
		if err != nil {
			return ProofNode{}, err
		}
		expectedType, err := d.readU32()
		return ProofNode{Tag: ProofEqRec, Motive: motive, EqProof: eqProof, BaseProof: baseProof, ExpectedType: expectedType}, err
	case ProofConstructor:
		constructor, err := d.readU32()
		if err != nil {
			return ProofNode{}, err
		}
		argumentProofs, err := d.readU32Vec()
		if err != nil {
			return ProofNode{}, err
		}
		expectedType, err := d.readU32()
		return ProofNode{Tag: ProofConstructor, Constructor: constructor, ArgumentProofs: argumentProofs, ExpectedType: expectedType}, err
	case ProofRecursor:
		recursor, err := d.readU32()
		if err != nil {
			return ProofNode{}, err
		}
		motive, err := d.readU32()
		if err != nil {
			return ProofNode{}, err
		}
		minorProofs, err := d.readU32Vec()
		if err != nil {
			return ProofNode{}, err
		}
		majorProof, err := d.readU32()
		if err != nil {
			return ProofNode{}, err
		}
		expectedType, err := d.readU32()
		return ProofNode{
			Tag:          ProofRecursor,
			Recursor:     recursor,
			Motive:       motive,
			MinorProofs:  minorProofs,
			MajorProof:   majorProof,
			ExpectedType: expectedType,
		}, err
	case ProofConv:
		proof, err := d.readU32()
		if err != nil {
			return ProofNode{}, err
		}
		expectedType, err := d.readU32()
		if err != nil {
			return ProofNode{}, err
		}
		defeqWitness, err := d.readOptionalU32()
		return ProofNode{Tag: ProofConv, Proof: proof, ExpectedType: expectedType, DefeqWitness: defeqWitness}, err
	case ProofTheory:
		theoryCertificate, err := d.readU32()
		if err != nil {
			return ProofNode{}, err
		}
		expectedType, err := d.readU32()
		return ProofNode{Tag: ProofTheory, TheoryCertificate: theoryCertificate, ExpectedType: expectedType}, err
	default:
		return ProofNode{}, d.errAt(start, DecodeUnknownTag, "")
	}
}

func (d *decoder) readProofTermAndExpected() (uint32, uint32, error) {
	term, err := d.readU32()
	if err != nil {
		return 0, 0, err
	}
	expectedType, err := d.readU32()
	return term, expectedType, err
}

func (d *decoder) readDeclaration() (Declaration, error) {
	name, err := d.readU32()
	if err != nil {
		return Declaration{}, err
	}
	start := d.offset
	tag, err := d.readU8()
	if err != nil {
		return Declaration{}, err
	}
	switch DeclarationTag(tag) {
	case DeclAxiom:
		ty, err := d.readU32()
		return Declaration{Name: name, Tag: DeclAxiom, Type: ty}, err
	case DeclDef:
		ty, err := d.readU32()
		if err != nil {
			return Declaration{}, err
		}
		value, err := d.readU32()
		if err != nil {
			return Declaration{}, err
		}
		reducibility, err := d.readReducibility()
		return Declaration{Name: name, Tag: DeclDef, Type: ty, Value: value, Reducibility: reducibility}, err
	case DeclTheorem:
		ty, err := d.readU32()
		if err != nil {
			return Declaration{}, err
		}
		proof, err := d.readU32()
		return Declaration{Name: name, Tag: DeclTheorem, Type: ty, Proof: proof}, err
	case DeclInductive:
		ty, err := d.readU32()
		return Declaration{Name: name, Tag: DeclInductive, Type: ty}, err
	case DeclConstructor:
		return d.readGeneratedDeclaration(name, DeclConstructor)
	case DeclRecursor:
		return d.readGeneratedDeclaration(name, DeclRecursor)
	case DeclTheoryPrimitive:
		ty, err := d.readU32()
		return Declaration{Name: name, Tag: DeclTheoryPrimitive, Type: ty}, err
	default:
		return Declaration{}, d.errAt(start, DecodeUnknownTag, "")
	}
}

func (d *decoder) readGeneratedDeclaration(name uint32, tag DeclarationTag) (Declaration, error) {
	ty, err := d.readU32()
	if err != nil {
		return Declaration{}, err
	}
	inductive, err := d.readU32()
	if err != nil {
		return Declaration{}, err
	}
	generated, err := d.readBool()
	if err != nil {
		return Declaration{}, err
	}
	return Declaration{Name: name, Tag: tag, Type: ty, Inductive: inductive, Generated: generated}, nil
}

func (d *decoder) readReducibility() (Reducibility, error) {
	start := d.offset
	tag, err := d.readU8()
	if err != nil {
		return 0, err
	}
	switch tag {
	case 0:
		return Reducible, nil
	case 1:
		return Opaque, nil
	default:
		return 0, d.errAt(start, DecodeUnknownReducibility, "")
	}
}

func (d *decoder) readTheoryCertificate() (TheoryCertificate, error) {
	format, err := d.readString()
	if err != nil {
		return TheoryCertificate{}, err
	}
	payload, err := d.readBytesWithLen()
	if err != nil {
		return TheoryCertificate{}, err
	}
	return TheoryCertificate{Format: format, Payload: payload}, nil
}

func (d *decoder) readExportEntry() (ExportEntry, error) {
	name, err := d.readU32()
	if err != nil {
		return ExportEntry{}, err
	}
	declaration, err := d.readU32()
	if err != nil {
		return ExportEntry{}, err
	}
	declarationHash, err := d.readHash()
	return ExportEntry{Name: name, Declaration: declaration, DeclarationHash: declarationHash}, err
}

func (d *decoder) readAxiomReport() (AxiomReport, error) {
	entries, err := readVec(d, (*decoder).readAxiomReportEntry)
	if err != nil {
		return AxiomReport{}, err
	}
	dependencies, err := readVec(d, (*decoder).readDeclarationAxiomDependencies)
	if err != nil {
		return AxiomReport{}, err
	}
	summary, err := d.readAxiomReportSummary()
	if err != nil {
		return AxiomReport{}, err
	}
	return AxiomReport{Entries: entries, DeclarationDependencies: dependencies, Summary: summary}, nil
}

func (d *decoder) readAxiomReportEntry() (AxiomReportEntry, error) {
	category, err := d.readAxiomCategory()
	if err != nil {
		return AxiomReportEntry{}, err
	}
	name, err := d.readString()
	if err != nil {
		return AxiomReportEntry{}, err
	}
	if err := validateName(name, "axiom_report.name"); err != nil {
		return AxiomReportEntry{}, err
	}
	originModule, err := d.readString()
	if err != nil {
		return AxiomReportEntry{}, err
	}
	if err := validateName(originModule, "axiom_report.origin_module"); err != nil {
		return AxiomReportEntry{}, err
	}
	typeHash, err := d.readHash()
	if err != nil {
		return AxiomReportEntry{}, err
	}
	declarationHash, err := d.readHash()
	if err != nil {
		return AxiomReportEntry{}, err
	}
	sourceCertificateHash, err := d.readOptionalHash()
	if err != nil {
		return AxiomReportEntry{}, err
	}
	direct, err := d.readU32Vec()
	if err != nil {
		return AxiomReportEntry{}, err
	}
	transitive, err := d.readU32Vec()
	if err != nil {
		return AxiomReportEntry{}, err
	}
	approvalProfile, err := d.readOptionalString()
	if err != nil {
		return AxiomReportEntry{}, err
	}
	reviewerNote, err := d.readOptionalString()
	if err != nil {
		return AxiomReportEntry{}, err
	}
	return AxiomReportEntry{
		Category:                        category,
		Name:                            name,
		OriginModule:                    originModule,
		TypeHash:                        typeHash,
		DeclarationHash:                 declarationHash,
		SourceCertificateHash:           sourceCertificateHash,
		DirectDependentDeclarations:     direct,
		TransitiveDependentDeclarations: transitive,
		ApprovalProfile:                 approvalProfile,
		ReviewerNote:                    reviewerNote,
	}, nil
}

func (d *decoder) readAxiomCategory() (AxiomCategory, error) {
	start := d.offset
	value, err := d.readString()
	if err != nil {
		return "", err
	}
	category := AxiomCategory(value)
	switch category {
	case AxiomCategoryCore, AxiomCategoryBuiltinTheory, AxiomCategoryGoSemantics, AxiomCategoryExternal:
		return category, nil
	default:
		return "", d.errAt(start, DecodeUnknownAxiomCategory, "")
	}
}

func (d *decoder) readDeclarationAxiomDependencies() (DeclarationAxiomDependencies, error) {
	declarationName, err := d.readString()
	if err != nil {
		return DeclarationAxiomDependencies{}, err
	}
	if err := validateName(declarationName, "axiom_report.declaration_name"); err != nil {
		return DeclarationAxiomDependencies{}, err
	}
	declarationHash, err := d.readHash()
	if err != nil {
		return DeclarationAxiomDependencies{}, err
	}
	direct, err := d.readU32Vec()
	if err != nil {
		return DeclarationAxiomDependencies{}, err
	}
	transitive, err := d.readU32Vec()
	if err != nil {
		return DeclarationAxiomDependencies{}, err
	}
	return DeclarationAxiomDependencies{
		DeclarationName:             declarationName,
		DeclarationHash:             declarationHash,
		DirectAxiomDependencies:     direct,
		TransitiveAxiomDependencies: transitive,
	}, nil
}

func (d *decoder) readAxiomReportSummary() (AxiomReportSummary, error) {
	core, err := d.readU64()
	if err != nil {
		return AxiomReportSummary{}, err
	}
	builtinTheory, err := d.readU64()
	if err != nil {
		return AxiomReportSummary{}, err
	}
	goSemantics, err := d.readU64()
	if err != nil {
		return AxiomReportSummary{}, err
	}
	external, err := d.readU64()
	if err != nil {
		return AxiomReportSummary{}, err
	}
	total, err := d.readU64()
	if err != nil {
		return AxiomReportSummary{}, err
	}
	return AxiomReportSummary{
		CoreAxiomCount:          core,
		BuiltinTheoryAxiomCount: builtinTheory,
		GoSemanticsAxiomCount:   goSemantics,
		ExternalAxiomCount:      external,
		TotalAxiomCount:         total,
	}, nil
}

func (d *decoder) readSourceManifest() (*SourceManifest, error) {
	present, err := d.readBool()
	if err != nil {
		return nil, err
	}
	if !present {
		return nil, nil
	}
	payload, err := d.readBytesWithLen()
	if err != nil {
		return nil, err
	}
	return &SourceManifest{Payload: payload}, nil
}

func (d *decoder) readHashes() (CertificateHashes, error) {
	exportHash, err := d.readHash()
	if err != nil {
		return CertificateHashes{}, err
	}
	axiomReportHash, err := d.readHash()
	if err != nil {
		return CertificateHashes{}, err
	}
	certificateHash, err := d.readHash()
	if err != nil {
		return CertificateHashes{}, err
	}
	return CertificateHashes{
		ExportHash:      exportHash,
		AxiomReportHash: axiomReportHash,
		CertificateHash: certificateHash,
	}, nil
}

func validateCertificateShape(c *Certificate) error {
	for i, level := range c.LevelTable {
		switch level.Tag {
		case LevelZero:
		case LevelSucc:
			if err := checkFutureIndex(level.A, i, "level.succ"); err != nil {
				return err
			}
		case LevelMax:
			if err := checkFutureIndex(level.A, i, "level.max.lhs"); err != nil {
				return err
			}
			if err := checkFutureIndex(level.B, i, "level.max.rhs"); err != nil {
				return err
			}
		case LevelParam:
			if err := checkIndex(level.A, len(c.NameTable), "level.param"); err != nil {
				return err
			}
		}
	}

	for i, term := range c.TermTable {
		switch term.Tag {
		case TermSort:
			if err := checkIndex(term.A, len(c.LevelTable), "term.sort"); err != nil {
				return err
			}
		case TermVar:
		case TermConst:
			if err := checkIndices(term.Arguments, len(c.LevelTable), "term.const.level"); err != nil {
				return err
			}
		case TermApp:
			if err := checkFutureIndex(term.A, i, "term.app.function"); err != nil {
				return err
			}
			if err := checkFutureIndices(term.Arguments, i, "term.app.argument"); err != nil {
				return err
			}
		case TermLam, TermPi:
			if err := checkFutureIndex(term.A, i, "term.binder.ty"); err != nil {
				return err
			}
			if err := checkFutureIndex(term.B, i, "term.binder.body"); err != nil {
				return err
			}
		case TermLet:
			if err := checkFutureIndex(term.A, i, "term.let.ty"); err != nil {
				return err
			}
			if err := checkFutureIndex(term.B, i, "term.let.value"); err != nil {
				return err
			}
			if err := checkFutureIndex(term.C, i, "term.let.body"); err != nil {
				return err
			}
		}
	}

	for i, proof := range c.ProofNodeTable {
		if err := validateProofNodeShape(c, proof, i); err != nil {
			return err
		}
	}

	for i, declaration := range c.Declarations {
		if err := checkIndex(declaration.Name, len(c.NameTable), "decl.name"); err != nil {
			return err
		}
		switch declaration.Tag {
		case DeclAxiom, DeclInductive, DeclTheoryPrimitive:
			if err := checkIndex(declaration.Type, len(c.TermTable), "decl.ty"); err != nil {
				return err
			}
		case DeclDef:
			if err := checkIndex(declaration.Type, len(c.TermTable), "decl.def.ty"); err != nil {
				return err
			}
			if err := checkIndex(declaration.Value, len(c.TermTable), "decl.def.value"); err != nil {
				return err
			}
		case DeclTheorem:
			if err := checkIndex(declaration.Type, len(c.TermTable), "decl.theorem.ty"); err != nil {
				return err
			}
			if err := checkIndex(declaration.Proof, len(c.TermTable), "decl.theorem.proof"); err != nil {
				return err
			}
		case DeclConstructor, DeclRecursor:
			if err := checkIndex(declaration.Type, len(c.TermTable), "decl.generated.ty"); err != nil {
				return err
			}
			if err := checkFutureIndex(declaration.Inductive, i, "decl.generated.inductive"); err != nil {
				return err
			}
		}
	}

	for _, export := range c.ExportBlock {
		if err := checkIndex(export.Name, len(c.NameTable), "export.name"); err != nil {
			return err
		}
		if err := checkIndex(export.Declaration, len(c.Declarations), "export.declaration"); err != nil {
			return err
		}
	}

	return validateAxiomReportShape(c)
}

func validateProofNodeShape(c *Certificate, proof ProofNode, index int) error {
	switch proof.Tag {
	case ProofExact, ProofRefl:
		if err := checkIndex(proof.Term, len(c.TermTable), "proof.term"); err != nil {
			return err
		}
		return checkIndex(proof.ExpectedType, len(c.TermTable), "proof.expected_type")
	case ProofApply:
		if err := checkFutureIndex(proof.FunctionProof, index, "proof.apply.function"); err != nil {
			return err
		}
		if err := checkFutureIndices(proof.ArgumentProofs, index, "proof.apply.argument"); err != nil {
			return err
		}
		return checkIndex(proof.ExpectedType, len(c.TermTable), "proof.expected_type")
	case ProofIntro:
		if err := checkIndex(proof.DomainType, len(c.TermTable), "proof.intro.domain"); err != nil {
			return err
		}
		if err := checkFutureIndex(proof.BodyProof, index, "proof.intro.body"); err != nil {
			return err
		}
		return checkIndex(proof.ExpectedType, len(c.TermTable), "proof.expected_type")
	case ProofLetProof:
		if err := checkIndex(proof.Value, len(c.TermTable), "proof.let.value"); err != nil {
			return err
		}
		if err := checkFutureIndex(proof.BodyProof, index, "proof.let.body"); err != nil {
			return err
		}
		return checkIndex(proof.ExpectedType, len(c.TermTable), "proof.expected_type")
	case ProofRewrite:
		if err := checkFutureIndex(proof.EqProof, index, "proof.rewrite.eq"); err != nil {
			return err
		}
		if err := checkFutureIndex(proof.TargetProof, index, "proof.rewrite.target"); err != nil {
			return err
		}
		return checkIndex(proof.ExpectedType, len(c.TermTable), "proof.expected_type")
	case ProofEqRec:
		if err := checkIndex(proof.Motive, len(c.TermTable), "proof.eq_rec.motive"); err != nil {
			return err
		}
		if err := checkFutureIndex(proof.EqProof, index, "proof.eq_rec.eq"); err != nil {
			return err
		}
		if err := checkFutureIndex(proof.BaseProof, index, "proof.eq_rec.base"); err != nil {
			return err
		}
		return checkIndex(proof.ExpectedType, len(c.TermTable), "proof.expected_type")
	case ProofConstructor:
		if err := checkFutureIndices(proof.ArgumentProofs, index, "proof.constructor.argument"); err != nil {
			return err
		}
		return checkIndex(proof.ExpectedType, len(c.TermTable), "proof.expected_type")
	case ProofRecursor:
		if err := checkIndex(proof.Motive, len(c.TermTable), "proof.recursor.motive"); err != nil {
			return err
		}
		if err := checkFutureIndices(proof.MinorProofs, index, "proof.recursor.minor"); err != nil {
			return err
		}
		if err := checkFutureIndex(proof.MajorProof, index, "proof.recursor.major"); err != nil {
			return err
		}
		return checkIndex(proof.ExpectedType, len(c.TermTable), "proof.expected_type")
	case ProofConv:
		if err := checkFutureIndex(proof.Proof, index, "proof.conv.proof"); err != nil {
			return err
		}
		if err := checkIndex(proof.ExpectedType, len(c.TermTable), "proof.expected_type"); err != nil {
			return err
		}
		if proof.DefeqWitness != nil {
			return checkIndex(*proof.DefeqWitness, len(c.TermTable), "proof.conv.defeq_witness")
		}
		return nil
	case ProofTheory:
		if err := checkIndex(proof.TheoryCertificate, len(c.TheoryCertificates), "proof.theory_certificate"); err != nil {
			return err
		}
		return checkIndex(proof.ExpectedType, len(c.TermTable), "proof.expected_type")
	default:
		return newDecodeError(DecodeUnknownTag, 0, "")
	}
}

func validateAxiomReportShape(c *Certificate) error {
	for _, entry := range c.AxiomReport.Entries {
		if err := checkIndices(entry.DirectDependentDeclarations, len(c.Declarations), "axiom_report.direct_dependent_declaration"); err != nil {
			return err
		}
		if err := checkIndices(entry.TransitiveDependentDeclarations, len(c.Declarations), "axiom_report.transitive_dependent_declaration"); err != nil {
			return err
		}
	}
	for _, dependencies := range c.AxiomReport.DeclarationDependencies {
		if err := checkIndices(dependencies.DirectAxiomDependencies, len(c.AxiomReport.Entries), "axiom_report.direct_axiom_dependency"); err != nil {
			return err
		}
		if err := checkIndices(dependencies.TransitiveAxiomDependencies, len(c.AxiomReport.Entries), "axiom_report.transitive_axiom_dependency"); err != nil {
			return err
		}
	}
	return nil
}

func checkIndex(id uint32, length int, field string) error {
	if uint64(id) < uint64(length) {
		return nil
	}
	return newDecodeError(
		DecodeInvalidReference,
		0,
		field+"="+formatUint64(uint64(id))+" len="+formatUint64(uint64(length)),
	)
}

func checkIndices(ids []uint32, length int, field string) error {
	for _, id := range ids {
		if err := checkIndex(id, length, field); err != nil {
			return err
		}
	}
	return nil
}

func checkFutureIndex(id uint32, current int, field string) error {
	if uint64(id) < uint64(current) {
		return nil
	}
	return newDecodeError(
		DecodeFutureReference,
		0,
		field+"="+formatUint64(uint64(id))+" current="+formatUint64(uint64(current)),
	)
}

func checkFutureIndices(ids []uint32, current int, field string) error {
	for _, id := range ids {
		if err := checkFutureIndex(id, current, field); err != nil {
			return err
		}
	}
	return nil
}

func validateName(name string, field string) error {
	if name == "" {
		return newDecodeError(DecodeInvalidName, 0, field+":EMPTY_NAME")
	}
	for i := 0; i < len(name); i++ {
		if name[i] > 0x7f {
			return newDecodeError(DecodeInvalidName, 0, field+":NON_ASCII")
		}
	}

	componentStart := 0
	componentIndex := 0
	for i := 0; i < len(name); i++ {
		if name[i] != '.' {
			continue
		}
		if err := validateNameComponent(name, field, componentStart, i, componentIndex); err != nil {
			return err
		}
		componentStart = i + 1
		componentIndex++
	}
	return validateNameComponent(name, field, componentStart, len(name), componentIndex)
}

func validateNameComponent(name string, field string, start int, end int, componentIndex int) error {
	if start == end {
		return newDecodeError(
			DecodeInvalidName,
			0,
			field+":EMPTY_COMPONENT:"+formatUint64(uint64(componentIndex)),
		)
	}
	if !isComponentStart(name[start]) {
		return newDecodeError(
			DecodeInvalidName,
			0,
			field+":INVALID_COMPONENT_START:"+formatUint64(uint64(componentIndex)),
		)
	}
	for i := start + 1; i < end; i++ {
		if !isComponentContinue(name[i]) {
			return newDecodeError(
				DecodeInvalidName,
				0,
				field+":INVALID_COMPONENT_CHAR:"+formatUint64(uint64(componentIndex)),
			)
		}
	}
	return nil
}

func isComponentStart(b byte) bool {
	return (b >= 'A' && b <= 'Z') || (b >= 'a' && b <= 'z') || b == '_'
}

func isComponentContinue(b byte) bool {
	return isComponentStart(b) || (b >= '0' && b <= '9') || b == '\''
}

func minimalVarintLen(value uint64) int {
	length := 1
	for value >= 0x80 {
		value >>= 7
		length++
	}
	return length
}

func maxInt() int {
	return int(^uint(0) >> 1)
}

func maxUint32() uint32 {
	return ^uint32(0)
}

func newDecodeError(kind DecodeErrorKind, offset int, detail string) *DecodeError {
	return &DecodeError{Kind: kind, Offset: offset, Detail: detail}
}

func equalBytes(lhs []byte, rhs []byte) bool {
	if len(lhs) != len(rhs) {
		return false
	}
	for i := range lhs {
		if lhs[i] != rhs[i] {
			return false
		}
	}
	return true
}

func validUTF8(bytes []byte) bool {
	for i := 0; i < len(bytes); {
		first := bytes[i]
		if first < 0x80 {
			i++
			continue
		}

		switch {
		case first >= 0xc2 && first <= 0xdf:
			if i+1 >= len(bytes) || !isUTF8Continuation(bytes[i+1]) {
				return false
			}
			i += 2
		case first == 0xe0:
			if i+2 >= len(bytes) ||
				bytes[i+1] < 0xa0 || bytes[i+1] > 0xbf ||
				!isUTF8Continuation(bytes[i+2]) {
				return false
			}
			i += 3
		case first >= 0xe1 && first <= 0xec:
			if i+2 >= len(bytes) ||
				!isUTF8Continuation(bytes[i+1]) ||
				!isUTF8Continuation(bytes[i+2]) {
				return false
			}
			i += 3
		case first == 0xed:
			if i+2 >= len(bytes) ||
				bytes[i+1] < 0x80 || bytes[i+1] > 0x9f ||
				!isUTF8Continuation(bytes[i+2]) {
				return false
			}
			i += 3
		case first >= 0xee && first <= 0xef:
			if i+2 >= len(bytes) ||
				!isUTF8Continuation(bytes[i+1]) ||
				!isUTF8Continuation(bytes[i+2]) {
				return false
			}
			i += 3
		case first == 0xf0:
			if i+3 >= len(bytes) ||
				bytes[i+1] < 0x90 || bytes[i+1] > 0xbf ||
				!isUTF8Continuation(bytes[i+2]) ||
				!isUTF8Continuation(bytes[i+3]) {
				return false
			}
			i += 4
		case first >= 0xf1 && first <= 0xf3:
			if i+3 >= len(bytes) ||
				!isUTF8Continuation(bytes[i+1]) ||
				!isUTF8Continuation(bytes[i+2]) ||
				!isUTF8Continuation(bytes[i+3]) {
				return false
			}
			i += 4
		case first == 0xf4:
			if i+3 >= len(bytes) ||
				bytes[i+1] < 0x80 || bytes[i+1] > 0x8f ||
				!isUTF8Continuation(bytes[i+2]) ||
				!isUTF8Continuation(bytes[i+3]) {
				return false
			}
			i += 4
		default:
			return false
		}
	}
	return true
}

func isUTF8Continuation(b byte) bool {
	return b&0xc0 == 0x80
}

func formatUint64(value uint64) string {
	if value == 0 {
		return "0"
	}
	var digits [20]byte
	index := len(digits)
	for value > 0 {
		index--
		digits[index] = byte('0' + value%10)
		value /= 10
	}
	return string(digits[index:])
}
