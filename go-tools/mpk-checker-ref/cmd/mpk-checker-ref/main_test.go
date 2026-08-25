package main

import (
	"bytes"
	"testing"
)

func TestReadCertificateInputFromStdin(t *testing.T) {
	want := []byte{0x00, 0x01, 0xfe, 0xff}
	got, err := readCertificateInputFrom("-", bytes.NewReader(want))
	if err != nil {
		t.Fatalf("read stdin: %v", err)
	}
	if !bytes.Equal(got, want) {
		t.Fatalf("stdin bytes differ: got %x want %x", got, want)
	}
}
