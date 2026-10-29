package com.voidstack.caller.ui.screens

import android.content.Context
import android.content.Intent
import android.net.Uri
import android.provider.Settings
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.ArrowBack
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.unit.dp
import com.voidstack.caller.ui.theme.VoidDarkBackground
import com.voidstack.caller.ui.theme.VoidSurface
import com.voidstack.caller.ui.theme.VoidTextMuted

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun SettingsScreen(onNavigateBack: () -> Unit) {
    val context = LocalContext.current
    val prefs = context.getSharedPreferences("VoidPrefs", Context.MODE_PRIVATE)
    
    var serverIp by remember { mutableStateOf(prefs.getString("server_ip", "") ?: "") }
    var serverPort by remember { mutableStateOf(prefs.getString("server_port", "7331") ?: "7331") }

    Scaffold(
        topBar = {
            TopAppBar(
                title = { Text("Settings") },
                navigationIcon = {
                    IconButton(onClick = onNavigateBack) {
                        Icon(Icons.Filled.ArrowBack, contentDescription = "Back")
                    }
                },
                colors = TopAppBarDefaults.topAppBarColors(
                    containerColor = VoidDarkBackground
                )
            )
        },
        containerColor = VoidDarkBackground
    ) { paddingValues ->
        Column(
            modifier = Modifier
                .fillMaxSize()
                .padding(paddingValues)
                .padding(16.dp)
        ) {
            Text("Connection Settings", color = MaterialTheme.colorScheme.primary)
            Spacer(modifier = Modifier.height(16.dp))
            
            OutlinedTextField(
                value = serverIp,
                onValueChange = { 
                    serverIp = it
                    prefs.edit().putString("server_ip", it).apply()
                },
                label = { Text("Relay PC IP Address", color = VoidTextMuted) },
                modifier = Modifier.fillMaxWidth(),
                colors = OutlinedTextFieldDefaults.colors(
                    unfocusedContainerColor = VoidSurface,
                    focusedContainerColor = VoidSurface,
                ),
                shape = RoundedCornerShape(12.dp)
            )
            
            Spacer(modifier = Modifier.height(16.dp))
            
            OutlinedTextField(
                value = serverPort,
                onValueChange = { 
                    serverPort = it
                    prefs.edit().putString("server_port", it).apply()
                },
                label = { Text("Port (Default: 7331)", color = VoidTextMuted) },
                modifier = Modifier.fillMaxWidth(),
                colors = OutlinedTextFieldDefaults.colors(
                    unfocusedContainerColor = VoidSurface,
                    focusedContainerColor = VoidSurface,
                ),
                shape = RoundedCornerShape(12.dp)
            )

            Spacer(modifier = Modifier.height(32.dp))
            Text("Permissions", color = MaterialTheme.colorScheme.primary)
            Spacer(modifier = Modifier.height(8.dp))
            Text(
                "Make sure to grant the app background execution, phone state, and call permissions.\n\n" +
                "⚠️ For Xiaomi/HyperOS devices, you MUST enable 'Display pop-up windows while running in the background' in system settings for background dialing to work.",
                color = VoidTextMuted
            )
            Spacer(modifier = Modifier.height(16.dp))
            Button(
                onClick = {
                    val intent = Intent(Settings.ACTION_APPLICATION_DETAILS_SETTINGS).apply {
                        data = Uri.fromParts("package", context.packageName, null)
                        flags = Intent.FLAG_ACTIVITY_NEW_TASK
                    }
                    context.startActivity(intent)
                },
                modifier = Modifier.fillMaxWidth(),
                shape = RoundedCornerShape(12.dp)
            ) {
                Text("Open App System Settings")
            }
        }
    }
}
