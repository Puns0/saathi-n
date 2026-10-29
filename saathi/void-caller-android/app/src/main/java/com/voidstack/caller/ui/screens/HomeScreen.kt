package com.voidstack.caller.ui.screens

import android.content.Context
import android.content.Intent
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Settings
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.navigation.compose.NavHost
import androidx.navigation.compose.composable
import androidx.navigation.compose.rememberNavController
import com.voidstack.caller.services.ConnectionService
import com.voidstack.caller.ui.theme.VoidDarkBackground
import com.voidstack.caller.ui.theme.VoidPrimary
import com.voidstack.caller.ui.theme.VoidSurface
import com.voidstack.caller.ui.theme.VoidTextMuted

@Composable
fun MainApp() {
    val navController = rememberNavController()
    NavHost(navController = navController, startDestination = "home") {
        composable("home") { HomeScreen(onNavigateToSettings = { navController.navigate("settings") }) }
        composable("settings") { SettingsScreen(onNavigateBack = { navController.popBackStack() }) }
    }
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun HomeScreen(onNavigateToSettings: () -> Unit) {
    val context = LocalContext.current
    var isServiceRunning by remember { mutableStateOf(false) } // In reality, check if service is running

    Scaffold(
        topBar = {
            TopAppBar(
                title = { Text("VoidCaller", fontWeight = FontWeight.Bold) },
                colors = TopAppBarDefaults.topAppBarColors(
                    containerColor = VoidDarkBackground,
                    titleContentColor = MaterialTheme.colorScheme.onBackground
                ),
                actions = {
                    IconButton(onClick = onNavigateToSettings) {
                        Icon(Icons.Filled.Settings, contentDescription = "Settings", tint = VoidTextMuted)
                    }
                }
            )
        },
        containerColor = VoidDarkBackground
    ) { paddingValues ->
        Column(
            modifier = Modifier
                .fillMaxSize()
                .padding(paddingValues)
                .padding(16.dp),
            horizontalAlignment = Alignment.CenterHorizontally,
            verticalArrangement = Arrangement.Center
        ) {
            Box(
                modifier = Modifier
                    .size(150.dp)
                    .clip(RoundedCornerShape(75.dp))
                    .background(if (isServiceRunning) VoidPrimary.copy(alpha = 0.2f) else VoidSurface)
                    .padding(8.dp),
                contentAlignment = Alignment.Center
            ) {
                Box(
                    modifier = Modifier
                        .fillMaxSize()
                        .clip(RoundedCornerShape(75.dp))
                        .background(if (isServiceRunning) VoidPrimary else VoidSurface),
                    contentAlignment = Alignment.Center
                ) {
                    Text(
                        text = if (isServiceRunning) "ON" else "OFF",
                        color = MaterialTheme.colorScheme.onPrimary,
                        fontSize = 32.sp,
                        fontWeight = FontWeight.Bold
                    )
                }
            }

            Spacer(modifier = Modifier.height(32.dp))

            Text(
                text = "Relay Connection",
                fontSize = 20.sp,
                fontWeight = FontWeight.Bold,
                color = MaterialTheme.colorScheme.onBackground
            )
            
            Text(
                text = if (isServiceRunning) "Connected to PC" else "Disconnected",
                color = VoidTextMuted,
                modifier = Modifier.padding(top = 8.dp)
            )

            Spacer(modifier = Modifier.height(48.dp))

            Button(
                onClick = {
                    val intent = Intent(context, ConnectionService::class.java)
                    if (isServiceRunning) {
                        context.stopService(intent)
                        isServiceRunning = false
                    } else {
                        context.startForegroundService(intent)
                        isServiceRunning = true
                    }
                },
                modifier = Modifier
                    .fillMaxWidth()
                    .height(56.dp),
                colors = ButtonDefaults.buttonColors(
                    containerColor = if (isServiceRunning) MaterialTheme.colorScheme.error else VoidPrimary
                ),
                shape = RoundedCornerShape(12.dp)
            ) {
                Text(
                    if (isServiceRunning) "Stop Relay Connection" else "Start Relay Connection",
                    fontSize = 16.sp,
                    fontWeight = FontWeight.Bold
                )
            }
        }
    }
}
