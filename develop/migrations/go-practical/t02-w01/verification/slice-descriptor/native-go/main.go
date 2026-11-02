package main
import("encoding/json";"os")
type Input struct {ID string `json:"id"`;Nil bool `json:"nil"`;Offset,Length,Capacity,Index,Low,High,Max uint64;Bytes []int}
type Output struct {ID string `json:"id"`;Nil bool `json:"nil"`;Length int `json:"length"`;Capacity int `json:"capacity"`;IndexUnsigned bool `json:"index_unsigned"`;IndexSigned bool `json:"index_signed"`;TwoUnsigned bool `json:"two_unsigned"`;TwoSigned bool `json:"two_signed"`;ThreeUnsigned bool `json:"three_unsigned"`;ThreeSigned bool `json:"three_signed"`}
func capture(f func())(ok bool){defer func(){if recover()!=nil{ok=false}}();f();return true}
func observe(in Input)Output{
 var a [256]byte;if len(in.Bytes)!=256{panic("whole input")};for i,v:=range in.Bytes{if v<0||v>255{panic("byte")};a[i]=byte(v)}
 var s []byte;if in.Nil{if in.Offset!=0||in.Length!=0||in.Capacity!=0{panic("nil input")}}else{s=a[in.Offset:in.Offset+in.Length:in.Offset+in.Capacity]}
 out:=Output{ID:in.ID,Nil:s==nil,Length:len(s),Capacity:cap(s)}
 out.IndexUnsigned=capture(func(){_=s[in.Index]});out.IndexSigned=capture(func(){_=s[int64(in.Index)]})
 out.TwoUnsigned=capture(func(){_=s[in.Low:in.High]});out.TwoSigned=capture(func(){_=s[int64(in.Low):int64(in.High)]})
 out.ThreeUnsigned=capture(func(){_=s[in.Low:in.High:in.Max]});out.ThreeSigned=capture(func(){_=s[int64(in.Low):int64(in.High):int64(in.Max)]})
 return out
}
func main(){var rows []Input;if err:=json.NewDecoder(os.Stdin).Decode(&rows);err!=nil{panic(err)};out:=make([]Output,0,len(rows));for _,r:=range rows{out=append(out,observe(r))};if err:=json.NewEncoder(os.Stdout).Encode(out);err!=nil{panic(err)}}
