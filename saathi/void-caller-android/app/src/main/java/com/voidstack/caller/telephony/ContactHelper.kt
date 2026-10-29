package com.voidstack.caller.telephony

import android.content.Context
import android.net.Uri
import android.provider.ContactsContract
import android.util.Log

object ContactHelper {
    private const val TAG = "ContactHelper"

    /**
     * Normalizes phone number into standard digits without formatting,
     * stripping spaces, dashes, parentheses, and leading +91 or 0 prefixes.
     */
    fun normalizeNumber(raw: String): String {
        var clean = raw.replace(Regex("[^0-9+]"), "")
        if (clean.startsWith("+91") && clean.length > 3) {
            clean = clean.substring(3)
        } else if (clean.startsWith("91") && clean.length == 12) {
            clean = clean.substring(2)
        } else if (clean.startsWith("0") && clean.length > 1) {
            clean = clean.substring(1)
        }
        return clean
    }

    /**
     * Looks up contact name from device contacts via ContactsContract.PhoneLookup
     */
    fun lookupContactName(context: Context, number: String): String? {
        if (number.isBlank() || number.equals("Unknown", ignoreCase = true) || number.equals("Private", ignoreCase = true)) {
            return null
        }
        return try {
            val uri = Uri.withAppendedPath(
                ContactsContract.PhoneLookup.CONTENT_FILTER_URI,
                Uri.encode(number)
            )
            val projection = arrayOf(ContactsContract.PhoneLookup.DISPLAY_NAME)
            context.contentResolver.query(uri, projection, null, null, null)?.use { cursor ->
                if (cursor.moveToFirst()) {
                    val idx = cursor.getColumnIndex(ContactsContract.PhoneLookup.DISPLAY_NAME)
                    if (idx >= 0) cursor.getString(idx) else null
                } else null
            }
        } catch (e: Exception) {
            Log.e(TAG, "Error querying ContactsContract for $number", e)
            null
        }
    }
}
