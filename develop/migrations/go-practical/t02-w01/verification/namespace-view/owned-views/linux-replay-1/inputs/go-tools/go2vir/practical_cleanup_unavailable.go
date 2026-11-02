//go:build !go1.25

package main

import "fmt"

func removePracticalReplayImage(string) error {
	return fmt.Errorf("original replay image cleanup needs the pinned Go1.25.0 root API")
}
