use rodio::{Decoder, OutputStream, Sink, OutputStreamHandle};
use rodio::source::{SineWave, Source, Amplify};
use std::{thread, time};

pub struct Synth {
    sink: Sink,
    stream: OutputStream,
    stream_handle: OutputStreamHandle,
    //source: Amplify<SineWave>,
    pub is_playing: bool,
}

impl Synth {
    pub fn new() -> Self {
        let (_stream, stream_handle) = OutputStream::try_default().unwrap();
        let sink = Sink::try_new(&stream_handle).unwrap();
        let source = SineWave::new(440.0).amplify(0.20);
        sink.append(source);
        //sink.sleep_until_end();
        sink.pause();
        Synth { 
            sink: sink,
            stream: _stream,
            //source: source,
            stream_handle: stream_handle,
            is_playing: false,
        }
    }

    pub fn play(&mut self) {
        self.is_playing = true;
        self.sink.play();
    }

    pub fn pause(&mut self) {
        self.is_playing = false;
        self.sink.pause();
    }
}