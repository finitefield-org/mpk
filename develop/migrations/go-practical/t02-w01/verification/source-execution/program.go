package policy
import "example.com/policy/helper"
import "example.com/policy/shadow"
type Text string
type Index uint64
type Record struct { Number int; Text string }
func Length(s string) int { return len(s) }
func At(s Text,i Index) byte { return s[i] }
func Signed(s string,i int8) byte { return s[i] }
func Literal(i int) byte { return "\x00\xff"[i] }
func Constant() int { return len("a") }
func FromCall(i int) byte { return helper.Text()[i] }
func CallLength() int { return len(helper.Text()) }
func Shadow() int { return shadow.Length("x") }
func Compute(a,b int8,n uint64)(int8,int8,int8,int8,int8,int8,bool){return a+b,a-b,a*b,a/b,a%b,a>>n,a<b}
func Overflow64(a int64,b uint64)(int64,uint64){return a+1,b<<63}
func Convert(n int32)string{return string(n)}
func RoundBytes(s string)string{return string([]byte(s))}
func RoundRunes(s string)string{return string([]rune(s))}
func Choices(s string,i int8,flag bool)bool{return flag && s[i]==0}
func Swap(i int,a [2]int)(int,[2]int){i,a[i]=1,99;return i,a}
func Both(a [2]int)[2]int{a[0],a[1]=a[1],a[0];return a}
func Named(x int)(result int,err error){result=x;return}
func PairCall(state []int)(int,int){state[0]++;return state[0],state[0]}
func Once(state []int)(int,int){return PairCall(state)}
func Loop(n int)int{v:=0;for i:=0;i<n;i++{if i==2{continue};v+=i;if v>20{break}};return v}
func Spin(){for{}}
func Switch(x int)string{switch y:=x+1;y{default:return "d";case 1,2:return "a";case 3:return "b"}}
func Range(a [3]int)int{n:=0;for i,v:=range a{n+=i+v};return n}
func Update(s []int)int{t:=s[0:2:2];t[1]=9;u:=append(t,7);copy(s[1:],s[:2]);return len(u)+s[1]+s[2]}
func SliceLenCap(s []byte)(int,int,bool){return len(s),cap(s),s==nil}
func Aliased(a,b []int)int{a[0]=5;return b[0]}
func ArrayView(a [2]int)int{s:=a[:];s[0]=9;return a[0]}
func FieldView(r Record)Record{r.Number+=2;r.Text=r.Text+"!";return r}
func Bounds(s string,lo,hi int)string{return s[lo:hi]}
func Make(n,c int)[]int{return make([]int,n,c)}
func Shift(n int8,count int64)int8{return n<<count}
func Entry(e error)string{return e.Error()}
type Case00 int16
func(Case00)Error()string{return "case00"}
type Case01 int16
func(Case01)Error()string{return "case01"}
type Case02 int16
func(Case02)Error()string{return "case02"}
type Case03 int16
func(Case03)Error()string{return "case03"}
type Case04 int16
func(Case04)Error()string{return "case04"}
type Case05 int16
func(Case05)Error()string{return "case05"}
type Case06 int16
func(Case06)Error()string{return "case06"}
type Case07 int16
func(Case07)Error()string{return "case07"}
type Case08 int16
func(Case08)Error()string{return "case08"}
type Case09 int16
func(Case09)Error()string{return "case09"}
type Case10 int16
func(Case10)Error()string{return "case10"}
type Case11 int16
func(Case11)Error()string{return "case11"}
type Case12 int16
func(Case12)Error()string{return "case12"}
type Case13 int16
func(Case13)Error()string{return "case13"}
type Case14 int16
func(Case14)Error()string{return "case14"}
type Case15 int16
func(Case15)Error()string{return "case15"}
type Case16 int16
func(Case16)Error()string{return "case16"}
type Case17 int16
func(Case17)Error()string{return "case17"}
type Case18 int16
func(Case18)Error()string{return "case18"}
type Case19 int16
func(Case19)Error()string{return "case19"}
type Case20 int16
func(Case20)Error()string{return "case20"}
type Case21 int16
func(Case21)Error()string{return "case21"}
type Case22 int16
func(Case22)Error()string{return "case22"}
type Case23 int16
func(Case23)Error()string{return "case23"}
type Case24 int16
func(Case24)Error()string{return "case24"}
type Case25 int16
func(Case25)Error()string{return "case25"}
type Case26 int16
func(Case26)Error()string{return "case26"}
type Case27 int16
func(Case27)Error()string{return "case27"}
type Case28 int16
func(Case28)Error()string{return "case28"}
type Case29 int16
func(Case29)Error()string{return "case29"}
type Case30 int16
func(Case30)Error()string{return "case30"}
type Case31 int16
func(Case31)Error()string{return "case31"}
type Case32 int16
func(Case32)Error()string{return "case32"}
type Case33 int16
func(Case33)Error()string{return "case33"}
type Case34 int16
func(Case34)Error()string{return "case34"}
type Case35 int16
func(Case35)Error()string{return "case35"}
type Case36 int16
func(Case36)Error()string{return "case36"}
type Case37 int16
func(Case37)Error()string{return "case37"}
type Case38 int16
func(Case38)Error()string{return "case38"}
type Case39 int16
func(Case39)Error()string{return "case39"}
type Case40 int16
func(Case40)Error()string{return "case40"}
type Case41 int16
func(Case41)Error()string{return "case41"}
type Case42 int16
func(Case42)Error()string{return "case42"}
type Case43 int16
func(Case43)Error()string{return "case43"}
type Case44 int16
func(Case44)Error()string{return "case44"}
type Case45 int16
func(Case45)Error()string{return "case45"}
type Case46 int16
func(Case46)Error()string{return "case46"}
type Case47 int16
func(Case47)Error()string{return "case47"}
type Case48 int16
func(Case48)Error()string{return "case48"}
type Case49 int16
func(Case49)Error()string{return "case49"}
type Case50 int16
func(Case50)Error()string{return "case50"}
type Case51 int16
func(Case51)Error()string{return "case51"}
type Case52 int16
func(Case52)Error()string{return "case52"}
type Case53 int16
func(Case53)Error()string{return "case53"}
type Case54 int16
func(Case54)Error()string{return "case54"}
type Case55 int16
func(Case55)Error()string{return "case55"}
type Case56 int16
func(Case56)Error()string{return "case56"}
type Case57 int16
func(Case57)Error()string{return "case57"}
type Case58 int16
func(Case58)Error()string{return "case58"}
type Case59 int16
func(Case59)Error()string{return "case59"}
type Case60 int16
func(Case60)Error()string{return "case60"}
type Case61 int16
func(Case61)Error()string{return "case61"}
type Case62 int16
func(Case62)Error()string{return "case62"}
type Case63 int16
func(Case63)Error()string{return "case63"}
type Case64 int16
func(Case64)Error()string{return "case64"}
type Case65 int16
func(Case65)Error()string{return "case65"}
type Case66 int16
func(Case66)Error()string{return "case66"}
type Case67 int16
func(Case67)Error()string{return "case67"}
type Case68 int16
func(Case68)Error()string{return "case68"}
type Case69 int16
func(Case69)Error()string{return "case69"}
