package com.example.fast_mic

import android.Manifest
import android.content.pm.PackageManager
import android.media.AudioFormat
import android.media.AudioRecord
import android.media.MediaRecorder
import android.media.audiofx.NoiseSuppressor
import android.util.Log
import androidx.core.app.ActivityCompat
import io.flutter.embedding.android.FlutterActivity
import io.flutter.embedding.engine.FlutterEngine
import io.flutter.plugin.common.MethodChannel
import java.util.*
import java.util.concurrent.Semaphore
import java.util.concurrent.atomic.AtomicBoolean
import kotlin.collections.ArrayList
import kotlin.concurrent.thread


class MainActivity : FlutterActivity() {
    private val sampleRate = 48000
    private val recorderChannelsIn = AudioFormat.CHANNEL_IN_MONO
    private val encoding = AudioFormat.ENCODING_PCM_16BIT
    private val bufferSize = AudioRecord.getMinBufferSize(
        sampleRate,
        recorderChannelsIn,
        encoding
    )


    private var recorder: AudioRecord? = null
    private val tag = "RECORDER"
    private val tagAux = "AUX-T"

    private fun createRecorder(): AudioRecord {
        val recorder = if (ActivityCompat.checkSelfPermission(
                this,
                Manifest.permission.RECORD_AUDIO
            ) != PackageManager.PERMISSION_GRANTED
        ) {
            throw RuntimeException("Permission not granted for recording")
        } else {
            AudioRecord.Builder()
                .setAudioFormat(
                    AudioFormat.Builder().setEncoding(AudioFormat.ENCODING_PCM_16BIT)
                        .setSampleRate(sampleRate).build()
                )
                .setAudioSource(MediaRecorder.AudioSource.MIC)
                .setBufferSizeInBytes(bufferSize)
                .build()


        }
        NoiseSuppressor.create(recorder.audioSessionId)
        return recorder
    }


    override fun configureFlutterEngine(flutterEngine: FlutterEngine) {
        super.configureFlutterEngine(flutterEngine)
        val sharedBufferSemaphore = Semaphore(1)
        val sharedBuffer = LinkedList<Short>()
        val threadSignal = AtomicBoolean()
        var readerThread: Thread? = null

        MethodChannel(
            flutterEngine.dartExecutor.binaryMessenger,
            "fast_mic/recording"
        ).setMethodCallHandler { call, result ->
            when (call.method) {
                "start" -> {
                    sharedBuffer.clear()
                    recorder = createRecorder()
                    recorder!!.startRecording()
                    threadSignal.set(true)
                    readerThread = spawnReader(sharedBufferSemaphore, sharedBuffer, threadSignal)
                    Log.d(tag, "Starting recorder, buffer size: $bufferSize")
                    result.success(null)
                }
                "poll" -> {
                    val resultBuffer = ArrayList<Short>(sharedBuffer.size)
                    sharedBufferSemaphore.acquire()
                    while (sharedBuffer.size > 0) {
                        resultBuffer.add(sharedBuffer.poll()!!)
                    }
                    sharedBufferSemaphore.release()
//                    Log.d(tag, "Polled ${resultBuffer.size} from sharedBuffer")
                    result.success(resultBuffer)
                }
                "stop" -> {
                    Log.d(tag, "Stopping recorder")
                    threadSignal.set(false)
                    recorder?.stop()
                    readerThread?.join()
                    result.success(null)
                }
                else -> {
                    result.notImplemented()
                }
            }

        }
    }


    private fun spawnReader(
        sharedBufferSemaphore: Semaphore,
        sharedBuffer: LinkedList<Short>,
        readFlag: AtomicBoolean
    ): Thread {
        return thread(isDaemon = true, start = true) {
            val buffer = ShortArray(bufferSize)
            Log.d(tagAux, "Starting reader thread")
            while (readFlag.get()) {
                if (recorder != null) {
                    val readShorts = recorder!!.read(buffer, 0, bufferSize)
//                    Log.d(tagAux, "Read $readShorts from recorder")
                    sharedBufferSemaphore.acquire()
                    sharedBuffer.addAll(buffer.asList())
                    sharedBufferSemaphore.release()
                }
            }
            Log.d(tagAux, "Exiting reader thread")
        }
    }

}
