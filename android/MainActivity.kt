package dev.dioxus.main

import android.os.Bundle
import android.webkit.WebView

class MainActivity : WryActivity() {
    override val handleBackNavigation: Boolean = false

    private var webView: WebView? = null

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        initDbPath(getFilesDir().absolutePath)
    }

    override fun onWebViewCreate(webView: WebView) {
        this.webView = webView
    }

    external fun signalBackPressed()
    external fun initDbPath(path: String)

    override fun onBackPressed() {
        webView?.let {
            if (it.canGoBack()) {
                it.goBack()
                return
            }
        }
        signalBackPressed()
    }
}
