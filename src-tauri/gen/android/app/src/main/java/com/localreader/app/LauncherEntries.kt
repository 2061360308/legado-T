package com.localreader.app

import android.app.Activity
import android.content.Intent
import android.os.Bundle

open class LauncherEntryActivity : Activity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        startActivity(Intent(this, MainActivity::class.java))
        finish()
    }
}

class LauncherDefault : LauncherEntryActivity()

class Launcher1 : LauncherEntryActivity()

class Launcher2 : LauncherEntryActivity()

class Launcher3 : LauncherEntryActivity()

class Launcher4 : LauncherEntryActivity()

class Launcher5 : LauncherEntryActivity()

class Launcher6 : LauncherEntryActivity()
