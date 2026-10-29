package com.voidstack.caller.services

import android.telecom.Connection
import android.telecom.ConnectionRequest
import android.telecom.ConnectionService
import android.telecom.PhoneAccountHandle
import android.telecom.TelecomManager
import android.util.Log

class VoidConnectionService : ConnectionService() {
    private val TAG = "VoidConnectionService"

    override fun onCreateOutgoingConnection(
        connectionManagerPhoneAccount: PhoneAccountHandle?,
        request: ConnectionRequest?
    ): Connection {
        Log.i(TAG, "onCreateOutgoingConnection")
        val connection = VoidConnection()
        connection.setDialing()
        return connection
    }

    override fun onCreateOutgoingConnectionFailed(
        connectionManagerPhoneAccount: PhoneAccountHandle?,
        request: ConnectionRequest?
    ) {
        Log.e(TAG, "onCreateOutgoingConnectionFailed")
    }

    override fun onCreateIncomingConnection(
        connectionManagerPhoneAccount: PhoneAccountHandle?,
        request: ConnectionRequest?
    ): Connection {
        Log.i(TAG, "onCreateIncomingConnection")
        val connection = VoidConnection()
        connection.setRinging()
        return connection
    }

    override fun onCreateIncomingConnectionFailed(
        connectionManagerPhoneAccount: PhoneAccountHandle?,
        request: ConnectionRequest?
    ) {
        Log.e(TAG, "onCreateIncomingConnectionFailed")
    }
}

class VoidConnection : Connection() {
    private val TAG = "VoidConnection"

    init {
        connectionProperties = PROPERTY_SELF_MANAGED
        audioModeIsVoip = false
    }

    override fun onAnswer() {
        Log.i(TAG, "onAnswer")
        setActive()
    }

    override fun onReject() {
        Log.i(TAG, "onReject")
        setDisconnected(android.telecom.DisconnectCause(android.telecom.DisconnectCause.REJECTED))
        destroy()
    }

    override fun onDisconnect() {
        Log.i(TAG, "onDisconnect")
        setDisconnected(android.telecom.DisconnectCause(android.telecom.DisconnectCause.LOCAL))
        destroy()
    }

    override fun onAbort() {
        Log.i(TAG, "onAbort")
        setDisconnected(android.telecom.DisconnectCause(android.telecom.DisconnectCause.CANCELED))
        destroy()
    }
}
