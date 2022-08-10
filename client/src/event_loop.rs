use std::{
    sync::mpsc::TryRecvError,
    thread::{self, sleep, JoinHandle},
    time::Duration,
};

use crate::{
    audio::{start_output_stream, AudioState},
    common::{UserAction, Communicator, LoopStatus, LoopMessage},
    socket::{socket_connect, SocketState},
};

use anyhow::{format_err, Result};

pub fn start_event_loop<F>(
    comm: Communicator<LoopMessage, UserAction>,
    gui_context: F,
) -> JoinHandle<()>
where
    F: Fn() + Send + Sync + 'static,
{
    thread::spawn(move || {
        let mut state = LoopState {
            address: String::new(),
            audio_state: None,
            socket_state: None,
            comm,
            status: LoopStatus::Ready,
            gui_context,
        };
        state.start_loop();
        println!("Exiting");
    })
}

struct LoopState<F>
where
    F: Fn() + Send + Sync + 'static,
{
    address: String,
    socket_state: Option<SocketState>,
    comm: Communicator<LoopMessage, UserAction>,
    audio_state: Option<AudioState>,
    status: LoopStatus,
    gui_context: F,
}

impl<F> LoopState<F>
where
    F: Fn() + Send + Sync + 'static,
{
    fn send(&mut self, message: LoopMessage) {
        self.comm.send(message).expect("Cannot send message");
        (self.gui_context)();
    }

    fn seek(&mut self) {
        if let Err(err) = self.socket_state.as_mut().unwrap().seek() {
            eprintln!("Cannot seek from socket: {}", err);
            self.disconnect().expect("Cannot disconnect from socket");
            self.send(LoopMessage::SocketReconnecting);
            for _ in 0..5 {
                match self.connect() {
                    Ok((socket, audio)) => {
                        self.socket_state.replace(socket);
                        self.audio_state.replace(audio);
                        self.send(LoopMessage::SocketConnected);
                        self.status = LoopStatus::Connected;
                        return;
                    }
                    Err(err) => {
                        eprintln!("Error reconnecting: {}", err);
                    }
                }
                sleep(Duration::from_secs(2));

            }
            self.send(LoopMessage::AudioStreamError("Lost connection".to_owned()));
        }
    }

    fn disconnect(&mut self) -> Result<()> {
        self.socket_state.take().unwrap().disconnect()?;
        self.audio_state.take().unwrap().stop()?;
        self.status = LoopStatus::Ready;
        Ok(())
    }

    fn connect(&mut self) -> Result<(SocketState, AudioState)> {
        let (producer, consumer) = ringbuf::RingBuffer::<i16>::new(10000).split();
        let audio_state = start_output_stream(consumer)?;
        let stream = socket_connect(self.address.as_str(), producer).map_err(|err| {
            eprintln!("Connection error: {:?}", err);
            match err {
                crate::socket::SocketError::AddressError => format_err!("Device address invalid"),
                crate::socket::SocketError::ConnectionError => {
                    format_err!("Error connecting to device")
                }
                crate::socket::SocketError::SetupError => format_err!("Internal error"),
            }
        })?;
        Ok((stream, audio_state))
    }

    fn start_loop(&mut self) {
        loop {
            match self.status {
                LoopStatus::Ready => match self.comm.receive().unwrap() {
                    UserAction::Connect(address) => {
                        self.address = address;
                        match self.connect() {
                            Ok((socket, audio)) => {
                                self.status = LoopStatus::Connected;
                                self.audio_state = Some(audio);
                                self.socket_state = Some(socket);
                                self.send(LoopMessage::SocketConnected);
                            }
                            Err(err) => {
                                self.status = LoopStatus::Ready;
                                self.send(LoopMessage::AudioStreamError(err.to_string()));
                            }
                        }
                    }
                    UserAction::UserDisconnect => eprintln!("Not connected yet"),
                    UserAction::Exit => break,
                },
                LoopStatus::Connected => match self.comm.try_receive() {
                    Ok(message) => match message {
                        UserAction::Connect(_) => {
                            eprintln!("Already connected");
                        }
                        UserAction::UserDisconnect => match self.disconnect() {
                            Err(err) => {
                                eprintln!("Error disconnecting");
                                self.send(LoopMessage::AudioStreamError(err.to_string()));
                            }
                            Ok(()) => {
                                self.send(LoopMessage::SocketClosed);
                            }
                        },
                        UserAction::Exit => break,
                    },
                    Err(err) => match err {
                        TryRecvError::Empty => {
                            self.seek();
                        }
                        TryRecvError::Disconnected => {
                            eprintln!("Communicator disconnected");
                            break;
                        }
                    },
                },
            }
        }
    }
}
