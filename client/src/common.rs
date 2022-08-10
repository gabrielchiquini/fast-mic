use std::sync::mpsc::{self, Iter, Receiver, Sender};

use anyhow::Result;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum GuiStatus {
    Ready,
    Connecting,
    Reconnecting,
    Connected,
    Failed,
    Disconnecting,
}

pub enum LoopStatus {
    Ready,
    Connected,
}

impl GuiStatus {
    pub fn can_connect(&self) -> bool {
        match self {
            GuiStatus::Ready | GuiStatus::Failed => true,
            GuiStatus::Connecting
            | GuiStatus::Reconnecting
            | GuiStatus::Connected
            | GuiStatus::Disconnecting => false,
        }
    }
}

impl Default for GuiStatus {
    fn default() -> Self {
        Self::Ready
    }
}

pub struct Communicator<S, T>
where
    S: Send + Sync,
    T: Send + Sync,
{
    sender: Sender<S>,
    receiver: Receiver<T>,
}

impl<S: Send + Sync + 'static, T: Send + Sync> Communicator<S, T> {
    pub fn send(&self, message: S) -> Result<()> {
        self.sender.send(message)?;
        Ok(())
    }

    pub fn try_receive(&mut self) -> Result<T, mpsc::TryRecvError> {
        self.receiver.try_recv()
    }

    pub fn receive(&mut self) -> Result<T> {
        self.receiver.recv().map_err(anyhow::Error::msg)
    }

    pub fn iter(&mut self) -> Iter<T> {
        self.receiver.iter()
    }

    pub fn create_pair() -> (Communicator<S, T>, Communicator<T, S>)
    {
        let (sender_1, receiver_2) = mpsc::channel::<S>();
        let (sender_2, receiver_1) = mpsc::channel::<T>();
        (
            Communicator {
                sender: sender_1,
                receiver: receiver_1,
            },
            Communicator {
                sender: sender_2,
                receiver: receiver_2,
            },
        )
    }

    pub fn split(self) -> (Sender<S>, Receiver<T>) {
        (self.sender, self.receiver)
    }
}

#[derive(Debug, Clone)]
pub enum LoopMessage {
    Ready,
    SocketConnected,
    SocketCannotConnect,
    SocketClosed,
    SocketReconnecting,
    AudioStreamError(String),
}

#[derive(Debug, Clone)]
pub enum UserAction {
    Connect(String),
    UserDisconnect,
    Exit,
}
