package bad
type A [1]byte
type B [1]byte
func bad(a A,b B)bool{return a==b}
