package dev.dioxus.main

import android.webkit.WebView

class MainActivity : WryActivity() {
    override val handleBackNavigation: Boolean = false

    private var webView: WebView? = null

    override fun onWebViewCreate(webView: WebView) {
        this.webView = webView
    }

    external fun signalBackPressed()

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
