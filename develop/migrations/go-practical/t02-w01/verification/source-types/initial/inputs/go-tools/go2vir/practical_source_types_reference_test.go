//go:build go1.25 && (linux || darwin)

package main

import (
 "bytes"
 "fmt"
 "testing"
)

func TestPracticalOriginalTypeDeclarationsReference(t *testing.T){
 files:=map[string][]byte{
 "go.mod":[]byte("module example.com/policy\ngo 1.25.0\n"),
 "policy.go":[]byte(`package policy
import h "example.com/policy/helper"
const (Size=1<<iota; Size2; Size4)
type Word uint16
type Alias=Word
type Base [Size2]Alias
type Wrap Base
type Chain=Wrap
type Anon=struct{ private Word "\xff"; É byte; ǅ byte; ᴬ byte; _ bool; _ bool }
type ArrayAlias=[2]Word
type SliceAlias=[]Word
type Empty struct{}
type Composite struct{
 GroupA,GroupB Alias "\000"
 Anon
 ArrayAlias
 SliceAlias
 E error
 Foreign h.Record
 Folded [(1<<100)>>99]Word
 Imported [h.Size]byte
 Zero [0]Empty
 Tags struct{ Raw string `+"`line\r\nnext`"+`; Empty string ""; Missing string }
}
func Observe(root Composite, wrap Wrap, chain Chain, array ArrayAlias, view SliceAlias, empty Empty, builtin error){
 _ = root
 _ = wrap
 _ = chain
 _ = array
 _ = view
 _ = empty
 _ = builtin
}
func init(){panic("application must not execute")}
`),
 "helper/helper.go":[]byte("package helper\nconst Size=3\ntype Record struct{private uint16; Exported byte}\n"),
 }
 owner,original:=practicalStorageTestOwner(t,files)
 for n,expression:=range []string{"root","wrap","chain","array","view","empty","builtin"}{
  keys,err:=owner.typeKeys("example.com/policy",practicalStorageTestSpan(t,original["policy.go"],expression));if err!=nil{t.Fatal(expression,err)}
  buffers:=[]map[string]any{};for _,b:=range keys.sources{buffers=append(buffers,map[string]any{"object":b.object,"raw":b.sourceBytes()})}
  retainPracticalSyntaxEvidence(t,fmt.Sprintf("source-types-%02d.json",n),map[string]any{"canonical_components":bytes.Clone(keys.canonical),"sources":buffers,"source_typing_or_context_or_program_authority":false})
 }
}
