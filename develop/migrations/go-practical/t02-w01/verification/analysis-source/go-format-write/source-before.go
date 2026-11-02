//go:build go1.25 && (linux || darwin)

package main

import (
	"encoding/hex"
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"
	"sort"
	"strings"
	"testing"
)

// Observe all original frontend and vendor source bytes, without claiming
// that the caller-supplied snapshot authenticates an original filesystem.
func TestPracticalAnalysisSourceDifferential(t *testing.T) {
	roots:=practicalAnalysisTestRoots(t, "");session:=practicalTestSession(t,roots)
	inputs,err:=session.capturePracticalAnalysis(practicalModuleLocation{capability:roots[0].Capability},practicalModuleLocation{capability:roots[1].Capability});if err!=nil {t.Fatal(err)}
	original,err:=session.seal();if err!=nil {t.Fatal(err)}
	wire,err:=original.snapshotInputs();if err!=nil || len(wire.sources)!=213 || len(inputs.sources)!=65 || len(inputs.vendor)!=133 {t.Fatalf("original analysis owner differs: %v",err)}
	const filename="practical_analysis.go"
	base,err:=original.readSource(roots[0].Capability,filename,"go_file");if err!=nil {t.Fatal(err)}
	type probe struct{name,appendText,entry string; mode os.FileMode; unread,want bool}
	probes:=[]probe{
		{name:"original",want:true},
		{name:"embed-argument",appendText:"\n//go:embed asset.txt\n"},
		{name:"embed-without-argument",appendText:"\n//go:embed\n"},
		{name:"embed-prefix",appendText:"\n//go:embedding\n"},
		{name:"embed-stripped-carriage-return",appendText:"\n//go:\rembed asset.txt\n"},
		{name:"embed-after-code",appendText:"\nvar mpkEmbedProbe = 1 //go:embed x\n"},
		{name:"embed-in-quoted-string",appendText:"\nconst mpkEmbedProbe = \"//go:embed x\"\n",want:true},
		{name:"embed-in-raw-string",appendText:"\nconst mpkEmbedProbe = `//go:embed x`\n",want:true},
		{name:"embed-in-block-comment",appendText:"\n/* first\n//go:embed x\n*/\n",want:true},
		{name:"embed-after-other-comment-text",appendText:"\n// text //go:embed x\n",want:true},
		{name:"spaced-comment",appendText:"\n// go:embed x\n",want:true},
		{name:"invalid-syntax",appendText:"\nfunc ( }\n"},
		{name:"unclosed-comment",appendText:"\n/* unterminated"},
		{name:"unread-frontend",unread:true},
		{name:"unread-test-go",entry:"missing_test.go",want:true},
		{name:"unknown-asset",entry:"unread.data",want:true},
		{name:"unmodeled-assembly",entry:"unread.S"},
		{name:"symlink-frontend",entry:"ghost.go",mode:os.ModeSymlink},
		{name:"nonutf8-unrelated",entry:string([]byte{0xff})+".data",want:true},
		{name:"nonutf8-go",entry:string([]byte{0xff})+".go"},
	}
	walk:=func(snapshot practicalSourceSnapshot)(int,error){
		entries,err:=snapshot.readDirectory(roots[0].Capability,"");if err!=nil{return 0,err};count:=0
		for _,entry:=range entries {
			for _,suffix:=range []string{".s",".S",".h",".c",".cc",".cpp",".cxx",".m",".f",".F",".for",".f90",".syso",".swig",".swigcxx"} {if strings.HasSuffix(entry.Name,suffix){return 0,fmt.Errorf("unmodeled native input")}}
			if strings.HasSuffix(entry.Name,".go")&&!strings.HasSuffix(entry.Name,"_test.go") {source,err:=snapshot.readSource(roots[0].Capability,entry.Name,"go_file");if err!=nil{return 0,err};if err:=validatePracticalAnalysisGoSource(source);err!=nil{return 0,err};count++}
		}
		for _,old:=range inputs.vendor {if old.object.Kind!="go_file"{continue};source,err:=snapshot.readSource(roots[0].Capability,old.object.Path,"go_file");if err!=nil{return 0,err};if err:=validatePracticalAnalysisGoSource(source);err!=nil{return 0,err};count++}
		return count,nil
	}
	type sourceRecord struct{Object practicalSourceObject `json:"object"`;RawFile string `json:"raw_file"`}
	type observation struct{Name string `json:"name"`;Snapshot string `json:"snapshot_hex"`;SourceIDs []string `json:"source_ids"`;Changed *sourceRecord `json:"changed"`;Accepted bool `json:"accepted"`;Sources int `json:"sources"`}
	out:=os.Getenv("MPK_GO_ANALYSIS_SOURCE_OUTPUT");if out!="" {if err:=os.MkdirAll(filepath.Join(out,"changed-bytes"),0o700);err!=nil{t.Fatal(err)}}
	var rows []observation
	for _,probe:=range probes {
		candidate:=practicalSourceSnapshot{buffers:map[string]practicalSourceBuffer{},paths:map[string]string{},directories:map[string][]practicalDirectoryEntry{},ids:append([]string(nil),original.ids...),roots:append([]string(nil),original.roots...)}
		for k,v:=range original.buffers{candidate.buffers[k]=v};for k,v:=range original.paths{candidate.paths[k]=v};for k,v:=range original.directories{candidate.directories[k]=append([]practicalDirectoryEntry(nil),v...)}
		var changed *sourceRecord
		if probe.unread||probe.appendText!="" {
			key:=practicalCaptureKey(roots[0].Capability,filename);old:=candidate.paths[key];delete(candidate.paths,key);delete(candidate.buffers,old)
			ids:=make([]string,0,len(candidate.ids));for _,id:=range candidate.ids{if id!=old{ids=append(ids,id)}};candidate.ids=ids
			if !probe.unread {data:=append(base.sourceBytes(),[]byte(probe.appendText)...);source,err:=newPracticalSourceBuffer(roots[0].Capability,filename,"go_file",data);if err!=nil{t.Fatal(err)};candidate.paths[key]=source.object.ID;candidate.buffers[source.object.ID]=source;candidate.ids=append(candidate.ids,source.object.ID);sort.Strings(candidate.ids);name:=source.object.RawSHA256+".raw";changed=&sourceRecord{source.object,name};if out!="" {if err:=os.WriteFile(filepath.Join(out,"changed-bytes",name),source.sourceBytes(),0o600);err!=nil{t.Fatal(err)}}}
		}
		if probe.entry!="" {key:=practicalCaptureKey(roots[0].Capability,"");candidate.directories[key]=append(candidate.directories[key],practicalDirectoryEntry{Name:probe.entry,Type:probe.mode});sort.Slice(candidate.directories[key],func(i,j int)bool{return candidate.directories[key][i].Name<candidate.directories[key][j].Name})}
		candidate.contentSHA256=practicalSnapshotDigest(candidate);packet,err:=candidate.snapshotInputs();if err!=nil{t.Fatalf("%s snapshot: %v",probe.name,err)}
		count,err:=walk(candidate);if (err==nil)!=probe.want || (err==nil&&count!=191) {t.Fatalf("%s: count=%d error=%v want=%v",probe.name,count,err,probe.want)}
		rows=append(rows,observation{probe.name,hex.EncodeToString(packet.canonical),candidate.ids,changed,err==nil,count})
	}
	if out!="" {data,err:=json.MarshalIndent(map[string]any{"cases":rows,"original_sources":len(wire.sources),"frontend_go_files":len(inputs.sources),"vendor_files":len(inputs.vendor),"root_or_binary_authority_claimed":false},"","  ");if err!=nil{t.Fatal(err)};if err:=os.WriteFile(filepath.Join(out,"native.json"),append(data,'\n'),0o600);err!=nil{t.Fatal(err)}}
}
