package com.voidstack.caller.receivers

import android.content.BroadcastReceiver
import android.content.Context
import android.content.Intent
import android.telephony.TelephonyManager
import android.util.Log
import com.voidstack.caller.services.ConnectionService

class CallStateReceiver : BroadcastReceiver() {
    private val TAG = "CallStateReceiver"

    companion object {
        private var activeNumber: String? = null
    }

    override fun onReceive(context: Context, intent: Intent) {
        if (intent.action == TelephonyManager.ACTION_PHONE_STATE_CHANGED) {
            val state = intent.getStringExtra(TelephonyManager.EXTRA_STATE)
            val number = intent.getStringExtra(TelephonyManager.EXTRA_INCOMING_NUMBER)
            
            Log.d(TAG, "Phone state changed: State=$state, Number=$number")

            if (!number.isNullOrEmpty()) {
                activeNumber = number
            }

            when (state) {
                TelephonyManager.EXTRA_STATE_RINGING -> {
                    val incomingNumber = number ?: activeNumber ?: "Unknown"
                    ConnectionService.instance?.notifyIncomingCall(incomingNumber)
                }
                TelephonyManager.EXTRA_STATE_OFFHOOK -> {
                    // Call was answered or placed
                    val num = activeNumber ?: "Unknown"
                    ConnectionService.instance?.notifyCallStarted(num)
                }
                TelephonyManager.EXTRA_STATE_IDLE -> {
                    // Call ended
                    ConnectionService.instance?.notifyCallEnded()
                    activeNumber = null
                }
            }
        }
    }
}
