package com.voidstack.caller.services

import android.app.Notification
import android.app.NotificationChannel
import android.app.NotificationManager
import android.app.Service
import android.content.Context
import android.content.Intent
import android.os.Build
import android.os.Handler
import android.os.IBinder
import android.os.Looper
import android.util.Log
import androidx.core.app.NotificationCompat
import com.voidstack.caller.network.WebSocketClient
import com.voidstack.caller.telephony.CallManager
import com.voidstack.caller.telephony.ContactHelper
import com.voidstack.caller.telephony.WhitelistManager
import com.voidstack.caller.audio.AudioRouter

class ConnectionService : Service(), WebSocketClient.WsListener {
    
    private lateinit var wsClient: WebSocketClient
    private lateinit var callManager: CallManager
    private lateinit var audioRouter: AudioRouter
    private lateinit var whitelistManager: WhitelistManager
    private val mainHandler = Handler(Looper.getMainLooper())
    private val TAG = "ConnectionService"
    
    private var pendingCallId: String? = null
    private var activeCallId: String? = null
    private var autoAnswerRunnable: Runnable? = null

    companion object {
        private const val NOTIFICATION_ID = 1
        private const val CHANNEL_ID = "voidcaller_service"
        var instance: ConnectionService? = null
    }

    override fun onCreate() {
        super.onCreate()
        instance = this
        callManager = CallManager(this)
        audioRouter = AudioRouter(this)
        whitelistManager = WhitelistManager(this)
        
        createNotificationChannel()
        startForeground(NOTIFICATION_ID, createNotification("Connecting to VoidRelay..."))

        // Read PC IP and port from SharedPreferences
        val prefs = getSharedPreferences("VoidPrefs", Context.MODE_PRIVATE)
        val pcIp = prefs.getString("server_ip", "192.168.1.100") ?: "192.168.1.100"
        val pcPort = prefs.getString("server_port", "7331") ?: "7331"
        val deviceName = Build.MODEL
        
        Log.i(TAG, "Connecting to VoidRelay at $pcIp:$pcPort")
        wsClient = WebSocketClient(pcIp, pcPort, deviceName, this)
        wsClient.connect()
    }

