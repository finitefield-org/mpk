package array

type Pair64 struct {
	Left  int64
	Right int64
}

type Triple64 struct {
	First  int64
	Second int64
	Third  int64
}

type PairU64 struct {
	Left  uint64
	Right uint64
}

type PairArray64 struct {
	Values [2]int64
	Extra  int64
}

func BuildPair64(left int64, right int64) Pair64 {
	return Pair64{Left: left, Right: right}
}

func SumPair64(pair Pair64) int64 {
	return pair.Left + pair.Right
}

func PickPairLeft(pair Pair64) int64 {
	return pair.Left
}

func PickPairRight(pair Pair64) int64 {
	return pair.Right
}

func SwapPair64(pair Pair64) Pair64 {
	return Pair64{Left: pair.Right, Right: pair.Left}
}

func BuildTriple64(first int64, second int64, third int64) Triple64 {
	return Triple64{First: first, Second: second, Third: third}
}

func SumTriple64(triple Triple64) int64 {
	return triple.First + triple.Second + triple.Third
}

func PickTripleFirst(triple Triple64) int64 {
	return triple.First
}

func PickTripleSecond(triple Triple64) int64 {
	return triple.Second
}

func PickTripleThird(triple Triple64) int64 {
	return triple.Third
}

func BuildArray2(first int64, second int64) [2]int64 {
	return [2]int64{first, second}
}

func BuildArray3(first int64, second int64, third int64) [3]int64 {
	return [3]int64{first, second, third}
}

func FirstArray2(values [2]int64) int64 {
	return values[0]
}

func SecondArray2(values [2]int64) int64 {
	return values[1]
}

func SumArray2(values [2]int64) int64 {
	return values[0] + values[1]
}

func FirstArray3(values [3]int64) int64 {
	return values[0]
}

func SecondArray3(values [3]int64) int64 {
	return values[1]
}

func ThirdArray3(values [3]int64) int64 {
	return values[2]
}

func SumArray3(values [3]int64) int64 {
	return values[0] + values[1] + values[2]
}

func BuildUArray2(first uint64, second uint64) [2]uint64 {
	return [2]uint64{first, second}
}

func FirstUArray2(values [2]uint64) uint64 {
	return values[0]
}

func SecondUArray2(values [2]uint64) uint64 {
	return values[1]
}

func SumUArray2(values [2]uint64) uint64 {
	return values[0] + values[1]
}

func BuildBoolArray2(first bool, second bool) [2]bool {
	return [2]bool{first, second}
}

func FirstBoolArray2(values [2]bool) bool {
	return values[0]
}

func SecondBoolArray2(values [2]bool) bool {
	return values[1]
}

func AllBoolArray2(values [2]bool) bool {
	return values[0] && values[1]
}

func AnyBoolArray2(values [2]bool) bool {
	return values[0] || values[1]
}

func BuildPairArray64(first int64, second int64, extra int64) PairArray64 {
	return PairArray64{Values: [2]int64{first, second}, Extra: extra}
}

func SumPairArray64(value PairArray64) int64 {
	return value.Values[0] + value.Values[1] + value.Extra
}

func SelectArrayValue(flag bool, values [2]int64) int64 {
	selected := values[1]
	if flag {
		selected = values[0]
	}
	return selected
}

func AddArrayFirst(left [2]int64, right [2]int64) int64 {
	return left[0] + right[0]
}

func BuildPairU64(left uint64, right uint64) PairU64 {
	return PairU64{Left: left, Right: right}
}
