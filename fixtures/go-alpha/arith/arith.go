package arith

func Add64(a int64, b int64) int64 {
	return a + b
}

func Sub64(a int64, b int64) int64 {
	return a - b
}

func Mul64(a int64, b int64) int64 {
	return a * b
}

func Neg64(a int64) int64 {
	return -a
}

func Add32(a int32, b int32) int32 {
	return a + b
}

func Sub32(a int32, b int32) int32 {
	return a - b
}

func Mul32(a int32, b int32) int32 {
	return a * b
}

func Neg32(a int32) int32 {
	return -a
}

func Add16(a int16, b int16) int16 {
	return a + b
}

func Sub16(a int16, b int16) int16 {
	return a - b
}

func Add8(a int8, b int8) int8 {
	return a + b
}

func Sub8(a int8, b int8) int8 {
	return a - b
}

func AddU64(a uint64, b uint64) uint64 {
	return a + b
}

func SubU64(a uint64, b uint64) uint64 {
	return a - b
}

func MulU64(a uint64, b uint64) uint64 {
	return a * b
}

func AndU64(a uint64, b uint64) uint64 {
	return a & b
}

func OrU64(a uint64, b uint64) uint64 {
	return a | b
}

func XorU64(a uint64, b uint64) uint64 {
	return a ^ b
}

func NotU64(a uint64) uint64 {
	return ^a
}

func ShiftLeftU64(value uint64, amount uint8) uint64 {
	return value << amount
}

func ShiftRightU64(value uint64, amount uint8) uint64 {
	return value >> amount
}

func AndU8(a uint8, b uint8) uint8 {
	return a & b
}

func OrU8(a uint8, b uint8) uint8 {
	return a | b
}

func XorU8(a uint8, b uint8) uint8 {
	return a ^ b
}

func NotU8(a uint8) uint8 {
	return ^a
}

func Less64(a int64, b int64) bool {
	return a < b
}

func LessEqual64(a int64, b int64) bool {
	return a <= b
}

func Greater64(a int64, b int64) bool {
	return a > b
}

func GreaterEqual64(a int64, b int64) bool {
	return a >= b
}

func Equal64(a int64, b int64) bool {
	return a == b
}

func NotEqual64(a int64, b int64) bool {
	return a != b
}

func BoolAnd(left bool, right bool) bool {
	return left && right
}

func BoolOr(left bool, right bool) bool {
	return left || right
}

func BoolNot(value bool) bool {
	return !value
}