    override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
        return START_STICKY
    }

    override fun onBind(intent: Intent?): IBinder? = null

    override fun onDestroy() {
        super.onDestroy()
        wsClient.disconnect()
        instance = null
    }

    // --- WebSocket Callbacks (called on OkHttp background thread) ---

    override fun onConnected() {
        Log.i(TAG, "WebSocket connected to VoidRelay")
        updateNotification("Connected to VoidRelay")
        wsClient.sendEvent("bt_connected", mapOf("device_name" to Build.MODEL))
    }

    override fun onDisconnected() {
        Log.w(TAG, "WebSocket disconnected from VoidRelay")
        updateNotification("Disconnected. Reconnecting...")
        // Reconnect after 5 seconds
        mainHandler.postDelayed({
            Log.i(TAG, "Attempting reconnection...")
            wsClient.connect()
        }, 5000)
    }

    override fun onCommandCall(number: String, callId: String) {
        Log.i(TAG, ">>> RECEIVED command_call: number=$number, callId=$callId")
        pendingCallId = callId
        
        // CRITICAL: Dispatch to main thread - Xiaomi/MIUI blocks placeCall from background threads
        mainHandler.post {
            Log.i(TAG, ">>> Executing call on MAIN thread for number: $number")
            try {
                callManager.initiateCall(number)
                Log.i(TAG, ">>> initiateCall completed successfully")
            } catch (e: Exception) {
                Log.e(TAG, ">>> initiateCall FAILED", e)
            }
        }
    }

    override fun onCommandHangup(callId: String) {
        Log.i(TAG, ">>> RECEIVED command_hangup: callId=$callId")
        mainHandler.post {
            callManager.endCall()
        }
    }
    
    fun notifyIncomingCall(number: String) {
        val prefs = getSharedPreferences("VoidPrefs", Context.MODE_PRIVATE)
        val treatContactsAsKnown = prefs.getBoolean("treat_contacts_as_known", true)
        val screenUnknown = prefs.getBoolean("screen_unknown_callers", false)
        val answerDelaySec = prefs.getInt("auto_answer_delay_sec", 3)
        val routeMode = prefs.getString("audio_route_mode", "bluetooth") ?: "bluetooth"

        val contactName = if (treatContactsAsKnown) {
            ContactHelper.lookupContactName(this, number)
        } else null

        val isWhitelisted = whitelistManager.isWhitelisted(number)
        val isKnown = (contactName != null) || isWhitelisted

        Log.i(TAG, "Incoming call: $number, contactName=$contactName, isWhitelisted=$isWhitelisted -> isKnown=$isKnown")

        val payload = mutableMapOf<String, Any>(
            "number" to number,
            "known" to isKnown
        )
        if (!contactName.isNullOrBlank()) {
            payload["contact_name"] = contactName
        }

        wsClient.sendEvent("incoming_call", payload)

        // Cancel any pending auto-answer runnable
        autoAnswerRunnable?.let { mainHandler.removeCallbacks(it) }
        autoAnswerRunnable = null

        if (isKnown) {
            Log.i(TAG, "Caller is known ($contactName). Letting phone ring normally.")
            return
        }

        // Unknown caller
        if (!screenUnknown) {
            Log.i(TAG, "Screen unknown callers is DISABLED. Letting phone ring normally.")
            return
        }

        if (!wsClient.isWsConnected()) {
            Log.w(TAG, "Relay PC is DISCONNECTED from WebSocket. Bypassing auto-answer to avoid dropped calls.")
            return
        }

        Log.i(TAG, "Scheduling auto-answer for unknown caller in ${answerDelaySec}s...")
        val runnable = Runnable {
            Log.i(TAG, "Executing auto-answer for unknown caller $number")
            try {
                callManager.acceptCall()
                audioRouter.applyCallRouting(routeMode)
            } catch (e: Exception) {
                Log.e(TAG, "Failed to auto-answer ringing call", e)
            }
        }
        autoAnswerRunnable = runnable
        mainHandler.postDelayed(runnable, (answerDelaySec * 1000L).coerceAtLeast(0L))
    }

    fun notifyCallStarted(number: String) {
        Log.i(TAG, "Call started: $number")
        autoAnswerRunnable?.let { 
            mainHandler.removeCallbacks(it)
            autoAnswerRunnable = null
        }
        val prefs = getSharedPreferences("VoidPrefs", Context.MODE_PRIVATE)
        val routeMode = prefs.getString("audio_route_mode", "bluetooth") ?: "bluetooth"
        audioRouter.applyCallRouting(routeMode)

        val callId = pendingCallId ?: java.util.UUID.randomUUID().toString()
        activeCallId = callId
        pendingCallId = null
        
        wsClient.sendEvent("call_started", mapOf(
            "number" to number,
            "call_id" to callId
        ))
    }

    fun notifyCallEnded() {
        Log.i(TAG, "Call ended")
        autoAnswerRunnable?.let {
            mainHandler.removeCallbacks(it)
            autoAnswerRunnable = null
        }
        audioRouter.resetRouting()
        val callId = activeCallId ?: ""
        activeCallId = null
        pendingCallId = null
        
        wsClient.sendEvent("call_ended", mapOf(
            "call_id" to callId,
            "duration_sec" to 0
        ))
    }

    // --- Notifications ---

    private fun createNotificationChannel() {
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
            val channel = NotificationChannel(
                CHANNEL_ID,
                "VoidCaller Background Service",
                NotificationManager.IMPORTANCE_LOW
            )
            val manager = getSystemService(Context.NOTIFICATION_SERVICE) as NotificationManager
            manager.createNotificationChannel(channel)
        }
    }

    private fun createNotification(text: String): Notification {
        return NotificationCompat.Builder(this, CHANNEL_ID)
            .setContentTitle("VoidCaller Active")
            .setContentText(text)
            .setSmallIcon(android.R.drawable.ic_dialog_info)
            .setOngoing(true)
            .build()
    }

    private fun updateNotification(text: String) {
        val manager = getSystemService(Context.NOTIFICATION_SERVICE) as NotificationManager
        manager.notify(NOTIFICATION_ID, createNotification(text))
    }
}
