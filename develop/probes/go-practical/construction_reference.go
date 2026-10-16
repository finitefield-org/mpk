// Actual make/append; raw model fields do not grant Go source admission.
package main

import (
	"encoding/json"
	"fmt"
	"math/bits"
	"os"
	"strconv"
)

type buildCell struct {
	Amount int64
	Tag    uint64
}
type buildView struct {
	Offset   string `json:"offset"`
	Length   string `json:"length"`
	Capacity string `json:"capacity"`
	Nil      bool   `json:"nil"`
	Region   bool   `json:"region"`
}
type buildInput struct {
	Kind        string    `json:"kind"`
	Profile     int       `json:"profile"`
	Storage     string    `json:"storage"`
	Length      string    `json:"length"`
	Capacity    string    `json:"capacity"`
	FreshRegion bool      `json:"fresh_region"`
	QueryRegion bool      `json:"query_region"`
	Live        [2]bool   `json:"live"`
	Index       string    `json:"index"`
	Destination buildView `json:"destination"`
	Source      buildView `json:"source"`
}

func buildN(s string) int64 {
	n, e := strconv.ParseInt(s, 10, 64)
	if e != nil {
		panic(e)
	}
	return n
}
func buildR(r bool) int {
	if r {
		return 1
	}
	return 0
}
func buildV(r bool, i uint64) buildCell {
	var seed uint64
	if r {
		seed = 0x6a09e667f3bcc909
	}
	return buildCell{int64(seed ^ ((i + 1) * 0x9e3779b97f4a7c15)), (seed ^ 0xa5a5a5a5a5a5a5a5) + i*0x100000001}
}
func buildT(v buildCell) string { return fmt.Sprintf("%016x:%016x", uint64(v.Amount), v.Tag) }
func buildH(n int) [2][]buildCell {
	var h [2][]buildCell
	for r := range h {
		h[r] = make([]buildCell, n)
		for i := range h[r] {
			h[r][i] = buildV(r != 0, uint64(i))
		}
	}
	return h
}
func buildRows(h [2][]buildCell) [2][]string {
	var out [2][]string
	for r := range h {
		out[r] = buildTexts(h[r])
	}
	return out
}
func buildTexts(v []buildCell) []string {
	out := make([]string, len(v))
	for i, x := range v {
		out[i] = buildT(x)
	}
	return out
}
func buildValid(v buildView, n int64, live [2]bool) bool {
	o, l, k := buildN(v.Offset), buildN(v.Length), buildN(v.Capacity)
	if v.Nil {
		return l == 0 && k == 0
	}
	end, c := bits.Add64(uint64(o), uint64(k), 0)
	return live[buildR(v.Region)] && o >= 0 && l >= 0 && k >= 0 && n >= 0 && uint64(l) <= uint64(k) && c == 0 && end <= uint64(n)
}
func buildConstruct(h [2][]buildCell, v buildView) (s []buildCell, ok, panicked bool) {
	defer func() {
		if recover() != nil {
			s = nil
			ok = false
			panicked = true
		}
	}()
	o, l, k := buildN(v.Offset), buildN(v.Length), buildN(v.Capacity)
	if v.Nil {
		if l != 0 || k != 0 {
			return nil, false, false
		}
		return nil, true, false
	}
	return h[buildR(v.Region)][o : o+l : o+k], true, false
}
func buildRead(v []buildCell, i int64) (s string, ok, panicked bool) {
	defer func() {
		if recover() != nil {
			s = ""
			ok = false
			panicked = true
		}
	}()
	return buildT(v[i]), true, false
}
func buildMake(l, k int64) (v []buildCell, ok, panicked bool) {
	defer func() {
		if recover() != nil {
			v = nil
			ok = false
			panicked = true
		}
	}()
	return make([]buildCell, l, k), true, false
}
func buildPad(b int) uint64 {
	p := uint64(1)
	for p < uint64(b) {
		p *= 2
	}
	return p
}
func makeObservation(r buildInput) map[string]any {
	n, l, k, i := buildN(r.Storage), buildN(r.Length), buildN(r.Capacity), buildN(r.Index)
	if n < 0 || n > 257 {
		panic("private old storage")
	}
	if l >= 0 && k >= l && k > 257 {
		panic("no huge allocation workload")
	}
	fresh := !r.Live[buildR(r.FreshRegion)]
	guard := l >= 0 && k >= 0 && l <= k
	modeled := buildV(r.QueryRegion, uint64(i)&(buildPad(r.Profile)-1))
	if r.QueryRegion == r.FreshRegion && uint64(i) < uint64(k) {
		modeled = buildCell{}
	}
	modeledLive := r.Live[buildR(r.QueryRegion)] || r.QueryRegion == r.FreshRegion
	h := buildH(int(n))
	before := ""
	if i >= 0 && i < n {
		before = buildT(h[buildR(r.QueryRegion)][i])
	}
	v, ok, panicked := buildMake(l, k)
	full := v[:cap(v)]
	value, read, rp := "", false, false
	tailValue, fullRead, fp := "", false, false
	if ok {
		value, read, rp = buildRead(v, i)
		tailValue, fullRead, fp = buildRead(full, i)
	}
	after := ""
	if i >= 0 && i < n {
		after = buildT(h[buildR(r.QueryRegion)][i])
	}
	capOut, lenOut := "", ""
	if ok {
		capOut = strconv.Itoa(cap(v))
		lenOut = strconv.Itoa(len(v))
	}
	modelDescriptor := buildView{"0", r.Length, r.Capacity, false, r.FreshRegion}
	postLive := r.Live
	postLive[buildR(r.FreshRegion)] = true
	return map[string]any{"kind": "make", "guard": guard, "fresh": fresh, "model_valid": buildValid(modelDescriptor, k, postLive), "storage_bound": uint64(k) <= uint64(r.Profile), "old_storage_bound": uint64(n) <= uint64(r.Profile), "query_covered": uint64(i) == (uint64(i) & (buildPad(r.Profile) - 1)), "query_live": r.Live[buildR(r.QueryRegion)], "new_query_live": modeledLive, "model_query_after": buildT(modeled), "borrowed_before": before, "borrowed_after": after,
		"constructed": ok, "make_panicked": panicked, "actual_nil": ok && v == nil, "actual_length": lenOut, "actual_capacity": capOut, "actual_values": buildTexts(v), "full_values": buildTexts(full), "read_defined": read, "read_panicked": rp, "full_read_defined": fullRead, "full_read_panicked": fp, "actual_value": value, "full_value": tailValue}
}
func appendObservation(r buildInput) map[string]any {
	n, i := buildN(r.Storage), buildN(r.Index)
	if n < 0 || n > 257 {
		panic("private storage")
	}
	d, s := r.Destination, r.Source
	dl, sl := uint64(buildN(d.Length)), uint64(buildN(s.Length))
	total, c := bits.Add64(dl, sl, 0)
	do := uint64(buildN(d.Offset))
	start, sc := bits.Add64(do, dl, 0)
	end, ec := bits.Add64(do, total, 0)
	norealloc := c == 0 && total <= uint64(buildN(d.Capacity))
	newDescriptor := d
	newDescriptor.Length = strconv.FormatInt(int64(total), 10)
	q := uint64(i)
	inside := r.QueryRegion == d.Region && q >= start && q < end
	modelValue := buildV(r.QueryRegion, q&(buildPad(r.Profile)-1))
	if inside {
		j, _ := bits.Sub64(q, start, 0)
		key, _ := bits.Add64(uint64(buildN(s.Offset)), j, 0)
		modelValue = buildV(s.Region, key&(buildPad(r.Profile)-1))
	}
	h := buildH(int(n))
	before := buildRows(h)
	dst, dc, dp := buildConstruct(h, d)
	src, ss, sp := buildConstruct(h, s)
	targetBefore, targetRead, targetP := "", false, false
	sourceBefore, sourceRead, sourceP := "", false, false
	fullBefore, fullRead, fullP := "", false, false
	if dc {
		targetBefore, targetRead, targetP = buildRead(dst, i)
		fullBefore, fullRead, fullP = buildRead(dst[:cap(dst)], i)
	}
	if ss {
		sourceBefore, sourceRead, sourceP = buildRead(src, i)
	}
	result := []buildCell(nil)
	appended := false
	actualNil := false
	actualLen, actualCap := "", ""
	values, fullValues := []string{}, []string{}
	resultAt, sourceAfter, oldAfter, fullAfter, appendedAt, aliasAt, naiveAt := "", "", "", "", "", "", ""
	oldPreserved, probe, shares := false, false, false
	resultRead, resultP, addedRead, addedP := false, false, false, false
	mapped, _ := bits.Add64(dl, uint64(i), 0)
	if dc && ss {
		oldLen, oldCap, oldNil := len(dst), cap(dst), dst == nil
		result = append(dst, src...)
		appended = true
		actualNil = result == nil
		actualLen = strconv.Itoa(len(result))
		actualCap = strconv.Itoa(cap(result))
		oldPreserved = oldLen == len(dst) && oldCap == cap(dst) && oldNil == (dst == nil)
		values = buildTexts(result)
		fullValues = buildTexts(result[:cap(result)])
		resultAt, resultRead, resultP = buildRead(result, i)
		appendedAt, addedRead, addedP = buildRead(result, int64(mapped))
		aliasAt, _, _ = buildRead(dst[:cap(dst)], int64(mapped))
		if norealloc {
			naive := buildH(int(n))
			nd, _, _ := buildConstruct(naive, d)
			ns, _, _ := buildConstruct(naive, s)
			nf := nd[:cap(nd)]
			for j := range ns {
				nf[len(nd)+j] = ns[j]
			}
			naiveAt, _, _ = buildRead(nf, int64(mapped))
		}
		if cap(dst) > 0 && cap(result) > 0 {
			probe = true
			a, b := dst[:cap(dst)], result[:cap(result)]
			old := a[0]
			saved := b[0]
			mark := buildCell{^old.Amount, ^old.Tag}
			b[0] = mark
			shares = a[0] == mark
			b[0] = saved
		}
	}
	if dc {
		oldAfter, _, _ = buildRead(dst, i)
		fullAfter, _, _ = buildRead(dst[:cap(dst)], i)
	}
	if ss {
		sourceAfter, _, _ = buildRead(src, i)
	}
	return map[string]any{"kind": "append", "destination_valid": buildValid(d, n, r.Live), "source_valid": buildValid(s, n, r.Live), "storage_bound": uint64(n) <= uint64(r.Profile), "total": strconv.FormatUint(total, 10), "length_carry": c != 0, "start": strconv.FormatUint(start, 10), "start_carry": sc != 0, "end": strconv.FormatUint(end, 10), "end_carry": ec != 0, "no_reallocation": norealloc, "model_valid": buildValid(newDescriptor, n, r.Live), "inside_write": inside, "model_query_after": buildT(modelValue),
		"destination_constructed": dc, "source_constructed": ss, "destination_construction_panicked": dp, "source_construction_panicked": sp, "appended": appended, "actual_nil": actualNil, "actual_length": actualLen, "actual_capacity": actualCap, "values": values, "full_values": fullValues, "old_metadata_preserved": oldPreserved, "probe_observed": probe, "shares_old": shares, "destination_read": targetRead, "destination_read_panicked": targetP, "source_read": sourceRead, "source_read_panicked": sourceP, "full_read": fullRead, "full_read_panicked": fullP, "result_read": resultRead, "result_read_panicked": resultP, "appended_read": addedRead, "appended_read_panicked": addedP,
		"destination_before": targetBefore, "destination_after": oldAfter, "full_before": fullBefore, "full_after": fullAfter, "source_before": sourceBefore, "source_after": sourceAfter, "result_at": resultAt, "appended_at": appendedAt, "alias_at": aliasAt, "naive_at": naiveAt, "mapped": strconv.FormatUint(mapped, 10), "before": before, "after": buildRows(h)}
}
func composedObservation(r buildInput) map[string]any {
	n, l, k, i := buildN(r.Storage), buildN(r.Length), buildN(r.Capacity), buildN(r.Index)
	if n != k || n < 0 || n > 257 {
		panic("composed workload uses a common bounded span")
	}
	if l >= 0 && k >= l && k > 257 {
		panic("no huge allocation workload")
	}
	h := buildH(int(n))
	before := buildRows(h)
	src, sc, sp := buildConstruct(h, r.Source)
	sourceBefore, sourceRead, sourceP := "", false, false
	if sc {
		sourceBefore, sourceRead, sourceP = buildRead(src, i)
	}
	initial, mc, mp := buildMake(l, k)
	total, carry := bits.Add64(uint64(l), uint64(buildN(r.Source.Length)), 0)
	nr := carry == 0 && total <= uint64(k)
	mapped, _ := bits.Add64(uint64(l), uint64(i), 0)
	liveAfter := r.Live
	liveAfter[buildR(r.FreshRegion)] = true
	modelDescriptor := buildView{"0", strconv.FormatInt(int64(total), 10), r.Capacity, false, r.FreshRegion}
	modelValue := buildV(r.QueryRegion, uint64(i)&(buildPad(r.Profile)-1))
	if r.QueryRegion == r.FreshRegion && uint64(i) < uint64(k) {
		modelValue = buildCell{}
	}
	if r.QueryRegion == r.FreshRegion && uint64(i) >= uint64(l) && uint64(i) < total {
		delta, _ := bits.Sub64(uint64(i), uint64(l), 0)
		key, _ := bits.Add64(uint64(buildN(r.Source.Offset)), delta, 0)
		modelValue = buildV(r.Source.Region, key&(buildPad(r.Profile)-1))
		if r.Source.Region == r.FreshRegion && key < uint64(k) {
			modelValue = buildCell{}
		}
	}
	result := []buildCell(nil)
	op := mc && sc
	actualNil := false
	actualLen, actualCap := "", ""
	values, fullValues := []string{}, []string{}
	value, sourceAfter := "", ""
	read, readP, probe, shares := false, false, false, false
	if op {
		result = append(initial, src...)
		actualNil = result == nil
		actualLen = strconv.Itoa(len(result))
		actualCap = strconv.Itoa(cap(result))
		values = buildTexts(result)
		fullValues = buildTexts(result[:cap(result)])
		value, read, readP = buildRead(result, int64(mapped))
		if cap(initial) > 0 && cap(result) > 0 {
			probe = true
			a, b := initial[:cap(initial)], result[:cap(result)]
			old, saved := a[0], b[0]
			mark := buildCell{^old.Amount, ^old.Tag}
			b[0] = mark
			shares = a[0] == mark
			b[0] = saved
		}
	}
	if sc {
		sourceAfter, _, _ = buildRead(src, i)
	}
	borrowedBefore, borrowedAfter := "", ""
	if i >= 0 && i < n {
		borrowedBefore = before[buildR(r.QueryRegion)][i]
		borrowedAfter = buildT(h[buildR(r.QueryRegion)][i])
	}
	return map[string]any{"kind": "composed", "guard": l >= 0 && k >= 0 && l <= k, "fresh": !r.Live[buildR(r.FreshRegion)], "source_valid": buildValid(r.Source, n, r.Live), "source_constructed": sc, "source_construction_panicked": sp, "storage_bound": uint64(k) <= uint64(r.Profile), "query_live": r.Live[buildR(r.QueryRegion)], "model_valid": buildValid(modelDescriptor, k, liveAfter), "model_query_after": buildT(modelValue), "made": mc, "make_panicked": mp, "appended": op, "total": strconv.FormatUint(total, 10), "length_carry": carry != 0, "no_reallocation": nr, "actual_nil": actualNil, "actual_length": actualLen, "actual_capacity": actualCap, "values": values, "full_values": fullValues, "source_read": sourceRead, "source_read_panicked": sourceP, "source_before": sourceBefore, "source_after": sourceAfter, "mapped": strconv.FormatUint(mapped, 10), "result_read": read, "result_read_panicked": readP, "result_value": value, "probe_observed": probe, "shares_preallocation": shares, "borrowed_before": borrowedBefore, "borrowed_after": borrowedAfter, "before": before, "after": buildRows(h)}
}

func main() {
	if len(os.Args) != 2 {
		panic("input path required")
	}
	raw, e := os.ReadFile(os.Args[1])
	if e != nil {
		panic(e)
	}
	var inputs []buildInput
	if e = json.Unmarshal(raw, &inputs); e != nil {
		panic(e)
	}
	out := make([]map[string]any, 0, len(inputs))
	for _, r := range inputs {
		switch r.Profile {
		case 0, 1, 8, 32, 127, 128, 129:
		default:
			panic("private bound required")
		}
		switch r.Kind {
		case "make":
			out = append(out, makeObservation(r))
		case "append":
			out = append(out, appendObservation(r))
		case "composed":
			out = append(out, composedObservation(r))
		default:
			panic("make/append required")
		}
	}
	if e = json.NewEncoder(os.Stdout).Encode(out); e != nil {
		panic(e)
	}
}
