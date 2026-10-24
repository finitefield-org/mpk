package main
import("fmt";"unicode")
func main(){
 if unicode.Version!="15.0.0"{panic(unicode.Version)}
 fmt.Println("// Exact Go1.25.0 Unicode15.0.0 IsGraphic ranges; generated from the pinned native table.")
 fmt.Println("pub(super) fn graphic(c:char)->bool { let n=c as u32; RANGES.binary_search_by(|&(lo,hi)| if n<lo {std::cmp::Ordering::Greater}else if n>hi {std::cmp::Ordering::Less}else{std::cmp::Ordering::Equal}).is_ok() }")
 fmt.Println("const RANGES:&[(u32,u32)]=&[")
 start:=-1
 for r:=0;r<=unicode.MaxRune+1;r++{
  yes:=r<=unicode.MaxRune&&unicode.IsGraphic(rune(r))
  if yes&&start<0{start=r};if !yes&&start>=0{fmt.Printf("(0x%04x,0x%04x),\n",start,r-1);start=-1}
 }
 fmt.Println("];")
}
