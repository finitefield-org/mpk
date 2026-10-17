// Independent pinned-Go big-integer interpreter for closed logical observations.
package main

import (
	"encoding/json"
	"fmt"
	"io"
	"math/big"
	"os"
	"strings"
)

type expression struct {
	Op      string         `json:"op"`
	Type    map[string]any `json:"type"`
	Payload payload        `json:"payload"`
}
type payload struct {
	Value any          `json:"value"`
	Args  []expression `json:"args"`
	Depth int          `json:"depth"`
	Lower *expression  `json:"lower"`
	Upper *expression  `json:"upper"`
	Body  *expression  `json:"body"`
}
type query struct {
	ID         string     `json:"id"`
	Expression expression `json:"expression"`
	Large      bool       `json:"large"`
}
type value struct {
	ty string
	b  bool
	n  *big.Int
}

func boolean(b bool) value { return value{ty: "Bool", b: b} }
func number(ty string, n *big.Int) value {
	return value{ty: ty, n: new(big.Int).Set(n)}
}
func evaluate(e expression, scope []*big.Int) value {
	p := e.Payload
	if e.Op == "bool.literal" {
		return boolean(p.Value.(bool))
	}
	if e.Op == "nat.literal" || e.Op == "int.literal" {
		n, ok := new(big.Int).SetString(p.Value.(string), 10)
		if !ok {
			panic("decimal")
		}
		ty := "Nat"
		if e.Op == "int.literal" {
			ty = "Int"
		}
		return number(ty, n)
	}
	if e.Op == "bound" {
		return number("Nat", scope[len(scope)-1-p.Depth])
	}
	if e.Op == "forall" || e.Op == "exists" || e.Op == "nat.sum" || e.Op == "int.sum" {
		lo, hi := evaluate(*p.Lower, scope).n, evaluate(*p.Upper, scope).n
		result := boolean(e.Op == "forall")
		if strings.HasSuffix(e.Op, ".sum") {
			ty := "Nat"
			if e.Op == "int.sum" {
				ty = "Int"
			}
			result = number(ty, new(big.Int))
		}
		count := 0
		for n := new(big.Int).Set(lo); n.Cmp(hi) < 0; n.Add(n, big.NewInt(1)) {
			count++
			if count > 1024 {
				panic("private native observation resource ceiling")
			}
			inner := append(append([]*big.Int{}, scope...), new(big.Int).Set(n))
			x := evaluate(*p.Body, inner)
			if e.Op == "forall" {
				result.b = result.b && x.b
			} else if e.Op == "exists" {
				result.b = result.b || x.b
			} else {
				result.n.Add(result.n, x.n)
			}
		}
		return result
	}
	args := make([]value, len(p.Args))
	for i, arg := range p.Args {
		args[i] = evaluate(arg, scope)
	}
	if e.Op == "if" {
		if args[0].b {
			return args[1]
		}
		return args[2]
	}
	a := args[0]
	switch e.Op {
	case "bool.not":
		return boolean(!a.b)
	case "int.neg":
		return number("Int", new(big.Int).Neg(a.n))
	case "nat.to_int":
		return number("Int", a.n)
	}
	b := args[1]
	switch e.Op {
	case "bool.and":
		return boolean(a.b && b.b)
	case "bool.or":
		return boolean(a.b || b.b)
	case "bool.implies":
		return boolean(!a.b || b.b)
	case "eq", "ne", "lt", "le", "gt", "ge":
		comparison := 0
		if a.ty == "Bool" {
			if a.b != b.b {
				comparison = 1
			}
		} else {
			comparison = a.n.Cmp(b.n)
		}
		return boolean(map[string]bool{"eq": comparison == 0, "ne": comparison != 0, "lt": comparison < 0, "le": comparison <= 0, "gt": comparison > 0, "ge": comparison >= 0}[e.Op])
	}
	n := new(big.Int)
	switch e.Op {
	case "nat.add", "int.add":
		n.Add(a.n, b.n)
	case "nat.mul", "int.mul":
		n.Mul(a.n, b.n)
	case "nat.sub", "int.sub":
		n.Sub(a.n, b.n)
		if e.Op == "nat.sub" && n.Sign() < 0 {
			n.SetInt64(0)
		}
	default:
		panic(fmt.Sprintf("unregistered logical observer operation: %s", e.Op))
	}
	return number(a.ty, n)
}
func main() {
	d := json.NewDecoder(os.Stdin)
	d.DisallowUnknownFields()
	var qs []query
	if err := d.Decode(&qs); err != nil {
		panic(err)
	}
	var trailing any
	if err := d.Decode(&trailing); err != io.EOF {
		panic("one query document")
	}
	out := make([]map[string]any, len(qs))
	for i, q := range qs {
		x := evaluate(q.Expression, nil)
		v := any(x.b)
		if x.ty != "Bool" {
			v = x.n.String()
		}
		out[i] = map[string]any{"id": q.ID, "type": x.ty, "value": v}
	}
	if err := json.NewEncoder(os.Stdout).Encode(out); err != nil {
		panic(err)
	}
}
