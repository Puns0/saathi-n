package com.voidstack.caller.audio

import android.annotation.SuppressLint
import android.content.Context
import android.media.AudioDeviceInfo
import android.media.AudioManager
import android.os.Build
import android.util.Log

class AudioRouter(private val context: Context) {
    private val audioManager = context.getSystemService(Context.AUDIO_SERVICE) as AudioManager
    private val TAG = "AudioRouter"

    private fun isWiredHeadsetConnected(): Boolean {
        return if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.M) {
            val devices = audioManager.getDevices(AudioManager.GET_DEVICES_OUTPUTS)
            devices.any { 
                it.type == AudioDeviceInfo.TYPE_WIRED_HEADSET || 
                it.type == AudioDeviceInfo.TYPE_WIRED_HEADPHONES ||
                it.type == AudioDeviceInfo.TYPE_USB_HEADSET
            }
        } else {
            @Suppress("DEPRECATION")
            audioManager.isWiredHeadsetOn
        }
    }

    @SuppressLint("MissingPermission")
    fun enableBluetoothSco() {
        try {
            audioManager.mode = AudioManager.MODE_IN_CALL
            
            if (isWiredHeadsetConnected()) {
                Log.i(TAG, "Wired headset / Aux cable detected. Bypassing Bluetooth SCO routing.")
                // Disable SCO to let phone route call audio directly to the Aux cable
                audioManager.isBluetoothScoOn = false
                audioManager.stopBluetoothSco()
                return
            }

            Log.i(TAG, "Enabling Bluetooth SCO")
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.S) {
                val devices = audioManager.getDevices(AudioManager.GET_DEVICES_OUTPUTS)
                val bluetoothScoDevice = devices.find { it.type == AudioDeviceInfo.TYPE_BLUETOOTH_SCO }
                if (bluetoothScoDevice != null) {
                    val result = audioManager.setCommunicationDevice(bluetoothScoDevice)
                    Log.i(TAG, "Set communication device to Bluetooth SCO: $result")
                } else {
                    Log.w(TAG, "Bluetooth SCO device not found, starting legacy SCO")
                    audioManager.startBluetoothSco()
                    audioManager.isBluetoothScoOn = true
                }
            } else {
                @Suppress("DEPRECATION")
                if (audioManager.isBluetoothScoAvailableOffCall) {
                    audioManager.startBluetoothSco()
                    audioManager.isBluetoothScoOn = true
                    Log.i(TAG, "Started legacy Bluetooth SCO")
                } else {
                    Log.w(TAG, "Bluetooth SCO not available off call")
                }
            }
        } catch (e: Exception) {
            Log.e(TAG, "Failed to enable BT SCO", e)
        }
    }

    fun disableBluetoothSco() {
        try {
            Log.i(TAG, "Disabling Bluetooth SCO")
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.S) {
                audioManager.clearCommunicationDevice()
            }
            audioManager.isBluetoothScoOn = false
            audioManager.stopBluetoothSco()
            audioManager.mode = AudioManager.MODE_NORMAL
        } catch (e: Exception) {
            Log.e(TAG, "Failed to disable BT SCO", e)
        }
    }
}
