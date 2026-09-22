package main
import("encoding/json";"os"; checker "github.com/finitefield-org/mpk/go-tools/mpk-checker-ref")
func main(){b,e:=os.ReadFile(os.Args[1]);if e!=nil{panic(e)};r,e:=checker.VerifyCertificateBytes(b);if e!=nil{v,ok:=e.(*checker.VerifyError);if !ok{panic(e)};json.NewEncoder(os.Stdout).Encode(map[string]any{"verdict":"rejected","error_kind":v.Kind,"error_detail":v.Detail,"certificate":checker.HashHex(checker.CertificateHash(b))});os.Exit(1)};json.NewEncoder(os.Stdout).Encode(map[string]any{"verdict":"accepted","report":r})}
