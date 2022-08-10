# Fast Mic

Fast Mic is an open-source Android app and a Windows client which allows you use your phone as a mic. It relies on [VB-Audio Cable](https://vb-audio.com/Cable/) to stream sound input.


## Build

### App
    cd app
    flutter pub get
    flutter build apk

### Client
    cd client
    cargo build --release
    

## Usage
Install VB-CABLE and the generated APK. 
With both PC and phone connected to the same LAN, start the server in the mobile app and connect the client to it. The output will be sent to `CABLE Output` device
