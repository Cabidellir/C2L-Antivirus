package com.c2l.antivirus

/**
 * Demonstration JNI interface. Each call operates on a bounded byte chunk.
 * The native library is built from android/native using the Android NDK.
 */
object RustEngine {
    init {
        System.loadLibrary("c2l_android")
    }

    external fun startHash(): Long
    external fun updateHash(id: Long, bytes: ByteArray, count: Int): Boolean
    external fun finishHash(id: Long): String?
    external fun cancelHash(id: Long)
    external fun classifyHash(hash: String, indicators: String): String?
}
