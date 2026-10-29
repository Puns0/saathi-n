package com.voidstack.caller.ui.screens

import android.content.Context
import android.content.Intent
import android.net.Uri
import android.provider.Settings
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.ArrowBack
import androidx.compose.material.icons.filled.Delete
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.unit.dp
import com.voidstack.caller.telephony.WhitelistManager
import com.voidstack.caller.ui.theme.VoidDarkBackground
import com.voidstack.caller.ui.theme.VoidSurface
import com.voidstack.caller.ui.theme.VoidTextMuted

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun SettingsScreen(onNavigateBack: () -> Unit) {
    val context = LocalContext.current
    val prefs = remember { context.getSharedPreferences("VoidPrefs", Context.MODE_PRIVATE) }
    val whitelistManager = remember { WhitelistManager(context) }
    
    var serverIp by remember { mutableStateOf(prefs.getString("server_ip", "") ?: "") }
    var serverPort by remember { mutableStateOf(prefs.getString("server_port", "7331") ?: "7331") }
    var screenUnknown by remember { mutableStateOf(prefs.getBoolean("screen_unknown_callers", false)) }
    var treatContactsAsKnown by remember { mutableStateOf(prefs.getBoolean("treat_contacts_as_known", true)) }
    var answerDelaySec by remember { mutableIntStateOf(prefs.getInt("auto_answer_delay_sec", 3)) }
    var audioRouteMode by remember { mutableStateOf(prefs.getString("audio_route_mode", "bluetooth") ?: "bluetooth") }
    
    var whitelist by remember { mutableStateOf(whitelistManager.getWhitelist().toList().sorted()) }
    var newNumberInput by remember { mutableStateOf("") }

    val scrollState = rememberScrollState()

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
                .verticalScroll(scrollState)
                .padding(16.dp)
        ) {
            // --- Section 1: Connection ---
            Text("Connection Settings", color = MaterialTheme.colorScheme.primary, style = MaterialTheme.typography.titleMedium)
            Spacer(modifier = Modifier.height(12.dp))
            
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
            
            Spacer(modifier = Modifier.height(12.dp))
            
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

            Spacer(modifier = Modifier.height(24.dp))

            // --- Section 2: Call Screening & Auto-Answer ---
            Text("Call Screening & Auto-Answer", color = MaterialTheme.colorScheme.primary, style = MaterialTheme.typography.titleMedium)
            Spacer(modifier = Modifier.height(12.dp))

            // Switch: Screen Unknown Callers
            Surface(
                modifier = Modifier.fillMaxWidth(),
                color = VoidSurface,
                shape = RoundedCornerShape(12.dp)
            ) {
                Row(
                    modifier = Modifier
                        .fillMaxWidth()
                        .padding(14.dp),
                    verticalAlignment = Alignment.CenterVertically,
                    horizontalArrangement = Arrangement.SpaceBetween
                ) {
                    Column(modifier = Modifier.weight(1f)) {
                        Text("Screen Unknown Callers", style = MaterialTheme.typography.bodyLarge)
                        Text(
                            "Auto-answers calls from non-whitelisted numbers only when Relay PC is connected.",
                            color = VoidTextMuted,
                            style = MaterialTheme.typography.bodySmall
                        )
                    }
                    Switch(
                        checked = screenUnknown,
                        onCheckedChange = {
                            screenUnknown = it
                            prefs.edit().putBoolean("screen_unknown_callers", it).apply()
                        }
                    )
                }
            }

            Spacer(modifier = Modifier.height(10.dp))

            // Switch: Treat Saved Contacts as Known
            Surface(
                modifier = Modifier.fillMaxWidth(),
                color = VoidSurface,
                shape = RoundedCornerShape(12.dp)
            ) {
                Row(
                    modifier = Modifier
                        .fillMaxWidth()
                        .padding(14.dp),
                    verticalAlignment = Alignment.CenterVertically,
                    horizontalArrangement = Arrangement.SpaceBetween
                ) {
                    Column(modifier = Modifier.weight(1f)) {
                        Text("Treat Saved Contacts as Known", style = MaterialTheme.typography.bodyLarge)
                        Text(
                            "Calls from device contacts will ring normally and bypass AI screening.",
                            color = VoidTextMuted,
                            style = MaterialTheme.typography.bodySmall
                        )
                    }
                    Switch(
                        checked = treatContactsAsKnown,
                        onCheckedChange = {
                            treatContactsAsKnown = it
                            prefs.edit().putBoolean("treat_contacts_as_known", it).apply()
                        }
                    )
                }
            }

            Spacer(modifier = Modifier.height(14.dp))

            // Auto-Answer Delay
            Text("Auto-Answer Delay: ${answerDelaySec}s", style = MaterialTheme.typography.bodyMedium)
            Slider(
                value = answerDelaySec.toFloat(),
                onValueChange = {
                    val rounded = it.toInt()
                    answerDelaySec = rounded
                    prefs.edit().putInt("auto_answer_delay_sec", rounded).apply()
                },
                valueRange = 0f..6f,
                steps = 5,
                modifier = Modifier.fillMaxWidth()
            )
            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.SpaceBetween
            ) {
                Text("0s (Instant)", color = VoidTextMuted, style = MaterialTheme.typography.bodySmall)
                Text("3s (~1 ring)", color = VoidTextMuted, style = MaterialTheme.typography.bodySmall)
                Text("6s (~2 rings)", color = VoidTextMuted, style = MaterialTheme.typography.bodySmall)
            }

            Spacer(modifier = Modifier.height(16.dp))

            // Audio Routing Mode Selection
            Text("Audio Route Mode", style = MaterialTheme.typography.bodyMedium)
            Spacer(modifier = Modifier.height(8.dp))
            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.spacedBy(8.dp)
            ) {
                listOf(
                    "bluetooth" to "Bluetooth (Path A)",
                    "speakerphone" to "Speaker (Path C)",
                    "wired" to "Wired Aux (Path B)"
                ).forEach { (mode, label) ->
                    val isSelected = audioRouteMode == mode
                    FilterChip(
                        selected = isSelected,
                        onClick = {
                            audioRouteMode = mode
                            prefs.edit().putString("audio_route_mode", mode).apply()
                        },
                        label = { Text(label, style = MaterialTheme.typography.labelSmall) }
                    )
                }
            }

            Spacer(modifier = Modifier.height(24.dp))

            // --- Section 3: Whitelist Numbers ---
            Text("Whitelist Numbers", color = MaterialTheme.colorScheme.primary, style = MaterialTheme.typography.titleMedium)
            Text(
                "Numbers in the whitelist always ring normally and will not be auto-screened.",
                color = VoidTextMuted,
                style = MaterialTheme.typography.bodySmall
            )
            Spacer(modifier = Modifier.height(10.dp))

            Row(
                modifier = Modifier.fillMaxWidth(),
                verticalAlignment = Alignment.CenterVertically,
                horizontalArrangement = Arrangement.spacedBy(8.dp)
            ) {
                OutlinedTextField(
                    value = newNumberInput,
                    onValueChange = { newNumberInput = it },
                    label = { Text("Add Phone Number", color = VoidTextMuted) },
                    modifier = Modifier.weight(1f),
                    colors = OutlinedTextFieldDefaults.colors(
                        unfocusedContainerColor = VoidSurface,
                        focusedContainerColor = VoidSurface,
                    ),
                    shape = RoundedCornerShape(12.dp),
                    singleLine = true
                )
                Button(
                    onClick = {
                        val trimmed = newNumberInput.trim()
                        if (trimmed.isNotEmpty()) {
                            whitelistManager.addNumber(trimmed)
                            whitelist = whitelistManager.getWhitelist().toList().sorted()
                            newNumberInput = ""
                        }
                    },
                    shape = RoundedCornerShape(12.dp),
                    modifier = Modifier.height(56.dp)
                ) {
                    Text("Add")
                }
            }

            Spacer(modifier = Modifier.height(10.dp))

            if (whitelist.isEmpty()) {
                Text("No whitelisted numbers added yet.", color = VoidTextMuted, style = MaterialTheme.typography.bodySmall)
            } else {
                Column(
                    modifier = Modifier.fillMaxWidth(),
                    verticalArrangement = Arrangement.spacedBy(6.dp)
                ) {
                    whitelist.forEach { num ->
                        Surface(
                            modifier = Modifier.fillMaxWidth(),
                            color = VoidSurface,
                            shape = RoundedCornerShape(8.dp)
                        ) {
                            Row(
                                modifier = Modifier
                                    .fillMaxWidth()
                                    .padding(horizontal = 12.dp, vertical = 6.dp),
                                verticalAlignment = Alignment.CenterVertically,
                                horizontalArrangement = Arrangement.SpaceBetween
                            ) {
                                Text(num, style = MaterialTheme.typography.bodyMedium)
                                IconButton(onClick = {
                                    whitelistManager.removeNumber(num)
                                    whitelist = whitelistManager.getWhitelist().toList().sorted()
                                }) {
                                    Icon(
                                        Icons.Filled.Delete,
                                        contentDescription = "Remove",
                                        tint = MaterialTheme.colorScheme.error
                                    )
                                }
                            }
                        }
                    }
                }
            }

            Spacer(modifier = Modifier.height(32.dp))

            // --- Section 4: Permissions & System Settings ---
            Text("Permissions & Troubleshooting", color = MaterialTheme.colorScheme.primary, style = MaterialTheme.typography.titleMedium)
            Spacer(modifier = Modifier.height(8.dp))
            Text(
                "Ensure VoidCaller has 'Call Phone', 'Phone State', 'Contacts', and 'Display pop-up windows while running in the background' enabled.",
                color = VoidTextMuted,
                style = MaterialTheme.typography.bodySmall
            )
            Spacer(modifier = Modifier.height(12.dp))
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
            Spacer(modifier = Modifier.height(24.dp))
        }
    }
}
