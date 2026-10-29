package com.voidstack.caller.telephony

import android.annotation.SuppressLint
import android.content.ComponentName
import android.content.Context
import android.content.Intent
import android.net.Uri
import android.os.Bundle
import android.telecom.TelecomManager
import android.util.Log
import com.voidstack.caller.services.VoidConnectionService

class CallManager(private val context: Context) {
    private val telecomManager = context.getSystemService(Context.TELECOM_SERVICE) as TelecomManager
    private val TAG = "CallManager"

    @SuppressLint("MissingPermission")
    fun initiateCall(number: String) {
        Log.i(TAG, "=== initiateCall START === number=$number")
        
        // Method 1: Direct Intent.ACTION_CALL (most reliable on Xiaomi)
        try {
            Log.i(TAG, "Trying Method 1: Intent.ACTION_CALL")
            val uri = Uri.parse("tel:$number")
            val intent = Intent(Intent.ACTION_CALL, uri).apply {
                flags = Intent.FLAG_ACTIVITY_NEW_TASK or Intent.FLAG_ACTIVITY_CLEAR_TOP
            }
            context.startActivity(intent)
            Log.i(TAG, "Method 1 SUCCESS: Intent.ACTION_CALL launched")
            return
        } catch (e: Exception) {
            Log.e(TAG, "Method 1 FAILED: Intent.ACTION_CALL", e)
        }

        // Method 2: TelecomManager.placeCall with SIM account
        try {
            Log.i(TAG, "Trying Method 2: TelecomManager.placeCall")
            val uri = Uri.fromParts("tel", number, null)
            val extras = Bundle()
            
            val accounts = telecomManager.callCapablePhoneAccounts
            Log.i(TAG, "Call-capable phone accounts: ${accounts?.size ?: 0}")
            if (accounts != null && accounts.isNotEmpty()) {
                val ourService = ComponentName(context, VoidConnectionService::class.java)
                val simAccount = accounts.find { it.componentName != ourService }
                if (simAccount != null) {
                    Log.i(TAG, "Using SIM account: ${simAccount.id}")
                    extras.putParcelable(TelecomManager.EXTRA_PHONE_ACCOUNT_HANDLE, simAccount)
                }
            }

            telecomManager.placeCall(uri, extras)
            Log.i(TAG, "Method 2 SUCCESS: TelecomManager.placeCall executed")
            return
        } catch (e: Exception) {
            Log.e(TAG, "Method 2 FAILED: TelecomManager.placeCall", e)
        }
        
        // Method 3: ACTION_DIAL (opens dialer UI for user to confirm — last resort)
        try {
            Log.i(TAG, "Trying Method 3: Intent.ACTION_DIAL (manual fallback)")
            val uri = Uri.parse("tel:$number")
            val intent = Intent(Intent.ACTION_DIAL, uri).apply {
                flags = Intent.FLAG_ACTIVITY_NEW_TASK or Intent.FLAG_ACTIVITY_CLEAR_TOP
            }
            context.startActivity(intent)
            Log.i(TAG, "Method 3 SUCCESS: ACTION_DIAL opened dialer")
            return
        } catch (e: Exception) {
            Log.e(TAG, "Method 3 FAILED: Intent.ACTION_DIAL", e)
        }
        
        Log.e(TAG, "=== ALL METHODS FAILED === Could not initiate call to $number")
    }

    @SuppressLint("MissingPermission")
    fun endCall() {
        try {
            if (android.os.Build.VERSION.SDK_INT >= android.os.Build.VERSION_CODES.P) {
                telecomManager.endCall()
                Log.i(TAG, "endCall() executed")
            } else {
                Log.w(TAG, "End call not supported below Android P via direct API")
            }
        } catch (e: Exception) {
            Log.e(TAG, "Failed to end call", e)
        }
    }

    @SuppressLint("MissingPermission")
    fun acceptCall() {
        try {
            if (android.os.Build.VERSION.SDK_INT >= android.os.Build.VERSION_CODES.O) {
                telecomManager.acceptRingingCall()
                Log.i(TAG, "acceptRingingCall() executed")
            }
        } catch (e: Exception) {
            Log.e(TAG, "Failed to accept call", e)
        }
    }
}
