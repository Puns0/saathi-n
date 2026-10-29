package com.voidstack.caller.network

import android.util.Log
import okhttp3.*
import org.json.JSONObject
import java.util.concurrent.TimeUnit

class WebSocketClient(
    private val serverIp: String,
    private val serverPort: String,
    private val deviceName: String,
    private val listener: WsListener
) {
    private var client: OkHttpClient? = null
    private var webSocket: WebSocket? = null
    private var isConnected = false
    private val TAG = "VoidWsClient"

    interface WsListener {
        fun onConnected()
        fun onDisconnected()
        fun onCommandCall(number: String, callId: String)
        fun onCommandHangup(callId: String)
    }

    fun connect() {
        if (isConnected) return

        client = OkHttpClient.Builder()
            .readTimeout(0, TimeUnit.MILLISECONDS)
            .pingInterval(10, TimeUnit.SECONDS)
            .build()

        val wsUrl = "ws://$serverIp:$serverPort"
        Log.i(TAG, "Connecting to $wsUrl")

        val request = Request.Builder()
            .url(wsUrl)
            .build()

        webSocket = client?.newWebSocket(request, object : WebSocketListener() {
            override fun onOpen(webSocket: WebSocket, response: Response) {
                Log.i(TAG, "Connected to VoidRelay at $wsUrl")
                isConnected = true
                
                // Send Hello
                val hello = JSONObject().apply {
                    put("type", "hello")
                    put("client", "android")
                    put("version", "1.0.0")
                    put("device_name", deviceName)
                }
                webSocket.send(hello.toString())
                Log.d(TAG, "Sent hello: ${hello.toString()}")
                
                listener.onConnected()
            }

            override fun onMessage(webSocket: WebSocket, text: String) {
                Log.d(TAG, "<<< Received message: $text")
                try {
                    val json = JSONObject(text)
                    val type = json.getString("type")
                    Log.i(TAG, "Message type: $type")
                    
                    when (type) {
                        "command_call" -> {
                            val number = json.getString("number")
                            val callId = json.getString("call_id")
                            Log.i(TAG, ">>> COMMAND_CALL received: number=$number callId=$callId")
                            listener.onCommandCall(number, callId)
                        }
                        "command_hangup" -> {
                            val callId = json.getString("call_id")
                            Log.i(TAG, ">>> COMMAND_HANGUP received: callId=$callId")
                            listener.onCommandHangup(callId)
                        }
                        "heartbeat" -> {
                            val ack = JSONObject().apply { put("type", "heartbeat_ack") }
                            webSocket.send(ack.toString())
                        }
                        else -> {
                            Log.w(TAG, "Unknown message type: $type")
                        }
                    }
                } catch (e: Exception) {
                    Log.e(TAG, "Error parsing WS message: $text", e)
                }
            }

            override fun onClosed(webSocket: WebSocket, code: Int, reason: String) {
                Log.i(TAG, "Disconnected from VoidRelay: $reason")
                isConnected = false
                listener.onDisconnected()
            }

            override fun onFailure(webSocket: WebSocket, t: Throwable, response: Response?) {
                Log.e(TAG, "WebSocket failure: ${t.message}", t)
                isConnected = false
                listener.onDisconnected()
            }
        })
    }

    fun disconnect() {
        webSocket?.close(1000, "User requested")
        client?.dispatcher?.executorService?.shutdown()
        isConnected = false
    }

    fun sendEvent(type: String, payload: Map<String, Any>) {
        if (!isConnected) {
            Log.w(TAG, "Cannot send event '$type' - not connected")
            return
        }
        try {
            val json = JSONObject().apply {
                put("type", type)
                put("client", "android")
                payload.forEach { (k, v) -> put(k, v) }
            }
            val msg = json.toString()
            Log.d(TAG, ">>> Sending event: $msg")
            webSocket?.send(msg)
        } catch (e: Exception) {
            Log.e(TAG, "Failed to send event", e)
        }
    }
}
