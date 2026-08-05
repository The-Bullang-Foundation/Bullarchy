// BuNative.java — declares the native methods implemented in bu_native.c.
// Generated only when a project actually calls builtin::open, or calls
// builtin::close/in/out with a fd that isn't provably 0/1/2 — see
// Makefile.native for how to build the companion library this loads.
public final class BuNative {
    private BuNative() {}

    static {
        System.loadLibrary("bullang_native");
    }

    static native int nOpen(String path, String mode);
    static native int nClose(int fd);
    static native String nIn(int fd);
    static native int nOut(int fd, String content);
}
