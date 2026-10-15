package com.c2l.antivirus

import android.net.Uri
import android.os.Bundle
import android.view.ViewGroup
import android.widget.Button
import android.widget.LinearLayout
import android.widget.ScrollView
import android.widget.TextView
import androidx.activity.ComponentActivity
import androidx.activity.result.contract.ActivityResultContracts

/**
 * Read-only file fingerprint demonstration using Android's system file picker.
 * No broad storage permission is requested, and no data is uploaded.
 */
class MainActivity : ComponentActivity() {
    private lateinit var status: TextView
    private lateinit var chooseButton: Button

    private val pickDocument = registerForActivityResult(
        ActivityResultContracts.OpenDocument()
    ) { uri: Uri? ->
        if (uri != null) {
            chooseButton.isEnabled = false
            status.text = "Calculating SHA-256…"
            Thread {
                val output = try {
                    analyze(uri)
                } catch (error: Exception) {
                    "Could not read selected document: ${error.message ?: "unknown error"}"
                }
                runOnUiThread {
                    status.text = output
                    chooseButton.isEnabled = true
                }
            }.start()
        }
    }

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)

        val padding = (20 * resources.displayMetrics.density).toInt()
        val layout = LinearLayout(this).apply {
            orientation = LinearLayout.VERTICAL
            setPadding(padding, padding, padding, padding)
        }

        val title = TextView(this).apply {
            text = "C2L Antivirus — Android prototype"
            textSize = 21f
        }
        layout.addView(title)

        val disclaimer = TextView(this).apply {
            text = "Read-only SHA-256 fingerprinting. Demonstration indicators only. " +
                "This app does NOT provide malware protection or inspect other apps."
            textSize = 14f
            setPadding(0, padding / 2, 0, padding / 2)
        }
        layout.addView(disclaimer)

        chooseButton = Button(this).apply {
            text = "Choose a file"
            setOnClickListener { pickDocument.launch(arrayOf("*/*")) }
        }
        layout.addView(chooseButton, ViewGroup.LayoutParams(
            ViewGroup.LayoutParams.MATCH_PARENT,
            ViewGroup.LayoutParams.WRAP_CONTENT
        ))

        status = TextView(this).apply {
            text = "Choose a file to calculate its SHA-256 fingerprint."
            textSize = 15f
            setTextIsSelectable(true)
        }
        layout.addView(status)

        val scrollView = ScrollView(this)
        scrollView.addView(layout)
        setContentView(scrollView)
    }

    private fun analyze(uri: Uri): String {
        val database = assets.open("demo-indicators.csv").bufferedReader().use { it.readText() }
        val stream = contentResolver.openInputStream(uri)
            ?: throw IllegalStateException("No readable stream was supplied")

        val handle = RustEngine.startHash()
        check(handle > 0) { "Could not start the native hashing engine" }

        var finished = false
        try {
            var length = 0L
            stream.use {
                val buffer = ByteArray(64 * 1024)
                while (true) {
                    val count = it.read(buffer)
                    if (count < 0) break
                    if (count == 0) continue
                    check(RustEngine.updateHash(handle, buffer, count)) {
                        "Native hashing update failed"
                    }
                    length += count
                }
            }

            val sha256 = RustEngine.finishHash(handle)
                ?: throw IllegalStateException("Native hashing failed")
            finished = true
            val verdict = RustEngine.classifyHash(sha256, database)
                ?: "INDICATOR CHECK ERROR"

            return "Document: ${uri.lastPathSegment ?: "selected file"}\n" +
                "Size: $length bytes\n" +
                "SHA-256: $sha256\n" +
                "Demo indicator: $verdict\n\n" +
                "No malware verdict is provided. An unmatched hash is not proof of safety."
        } finally {
            if (!finished) {
                RustEngine.cancelHash(handle)
            }
        }
    }
}
