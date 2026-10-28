"""Read-only Linux syscall diagnostic for the installed-root test environment."""
import ctypes
import json
import os


class OpenHow(ctypes.Structure):
    _fields_ = [('flags', ctypes.c_uint64), ('mode', ctypes.c_uint64), ('resolve', ctypes.c_uint64)]


def main():
    libc=ctypes.CDLL(None,use_errno=True)
    how=OpenHow(os.O_RDONLY|os.O_DIRECTORY|os.O_CLOEXEC|os.O_NOFOLLOW,0,0x02|0x04)
    rows=[]
    for path in ['/installed','/tmp']:
        ctypes.set_errno(0)
        fd=libc.syscall(437,-100,path.encode(),ctypes.byref(how),ctypes.sizeof(how))
        errno=ctypes.get_errno() if fd<0 else 0
        if fd>=0:
            os.close(fd)
        rows.append({'path':path,'accepted':fd>=0,'errno':errno,'error':os.strerror(errno) if errno else None})
    print(json.dumps({'syscall':'openat2','resolve':'NO_MAGICLINKS|NO_SYMLINKS','observations':rows}))


if __name__=='__main__':
    main()
