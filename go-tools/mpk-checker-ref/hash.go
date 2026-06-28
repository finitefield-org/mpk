package mpkcheckerref

type HashDomain string

const (
	HashDomainModuleExport      HashDomain = "MPK-MODULE-EXPORT-0.1"
	HashDomainModuleCertificate HashDomain = "MPK-MODULE-CERT-0.1"
	HashDomainAxiomReport       HashDomain = "MPK-AXIOM-REPORT-0.1"
	HashDomainLevel             HashDomain = "MPK-LEVEL-0.1"
	HashDomainTerm              HashDomain = "MPK-TERM-0.1"
	HashDomainProofNode         HashDomain = "MPK-PROOF-NODE-0.1"
	HashDomainDeclaration       HashDomain = "MPK-DECL-0.1"
	HashDomainTheoryCertificate HashDomain = "MPK-THEORY-CERT-0.1"
	HashDomainSourceManifest    HashDomain = "MPK-SOURCE-MANIFEST-0.1"
)

func HashWithDomain(domain HashDomain, canonicalPayload []byte) HashBytes {
	payload := make([]byte, 0, len(domain)+1+len(canonicalPayload))
	payload = append(payload, string(domain)...)
	payload = append(payload, 0)
	payload = append(payload, canonicalPayload...)
	return sha256Sum(payload)
}

func CertificateHash(canonicalCertificate []byte) HashBytes {
	return HashWithDomain(HashDomainModuleCertificate, canonicalCertificate)
}

func ExportBlockHash(exportBlock []ExportEntry) HashBytes {
	return HashWithDomain(HashDomainModuleExport, EncodeExportBlock(exportBlock))
}

func AxiomReportHash(report AxiomReport) HashBytes {
	return HashWithDomain(HashDomainAxiomReport, EncodeAxiomReport(report))
}

func RecomputeHashes(certificate *Certificate, canonicalCertificate []byte) CertificateHashes {
	return CertificateHashes{
		ExportHash:      ExportBlockHash(certificate.ExportBlock),
		AxiomReportHash: AxiomReportHash(certificate.AxiomReport),
		CertificateHash: CertificateHash(canonicalCertificate),
	}
}

func HashHex(hash HashBytes) string {
	const digits = "0123456789abcdef"
	out := make([]byte, 0, len(hash)*2)
	for _, b := range hash {
		out = append(out, digits[b>>4], digits[b&0x0f])
	}
	return string(out)
}

func EncodeExportBlock(exportBlock []ExportEntry) []byte {
	encoder := payloadEncoder{}
	encoder.writeLen(len(exportBlock))
	for _, entry := range exportBlock {
		encoder.writeU32(entry.Name)
		encoder.writeU32(entry.Declaration)
		encoder.writeHash(entry.DeclarationHash)
	}
	return encoder.bytes
}

func EncodeAxiomReport(report AxiomReport) []byte {
	encoder := payloadEncoder{}
	encoder.writeLen(len(report.Entries))
	for _, entry := range report.Entries {
		encoder.writeString(string(entry.Category))
		encoder.writeString(entry.Name)
		encoder.writeString(entry.OriginModule)
		encoder.writeHash(entry.TypeHash)
		encoder.writeHash(entry.DeclarationHash)
		encoder.writeOptionalHash(entry.SourceCertificateHash)
		encoder.writeU32Vec(entry.DirectDependentDeclarations)
		encoder.writeU32Vec(entry.TransitiveDependentDeclarations)
		encoder.writeOptionalString(entry.ApprovalProfile)
		encoder.writeOptionalString(entry.ReviewerNote)
	}

	encoder.writeLen(len(report.DeclarationDependencies))
	for _, dependencies := range report.DeclarationDependencies {
		encoder.writeString(dependencies.DeclarationName)
		encoder.writeHash(dependencies.DeclarationHash)
		encoder.writeU32Vec(dependencies.DirectAxiomDependencies)
		encoder.writeU32Vec(dependencies.TransitiveAxiomDependencies)
	}

	encoder.writeU64(report.Summary.CoreAxiomCount)
	encoder.writeU64(report.Summary.BuiltinTheoryAxiomCount)
	encoder.writeU64(report.Summary.GoSemanticsAxiomCount)
	encoder.writeU64(report.Summary.ExternalAxiomCount)
	encoder.writeU64(report.Summary.TotalAxiomCount)
	return encoder.bytes
}

type payloadEncoder struct {
	bytes []byte
}

func (e *payloadEncoder) writeU8(value uint8) {
	e.bytes = append(e.bytes, value)
}

func (e *payloadEncoder) writeBool(value bool) {
	if value {
		e.writeU8(1)
	} else {
		e.writeU8(0)
	}
}

func (e *payloadEncoder) writeU32(value uint32) {
	e.writeU64(uint64(value))
}

