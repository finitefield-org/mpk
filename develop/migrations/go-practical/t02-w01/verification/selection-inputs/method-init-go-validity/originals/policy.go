package policy
type Count uint16
func(c Count)init()uint16{return uint16(c)+1}
func init(){panic("must not execute")}
func Unused(c chan int,f func(int)int)int{return f(<-c)}
