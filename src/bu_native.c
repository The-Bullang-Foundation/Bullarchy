// bu_native.c — JNI glue for Bullang's Java backend arbitrary-fd support.
//
// Implements the four native methods declared in BuNative.java, using the
// exact same POSIX I/O logic as this project's own C backend (see
// src/stdlib/{open,close,fd_in,fd_out}.rs) — including the byte-by-byte read
// in nIn, for the same reason fd_in.rs reads byte-by-byte instead of via a
// buffered reader: a single builtin::in() call must consume only the bytes
// of the line it returns, so a fresh call reading the same fd again picks up
// right where the last one left off.
//
// Build: see Makefile.native in the same directory (invokes this file
// directly — nothing here is project-specific, so it never changes between
// builds of the same project).

#include <jni.h>
#include <fcntl.h>
#include <unistd.h>
#include <string.h>
#include <sys/stat.h>

JNIEXPORT jint JNICALL Java_BuNative_nOpen(JNIEnv *env, jclass cls, jstring path, jstring mode) {
    (void)cls;
    const char *__path = (*env)->GetStringUTFChars(env, path, NULL);
    const char *__mode = (*env)->GetStringUTFChars(env, mode, NULL);

    int __flags;
    if      (strcmp(__mode, "r")  == 0) __flags = O_RDONLY;
    else if (strcmp(__mode, "w")  == 0) __flags = O_WRONLY | O_CREAT | O_TRUNC;
    else if (strcmp(__mode, "a")  == 0) __flags = O_WRONLY | O_CREAT | O_APPEND;
    else if (strcmp(__mode, "rw") == 0) __flags = O_RDWR  | O_CREAT;
    else                                __flags = O_RDONLY;

    int __fd = open(__path, __flags, 0644);

    (*env)->ReleaseStringUTFChars(env, path, __path);
    (*env)->ReleaseStringUTFChars(env, mode, __mode);
    return (jint)__fd;
}

JNIEXPORT jint JNICALL Java_BuNative_nClose(JNIEnv *env, jclass cls, jint fd) {
    (void)env;
    (void)cls;
    return close((int)fd) == 0 ? 0 : -1;
}

JNIEXPORT jstring JNICALL Java_BuNative_nIn(JNIEnv *env, jclass cls, jint fd) {
    (void)cls;
    char __buf[4096];
    size_t __i = 0;
    char __ch;
    ssize_t __n;

    while (__i < sizeof(__buf) - 1 && (__n = read((int)fd, &__ch, 1)) > 0) {
        if (__ch == '\n') break;
        __buf[__i++] = __ch;
    }
    if (__i > 0 && __buf[__i - 1] == '\r') __i--;
    __buf[__i] = '\0';

    return (*env)->NewStringUTF(env, __buf);
}

JNIEXPORT jint JNICALL Java_BuNative_nOut(JNIEnv *env, jclass cls, jint fd, jstring content) {
    (void)cls;
    const char *__c = (*env)->GetStringUTFChars(env, content, NULL);
    ssize_t __n = write((int)fd, __c, strlen(__c));
    (*env)->ReleaseStringUTFChars(env, content, __c);
    return (jint)__n;
}
