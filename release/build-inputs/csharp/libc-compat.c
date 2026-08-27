#include <sys/stat.h>

/*
 * glibc did not export the fstat/fstatat names until 2.33. The frozen C#
 * frontend uses those stable P/Invoke entry points while the selected release
 * runtime intentionally keeps a 2.27 ABI floor. Bridge only those names to
 * the versioned interfaces exported by the selected libc.
 */
extern int __fxstat(int version, int descriptor, struct stat *status);
extern int __fxstatat(
    int version,
    int directory,
    const char *path,
    struct stat *status,
    int flags);

int fstat(int descriptor, struct stat *status)
{
    return __fxstat(1, descriptor, status);
}

int fstatat(int directory, const char *path, struct stat *status, int flags)
{
    return __fxstatat(1, directory, path, status, flags);
}
