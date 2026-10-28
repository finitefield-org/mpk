// A standalone Linux/amd64 diagnostic for the existing installed-root syscall.
package main

import (
	"encoding/json"
	"fmt"
	"os"
	"syscall"
	"unsafe"
)

type openHow struct{ Flags, Mode, Resolve uint64 }

func main() {
	rows := []map[string]any{}
	for _, path := range []string{"/installed", "/tmp"} {
		p, err := syscall.BytePtrFromString(path)
		if err != nil {
			panic(err)
		}
		how := openHow{uint64(syscall.O_RDONLY | syscall.O_DIRECTORY | syscall.O_CLOEXEC | syscall.O_NOFOLLOW), 0, 0x02 | 0x04}
		fd, _, errno := syscall.RawSyscall6(437, ^uintptr(99), uintptr(unsafe.Pointer(p)), uintptr(unsafe.Pointer(&how)), unsafe.Sizeof(how), 0, 0)
		if errno == 0 {
			_ = syscall.Close(int(fd))
		}
		rows = append(rows, map[string]any{"path": path, "accepted": errno == 0, "errno": uint64(errno)})
	}
	executable, _ := os.Readlink("/proc/self/exe")
	result, err := json.Marshal(map[string]any{"syscall": "openat2", "observations": rows, "proc_self_exe": executable})
	if err != nil {
		panic(err)
	}
	fmt.Println(string(result))
}