func (e *payloadEncoder) writeU64(value uint64) {
	for {
		b := byte(value & 0x7f)
		value >>= 7
		if value != 0 {
			b |= 0x80
		}
		e.bytes = append(e.bytes, b)
		if value == 0 {
			return
		}
	}
}

func (e *payloadEncoder) writeLen(length int) {
	e.writeU64(uint64(length))
}

func (e *payloadEncoder) writeBytes(bytes []byte) {
	e.bytes = append(e.bytes, bytes...)
}

func (e *payloadEncoder) writeBytesWithLen(bytes []byte) {
	e.writeLen(len(bytes))
	e.writeBytes(bytes)
}

func (e *payloadEncoder) writeString(value string) {
	e.writeBytesWithLen([]byte(value))
}

func (e *payloadEncoder) writeHash(hash HashBytes) {
	e.writeBytes(hash[:])
}

func (e *payloadEncoder) writeOptionalHash(hash *HashBytes) {
	if hash == nil {
		e.writeBool(false)
		return
	}
	e.writeBool(true)
	e.writeHash(*hash)
}

func (e *payloadEncoder) writeOptionalString(value *string) {
	if value == nil {
		e.writeBool(false)
		return
	}
	e.writeBool(true)
	e.writeString(*value)
}

func (e *payloadEncoder) writeU32Vec(values []uint32) {
	e.writeLen(len(values))
	for _, value := range values {
		e.writeU32(value)
	}
}

func sha256Sum(message []byte) HashBytes {
	h := [8]uint32{
		0x6a09e667,
		0xbb67ae85,
		0x3c6ef372,
		0xa54ff53a,
		0x510e527f,
		0x9b05688c,
		0x1f83d9ab,
		0x5be0cd19,
	}

	remaining := message
	for len(remaining) >= 64 {
		sha256Block(&h, remaining[:64])
		remaining = remaining[64:]
	}

	var block [128]byte
	n := copy(block[:], remaining)
	block[n] = 0x80
	if n >= 56 {
		sha256Block(&h, block[:64])
		for i := 0; i < 64; i++ {
			block[i] = 0
		}
	}

	bitLen := uint64(len(message)) * 8
	for i := 0; i < 8; i++ {
		block[63-i] = byte(bitLen >> (8 * i))
	}
	sha256Block(&h, block[:64])

	var out HashBytes
	for i, value := range h {
		out[i*4] = byte(value >> 24)
		out[i*4+1] = byte(value >> 16)
		out[i*4+2] = byte(value >> 8)
		out[i*4+3] = byte(value)
	}
	return out
}

func sha256Block(h *[8]uint32, block []byte) {
	k := [64]uint32{
		0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5,
		0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
		0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3,
		0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
		0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc,
		0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
		0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7,
		0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
		0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13,
		0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
		0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3,
		0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
		0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5,
		0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
		0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208,
		0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
	}

	var w [64]uint32
	for i := 0; i < 16; i++ {
		j := i * 4
		w[i] = uint32(block[j])<<24 |
			uint32(block[j+1])<<16 |
			uint32(block[j+2])<<8 |
			uint32(block[j+3])
	}
	for i := 16; i < 64; i++ {
		w[i] = sha256SmallSigma1(w[i-2]) + w[i-7] + sha256SmallSigma0(w[i-15]) + w[i-16]
	}

	a := h[0]
	b := h[1]
	c := h[2]
	d := h[3]
	e := h[4]
	f := h[5]
	g := h[6]
	hh := h[7]

	for i := 0; i < 64; i++ {
		t1 := hh + sha256BigSigma1(e) + sha256Choose(e, f, g) + k[i] + w[i]
		t2 := sha256BigSigma0(a) + sha256Majority(a, b, c)
		hh = g
		g = f
		f = e
		e = d + t1
		d = c
		c = b
		b = a
		a = t1 + t2
	}

	h[0] += a
	h[1] += b
	h[2] += c
	h[3] += d
	h[4] += e
	h[5] += f
	h[6] += g
	h[7] += hh
}

func sha256Choose(x uint32, y uint32, z uint32) uint32 {
	return (x & y) ^ (^x & z)
}

func sha256Majority(x uint32, y uint32, z uint32) uint32 {
	return (x & y) ^ (x & z) ^ (y & z)
}

func sha256BigSigma0(x uint32) uint32 {
	return rotateRight32(x, 2) ^ rotateRight32(x, 13) ^ rotateRight32(x, 22)
}

func sha256BigSigma1(x uint32) uint32 {
	return rotateRight32(x, 6) ^ rotateRight32(x, 11) ^ rotateRight32(x, 25)
}

func sha256SmallSigma0(x uint32) uint32 {
	return rotateRight32(x, 7) ^ rotateRight32(x, 18) ^ (x >> 3)
}

func sha256SmallSigma1(x uint32) uint32 {
	return rotateRight32(x, 17) ^ rotateRight32(x, 19) ^ (x >> 10)
}

func rotateRight32(x uint32, n uint32) uint32 {
	return (x >> n) | (x << (32 - n))
}
