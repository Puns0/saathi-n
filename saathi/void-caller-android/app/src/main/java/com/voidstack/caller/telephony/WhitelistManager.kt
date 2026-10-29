package com.voidstack.caller.telephony

import android.content.Context
import android.content.SharedPreferences

class WhitelistManager(context: Context) {
    private val prefs: SharedPreferences = context.getSharedPreferences("VoidPrefs", Context.MODE_PRIVATE)
    private val KEY_WHITELIST = "whitelisted_numbers"

    fun getWhitelist(): Set<String> {
        return prefs.getStringSet(KEY_WHITELIST, emptySet()) ?: emptySet()
    }

    fun addNumber(rawNumber: String) {
        val normalized = ContactHelper.normalizeNumber(rawNumber)
        if (normalized.isBlank()) return
        val current = getWhitelist().toMutableSet()
        current.add(normalized)
        prefs.edit().putStringSet(KEY_WHITELIST, current).apply()
    }

    fun removeNumber(rawNumber: String) {
        val normalized = ContactHelper.normalizeNumber(rawNumber)
        val current = getWhitelist().toMutableSet()
        current.remove(normalized)
        prefs.edit().putStringSet(KEY_WHITELIST, current).apply()
    }

    fun isWhitelisted(rawNumber: String): Boolean {
        val normalized = ContactHelper.normalizeNumber(rawNumber)
        if (normalized.isBlank()) return false
        val list = getWhitelist()
        return list.contains(normalized) || list.any { ContactHelper.normalizeNumber(it) == normalized }
    }
}
