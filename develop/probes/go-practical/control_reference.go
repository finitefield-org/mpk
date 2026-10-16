// Private observations of actual for/continue/break/return and full value frames.
package main

import (
	"encoding/json"
	"os"
	"strconv"
)

type encodedFrame struct {
	Low  string `json:"low"`
	High string `json:"high"`
}
type frame [2]uint64

func decode(x encodedFrame) frame {
	lo, e := strconv.ParseUint(x.Low, 10, 64)
	if e != nil {
		panic(e)
	}
	hi, e := strconv.ParseUint(x.High, 10, 64)
	if e != nil {
		panic(e)
	}
	return frame{lo, hi}
}
func encode(s frame) encodedFrame {
	return encodedFrame{strconv.FormatUint(s[0], 10), strconv.FormatUint(s[1], 10)}
}
func rest(s frame) frame { return frame{s[0] ^ 128, s[1] ^ 1} }
func post(s frame) frame { return frame{s[0] + 2, s[1] ^ (uint64(1) << 63)} }
func next(s frame, bad bool) frame {
	s = post(rest(s))
	if bad {
		s[0] ^= 1
	}
	return s
}
func invariant(s frame) bool { return s[0]&1 == 0 }

type input struct {
	N           int          `json:"n"`
	State       encodedFrame `json:"state"`
	ReturnFrame encodedFrame `json:"return_frame"`
	BreakFrame  encodedFrame `json:"break_frame"`
	Returned    bool         `json:"returned"`
	Broken      bool         `json:"broken"`
	Continued   bool         `json:"continued"`
	BadStep     bool         `json:"bad_step"`
}
type output struct {
	Iteration      encodedFrame `json:"iteration"`
	Invariant      bool         `json:"invariant"`
	Absorbed       encodedFrame `json:"absorbed"`
	Edge           encodedFrame `json:"edge"`
	RestCalls      int          `json:"rest_calls"`
	PostCalls      int          `json:"post_calls"`
	InputUnchanged bool         `json:"input_unchanged"`
}

func observe(r input) output {
	s := decode(r.State)
	before := s
	a := s
	h := s
	for i := 0; i < r.N; i++ {
		a = next(a, r.BadStep)
		if h[0]&1 == 0 {
			h = next(h, r.BadStep)
		}
	}
	edge, restCalls, postCalls := func() (frame, int, int) {
		body := s
		restCalls, postCalls := 0, 0
		for i := 0; i < 1; func() { body = post(body); postCalls++ }() {
			i++
			if r.Returned {
				return decode(r.ReturnFrame), restCalls, postCalls
			}
			if r.Broken {
				body = decode(r.BreakFrame)
				break
			}
			if r.Continued {
				continue
			}
			body = rest(body)
			restCalls++
		}
		return body, restCalls, postCalls
	}()
	return output{encode(a), invariant(a), encode(h), encode(edge), restCalls, postCalls, s == before}
}
func main() {
	var rows []input
	f := os.Stdin
	if len(os.Args) == 2 {
		p, e := os.Open(os.Args[1])
		if e != nil {
			panic(e)
		}
		defer p.Close()
		f = p
	}
	if e := json.NewDecoder(f).Decode(&rows); e != nil {
		panic(e)
	}
	out := make([]output, len(rows))
	for i, r := range rows {
		out[i] = observe(r)
	}
	if e := json.NewEncoder(os.Stdout).Encode(out); e != nil {
		panic(e)
	}
}
