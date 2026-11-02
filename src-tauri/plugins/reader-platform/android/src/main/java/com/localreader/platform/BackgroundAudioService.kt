package com.localreader.platform

import android.app.Notification
import android.app.NotificationChannel
import android.app.NotificationManager
import android.app.PendingIntent
import android.app.Service
import android.content.Intent
import android.content.pm.ServiceInfo
import android.media.session.MediaSession
import android.media.session.PlaybackState
import android.os.Build
import android.os.IBinder

class BackgroundAudioService : Service() {
    companion object {
        const val ACTION_START = "com.localreader.platform.audio.START"
        const val ACTION_STOP = "com.localreader.platform.audio.STOP"
        const val ACTION_PLAY = "com.localreader.platform.audio.PLAY"
        const val ACTION_PAUSE = "com.localreader.platform.audio.PAUSE"
        const val ACTION_COMMAND = "com.localreader.platform.audio.COMMAND"
        const val EXTRA_TITLE = "title"
        const val EXTRA_PLAYING = "playing"
        const val EXTRA_COMMAND = "command"
        private const val CHANNEL_ID = "local-reader-read-aloud"
        private const val NOTIFICATION_ID = 4101
    }

    private var mediaSession: MediaSession? = null
    private var playing = true
    private var title = "正在朗读"

    override fun onCreate() {
        super.onCreate()
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
            val channel = NotificationChannel(
                CHANNEL_ID,
                "朗读",
                NotificationManager.IMPORTANCE_LOW
            )
            channel.description = "朗读后台播放"
            getSystemService(NotificationManager::class.java).createNotificationChannel(channel)
        }
        mediaSession = MediaSession(this, "local-reader-read-aloud").apply {
            setCallback(object : MediaSession.Callback() {
                override fun onPlay() {
                    updatePlaying(true)
                    sendCommand("play")
                }

                override fun onPause() {
                    updatePlaying(false)
                    sendCommand("pause")
                }

                override fun onStop() {
                    sendCommand("stop")
                    stopForeground(STOP_FOREGROUND_REMOVE)
                    stopSelf()
                }
            })
            isActive = true
        }
        updatePlaybackState()
    }

    override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
        when (intent?.action) {
            ACTION_STOP -> {
                sendCommand("stop")
                stopForeground(STOP_FOREGROUND_REMOVE)
                stopSelf()
                return START_NOT_STICKY
            }
            ACTION_PLAY -> {
                updatePlaying(true)
                sendCommand("play")
            }
            ACTION_PAUSE -> {
                updatePlaying(false)
                sendCommand("pause")
            }
            ACTION_START -> {
                title = intent.getStringExtra(EXTRA_TITLE)?.ifBlank { null } ?: "正在朗读"
                playing = intent.getBooleanExtra(EXTRA_PLAYING, true)
                updatePlaybackState()
            }
        }
        startForegroundNotification()
        return START_NOT_STICKY
    }

    private fun updatePlaying(next: Boolean) {
        playing = next
        updatePlaybackState()
        startForegroundNotification()
    }

    private fun updatePlaybackState() {
        val session = mediaSession ?: return
        val state = if (playing) PlaybackState.STATE_PLAYING else PlaybackState.STATE_PAUSED
        val actions = PlaybackState.ACTION_PLAY or
            PlaybackState.ACTION_PAUSE or
            PlaybackState.ACTION_PLAY_PAUSE or
            PlaybackState.ACTION_STOP
        session.setPlaybackState(
            PlaybackState.Builder()
                .setActions(actions)
                .setState(state, PlaybackState.PLAYBACK_POSITION_UNKNOWN, if (playing) 1f else 0f)
                .build()
        )
    }

    private fun startForegroundNotification() {
        val notification = buildNotification()
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.Q) {
            startForeground(
                NOTIFICATION_ID,
                notification,
                ServiceInfo.FOREGROUND_SERVICE_TYPE_MEDIA_PLAYBACK
            )
        } else {
            startForeground(NOTIFICATION_ID, notification)
        }
    }

    private fun sendCommand(command: String) {
        val intent = Intent(ACTION_COMMAND)
            .setPackage(packageName)
            .putExtra(EXTRA_COMMAND, command)
        sendBroadcast(intent)
    }

    private fun buildNotification(): Notification {
        val flags = PendingIntent.FLAG_UPDATE_CURRENT or PendingIntent.FLAG_IMMUTABLE
        val stopIntent = Intent(this, BackgroundAudioService::class.java).apply {
            action = ACTION_STOP
        }
        val toggleIntent = Intent(this, BackgroundAudioService::class.java).apply {
            action = if (playing) ACTION_PAUSE else ACTION_PLAY
        }
        val stopPending = PendingIntent.getService(this, 1, stopIntent, flags)
        val togglePending = PendingIntent.getService(this, 2, toggleIntent, flags)
        val launchIntent = packageManager.getLaunchIntentForPackage(packageName)
        val contentPending = launchIntent?.let {
            PendingIntent.getActivity(this, 3, it, flags)
        }
        val builder = if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
            Notification.Builder(this, CHANNEL_ID)
        } else {
            Notification.Builder(this)
        }
        builder
            .setContentTitle("本地阅读")
            .setContentText(if (playing) title else "已暂停")
            .setSmallIcon(android.R.drawable.ic_media_play)
            .setOngoing(true)
            .setOnlyAlertOnce(true)
            .setVisibility(Notification.VISIBILITY_PUBLIC)
            .addAction(
                if (playing) android.R.drawable.ic_media_pause else android.R.drawable.ic_media_play,
                if (playing) "暂停" else "继续",
                togglePending
            )
            .addAction(android.R.drawable.ic_menu_close_clear_cancel, "停止", stopPending)
        if (contentPending != null) builder.setContentIntent(contentPending)
        mediaSession?.let { session ->
            builder.setStyle(
                Notification.MediaStyle()
                    .setMediaSession(session.sessionToken)
                    .setShowActionsInCompactView(0, 1)
            )
        }
        return builder.build()
    }

    override fun onDestroy() {
        mediaSession?.isActive = false
        mediaSession?.release()
        mediaSession = null
        super.onDestroy()
    }

    override fun onBind(intent: Intent?): IBinder? = null
}
