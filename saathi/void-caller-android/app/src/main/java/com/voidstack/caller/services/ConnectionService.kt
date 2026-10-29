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
import com.voidstack.caller.audio.AudioRouter

class ConnectionService : Service(), WebSocketClient.WsListener {
    
    private lateinit var wsClient: WebSocketClient
    private lateinit var callManager: CallManager
    private lateinit var audioRouter: AudioRouter
    private val mainHandler = Handler(Looper.getMainLooper())
    private val TAG = "ConnectionService"
    
    private var pendingCallId: String? = null
    private var activeCallId: String? = null

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
        Log.i(TAG, "Incoming call detected: $number")
        wsClient.sendEvent("incoming_call", mapOf("number" to number))
        // Let the phone's native Bluetooth Auto-Answer handle the answering
        // so that Android automatically routes call audio to MSI-PUNS0.
        /*
        mainHandler.post {
            callManager.acceptCall()
        }
        */
    }

    fun notifyCallStarted(number: String) {
        Log.i(TAG, "Call started: $number")
        audioRouter.enableBluetoothSco()
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
        audioRouter.disableBluetoothSco()
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
