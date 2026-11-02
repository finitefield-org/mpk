package main

type Receiver struct{ Value int }

func Named(_ int, input int) (out int, _ bool) {
	out = input
	return
}

func ReadZero(input int) (out [2]int, text string, err error) {
	var local int
	local += input
	out[0] = local
	return
}

func (_ Receiver) Hidden(_ int) (int, bool) { return 7, true }

func Shadow(x int) (result int) {
	{
		x := x + 1
		result = x
	}
	return
}

// These original bodies remain present and type-check in the native project.
func UnusedCallback(ch chan int, f func()) {
	go func() { ch <- 1; f() }()
}

func UnusedMap() {
	m := map[string]int{"x": 1}
	println(m)
}

func main() {
	a, b := Named(99, 41)
	println("named", a, b)
	array, text, err := ReadZero(8)
	println("zero", array[0], array[1], len(text), err == nil)
	a, b = (Receiver{Value: 13}).Hidden(45)
	println("hidden", a, b)
	println("shadow", Shadow(5))
}
