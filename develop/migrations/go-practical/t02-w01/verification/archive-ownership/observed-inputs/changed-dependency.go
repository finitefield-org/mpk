package dep
type Amount int64
func Twice(x int64) int64 { return x+x }
var Mutable int64 = 7
func init() { panic("application init must never execute in the probe") }

// caller-owned changed dependency
